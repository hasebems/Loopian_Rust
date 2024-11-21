//  Created by Hasebe Masahiko on 2024/10/07.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::cmdparse::*;
use super::txt_common::*;
use crate::lpnlib::*;

impl LoopianCmd {
    pub fn parse_set_command(&mut self, input_text: &str) -> String {
        if let Some((cmd, prm)) = separate_cmnd_and_str(&input_text[4..]) {
            if cmd == "key" {
                if self.change_key(prm) {
                    "Key has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            } else if cmd == "oct" {
                if self.change_oct(prm) {
                    "Octave has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            } else if cmd == "bpm" {
                match prm.parse::<i16>() {
                    Ok(msg) => {
                        self.dtstk.change_bpm(msg);
                        self.sndr
                            .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_BPM, msg]));
                        self.sndr
                            .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
                        "BPM has changed!".to_string()
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        "Number is wrong.".to_string()
                    }
                }
            } else if cmd == "beat" || cmd == "meter" {
                let numvec = split_by('/', prm.to_string());
                if numvec.len() < 2 {
                    "Number is wrong.".to_string()
                } else {
                    match (numvec[0].parse::<i16>(), numvec[1].parse::<i16>()) {
                        (Ok(numerator), Ok(denomirator)) => {
                            self.dtstk.change_beat(numerator, denomirator);
                            self.sndr
                                .send_msg_to_elapse(ElpsMsg::SetBeat([numerator, denomirator]));
                            self.sndr
                                .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
                            "Beat has changed!".to_string()
                        }
                        _ => "Number is wrong.".to_string(),
                    }
                }
            } else if cmd == "input" {
                if self.change_input_mode(prm) {
                    "Input mode has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            } else if cmd == "samenote" {
                "what?".to_string()
            } else if cmd == "turnnote" {
                if self.change_turnnote(prm) {
                    "Turn note has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            } else if cmd == "path" {
                if self.change_path(prm) {
                    "Path has changed!".to_string()
                } else {
                    "what?".to_string()
                }
            } else {
                "what?".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    //*************************************************************************
    fn change_key(&mut self, key_text: &str) -> bool {
        let mut key = END_OF_DATA;
        let length = key_text.len();
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
            if length >= 2 {
                let mut num_txt = "".to_string();
                if let Some(ltr2) = key_text.chars().nth(1) {
                    match ltr2 {
                        '#' => {
                            key += 1;
                            num_txt = key_text[2..].to_string();
                        }
                        'b' => {
                            key -= 1;
                            num_txt = key_text[2..].to_string();
                        }
                        _ => {
                            num_txt = key_text[1..].to_string();
                        }
                    }
                }
                if let Ok(oct_num) = num_txt.parse::<i32>() {
                    oct = oct_num;
                }
            }
            if key < 0 {
                key += 12;
            } else if key >= 12 {
                key -= 12;
            }
            #[cfg(feature = "verbose")]
            println!("CHANGE KEY: {}, {}", key, oct);
            // phrase 再生成(新oct込み)
            if oct != 0 {
                if self.dtstk.change_oct(oct, false, self.get_input_part()) {
                    self.sndr
                        .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
                }
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
    fn change_oct(&mut self, oct_txt: &str) -> bool {
        let mut oct = FULL;
        if let Ok(oct_num) = oct_txt.parse::<i32>() {
            oct = oct_num;
        }
        if oct != FULL {
            if self.dtstk.change_oct(oct, true, self.get_input_part()) {
                self.sndr
                    .send_all_vari_and_phrase(self.get_input_part(), &self.dtstk);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    fn change_input_mode(&mut self, imd: &str) -> bool {
        if imd == "fixed" {
            self.dtstk.change_input_mode(InputMode::Fixed);
            true
        } else if imd == "closer" {
            self.dtstk.change_input_mode(InputMode::Closer);
            true
        } else {
            false
        }
    }
    fn change_turnnote(&mut self, ntnum: &str) -> bool {
        if let Ok(turn_note) = ntnum.parse::<i16>() {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_TURN, turn_note]));
            true
        } else {
            false
        }
    }
    fn change_path(&mut self, path: &str) -> bool {
        self.path(path.to_string());
        true
    }
}
