//  Created by Hasebe Masahiko on 2024/10/07.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::cmdparse::*;
use crate::common::txt_common::*;
use crate::lpnlib::*;

//*******************************************************************
//      Set Command: Enum と純粋パース
//*******************************************************************
pub enum SetCommand {
    Key(String),
    Oct(String),
    Bpm(i16),
    Meter(i16, i16),
    Msr(i16),
    Input(String),
    SameNote,
    TurnNote(i16),
    Path(String),
    FlowReso(i16),
    FlowVel(i16),
    MidiInputCh(u8),
}

pub enum SetCmdError {
    UnknownCommand,   // "what?"
    BadNumber,        // "Number is wrong."
    BadChannel,       // "Channel number is wrong."
}

impl SetCommand {
    /// 入力文字列を型付きコマンドに変換する純粋関数（副作用なし）
    pub fn parse(input_text: &str) -> Result<Self, SetCmdError> {
        let (cmd, prm) = separate_cmnd_and_str(input_text)
            .ok_or(SetCmdError::UnknownCommand)?;
        match cmd {
            "key" => Ok(Self::Key(prm.to_string())),
            "oct" => Ok(Self::Oct(prm.to_string())),
            "bpm" => prm
                .parse::<i16>()
                .map(Self::Bpm)
                .map_err(|_| SetCmdError::BadNumber),
            "beat" | "meter" => {
                let parts = split_by('/', prm.to_string());
                if parts.len() < 2 {
                    return Err(SetCmdError::BadNumber);
                }
                let n = parts[0].parse::<i16>().map_err(|_| SetCmdError::BadNumber)?;
                let d = parts[1].parse::<i16>().map_err(|_| SetCmdError::BadNumber)?;
                Ok(Self::Meter(n, d))
            }
            "msr" => {
                let msr = prm.parse::<i16>().map_err(|_| SetCmdError::BadNumber)?;
                if msr < 1 {
                    return Err(SetCmdError::BadNumber);
                }
                Ok(Self::Msr(msr))
            }
            "input" => Ok(Self::Input(prm.to_string())),
            "samenote" => Ok(Self::SameNote),
            "turnnote" => prm
                .parse::<i16>()
                .map(Self::TurnNote)
                .map_err(|_| SetCmdError::UnknownCommand),
            "path" => Ok(Self::Path(prm.to_string())),
            "flowreso" => prm
                .parse::<i16>()
                .map(Self::FlowReso)
                .map_err(|_| SetCmdError::UnknownCommand),
            "flowvel" => prm
                .parse::<i16>()
                .map(Self::FlowVel)
                .map_err(|_| SetCmdError::UnknownCommand),
            "midi_input_ch" => {
                let ch = prm.parse::<u8>().map_err(|_| SetCmdError::BadChannel)?;
                if !(1..=16).contains(&ch) {
                    return Err(SetCmdError::BadChannel);
                }
                Ok(Self::MidiInputCh(ch))
            }
            _ => Err(SetCmdError::UnknownCommand),
        }
    }
}

impl LoopianCmd {
    /// setコマンドのエントリ: パース → 実行 の2段構成
    pub fn parse_set_command(&mut self, input_text: &str) -> String {
        match SetCommand::parse(input_text) {
            Ok(cmd) => self.execute_set_command(cmd),
            Err(SetCmdError::UnknownCommand) => "what?".to_string(),
            Err(SetCmdError::BadNumber) => "Number is wrong.".to_string(),
            Err(SetCmdError::BadChannel) => "Channel number is wrong.".to_string(),
        }
    }
    fn execute_set_command(&mut self, cmd: SetCommand) -> String {
        match cmd {
            SetCommand::Key(prm) => {
                if self.change_key(&prm) {
                    "Key has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            }
            SetCommand::Oct(prm) => {
                let part_num = self.get_input_part();
                if self.change_oct(&prm, part_num) {
                    "Octave has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            }
            SetCommand::Bpm(bpm) => {
                self.change_bpm(bpm);
                "BPM has changed!".to_string()
            }
            SetCommand::Meter(n, d) => {
                self.change_meter(n, d);
                "Meter has changed!".to_string()
            }
            SetCommand::Msr(msr) => {
                self.set_measure(msr - 1); // 1小節前にセット
                "Measure has changed!".to_string()
            }
            SetCommand::Input(prm) => {
                if self.change_input_mode(&prm) {
                    "Input mode has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            }
            SetCommand::SameNote => "what?".to_string(),
            SetCommand::TurnNote(turn_note) => {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_TURN, turn_note]));
                "Turn note has changed!".to_string()
            }
            SetCommand::Path(prm) => {
                self.change_path(&prm);
                "Path has changed!".to_string()
            }
            SetCommand::FlowReso(reso) => {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_FLOW_TICK_RESOLUTION, reso]));
                "Flow tick resolution has changed!".to_string()
            }
            SetCommand::FlowVel(vel) => {
                if !(1..=127).contains(&vel) {
                    return "what?".to_string();
                }
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_FLOW_VELOCITY, vel]));
                "Flow velocity has changed!".to_string()
            }
            SetCommand::MidiInputCh(ch) => {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_MIDI_INPUT_CH, ch as i16]));
                "MIDI Input Ch has changed!".to_string()
            }
        }
    }
    //*************************************************************************
    pub fn change_key(&mut self, key_text: &str) -> bool {
        let mut key = END_OF_DATA;
        match key_text.chars().nth(0) {
            Some('C') => key = 0,
            Some('D') => key = 2,
            Some('E') => key = 4,
            Some('F') => key = 5,
            Some('G') => key = 7,
            Some('A') => key = 9,
            Some('B') => key = 11,
            Some(_) => (),
            None => (),
        }
        if key != END_OF_DATA {
            let mut oct = 0;
            let mut num_txt = "".to_string();
            if key_text.len() >= 2
                && let Some(ltr2) = key_text.chars().nth(1)
            {
                num_txt = match ltr2 {
                    '#' => {
                        key += 1;
                        if key_text.len() >= 3 {
                            key_text[2..].to_string()
                        } else {
                            String::new()
                        }
                    }
                    'b' => {
                        key -= 1;
                        if key_text.len() >= 3 {
                            key_text[2..].to_string()
                        } else {
                            String::new()
                        }
                    }
                    _ => key_text[1..].to_string(),
                };
            }
            if let Ok(oct_num) = num_txt.parse::<i32>() {
                oct = oct_num;
            }
            if key < 0 {
                key += 12;
            } else if key >= 12 {
                key -= 12;
            }
            #[cfg(feature = "verbose")]
            println!("CHANGE KEY: {key}, {oct}");
            // phrase 再生成(新oct込み)
            if oct != 0 && self.dtstk.change_oct(oct, false, self.get_input_part()) {
                self.sndr
                    .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
            }
            // elapse に key を送る
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_KEY, key as i16]));
            self.indicator_key_stock(key_text.to_string());
            true
        } else {
            false
        }
    }
    pub fn change_oct(&mut self, oct_txt: &str, part_num: usize) -> bool {
        if let Ok(oct_num) = oct_txt.parse::<i32>() {
            if self.dtstk.change_oct(oct_num, true, part_num) {
                self.sndr.send_all_vari_and_phrase(part_num, &self.dtstk);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn change_bpm(&mut self, bpm: i16) {
        self.dtstk.change_bpm(bpm);
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_BPM, bpm]));
        self.sndr
            .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
    }
    pub fn change_meter(&mut self, numerator: i16, denominator: i16) {
        self.dtstk.change_beat(numerator, denominator);
        self.sndr
            .send_msg_to_elapse(ElpsMsg::SetMeter([numerator, denominator]));
        self.sndr
            .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
    }
    fn change_input_mode(&mut self, imd: &str) -> bool {
        if imd == "fixed" {
            self.dtstk.change_input_mode(InputMode::Fixed);
            true
        } else if imd == "closer" {
            self.dtstk.change_input_mode(InputMode::Closer);
            true
        } else if imd == "upcloser" {
            self.dtstk.change_input_mode(InputMode::Upcloser);
            true
        } else {
            false
        }
    }
    fn change_path(&mut self, path: &str) {
        self.path(path.to_string());
    }
}
