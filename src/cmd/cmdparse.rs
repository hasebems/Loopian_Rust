//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;

use super::input_txt::CmndRtn;
use super::send_msg::*;
use super::seq_stock::*;
use crate::common::txt_common::*;
use super::txt2seq_cmps::*;
use crate::graphic::generative_view::{GraphicMsg, generate_graphic_msg};
use crate::lpnlib::*;

#[derive(Debug, Clone)]
pub struct CmdReply {
    pub text: String,
    pub graphic: GraphicMsg,
}

impl CmdReply {
    fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            graphic: GraphicMsg::NoMsg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdError {
    InvalidInput,
    UnknownCommand,
    BadNumber,
    BadChannel,
    InvalidPart,
    Phrase(PhraseCmdError),
    Composition(CompositionCmdError),
}

pub type CmdResult = Result<CmdReply, CmdError>;

pub fn cmd_error_to_text(error: &CmdError) -> String {
    match error {
        CmdError::InvalidInput => "Invalid input.".to_string(),
        CmdError::UnknownCommand => "what?".to_string(),
        CmdError::BadNumber => "Number is wrong.".to_string(),
        CmdError::BadChannel => "Channel number is wrong.".to_string(),
        CmdError::InvalidPart => "what?".to_string(),
        CmdError::Phrase(_) => "what?".to_string(),
        CmdError::Composition(_) => "what?".to_string(),
    }
}

//*******************************************************************
//      Tokenizer / Classifier (pure, no state)
//*******************************************************************
/// 入力テキストの先頭文字によるコマンド種別分類
pub enum CmdKind {
    TogglePlay,   // 単体の "."
    Slash,        // "/" で始まる
    At,           // "@" で始まる
    Bracket,      // "[" で始まる
    Brace,        // "{" で始まる
    PartSelect,   // part 名単体による current part 切替
    PartShortcut, // part 指定 + payload の shortcut 形式
    OneWord,      // トークン数 1
    MultiWord,    // トークン数 2 以上
}

/// 入力テキストと token 列からコマンド種別を判定する純関数
pub fn classify_cmd(input_text: &str, tokens: &[String]) -> CmdKind {
    let first = input_text.chars().next().unwrap_or(' ');
    if input_text.len() == 1 && first == '.' {
        return CmdKind::TogglePlay;
    }
    let token_count = tokens.len();
    let first_token = tokens.first().map(|token| token.as_str()).unwrap_or("");
    match first {
        '/' => CmdKind::Slash,
        '@' => CmdKind::At,
        '[' => CmdKind::Bracket,
        '{' => CmdKind::Brace,
        _ if token_count >= 2 && matches!(first, 'L' | 'R' | 'F' | 'A' | 'D' | 'S') => {
            CmdKind::PartShortcut
        }
        _ if detect_part(first_token).is_some() => CmdKind::PartSelect,
        _ if token_count == 1 => CmdKind::OneWord,
        _ => CmdKind::MultiWord,
    }
}

/// パート文字列を部番号に変換する純関数
pub fn detect_part(part_str: &str) -> Option<usize> {
    match part_str {
        "left1" | "L1" => Some(LEFT1),
        "left2" | "L2" => Some(LEFT2),
        "right1" | "R1" => Some(RIGHT1),
        "right2" | "R2" => Some(RIGHT2),
        _ => None,
    }
}

pub struct LoopianCmd {
    during_play: bool,
    indicator_key_stock: String,
    input_part: usize,
    path: Option<String>,
    pub dtstk: SeqDataStock,
    pub sndr: MessageSender,
}
impl LoopianCmd {
    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>) -> Self {
        Self {
            during_play: false,
            indicator_key_stock: "C".to_string(),
            input_part: RIGHT1,
            path: None,
            dtstk: SeqDataStock::new(),
            sndr: MessageSender::new(msg_hndr),
        }
    }
    pub fn get_indicator_key_stock(&self) -> String {
        self.indicator_key_stock.clone()
    }
    pub fn indicator_key_stock(&mut self, kstk: String) {
        self.indicator_key_stock = kstk;
    }
    pub fn get_input_part(&self) -> usize {
        self.input_part
    }
    pub fn get_path(&self) -> Option<String> {
        self.path.clone()
    }
    pub fn path(&mut self, path: String) {
        self.path = Some(path);
    }
    pub fn send_quit(&self) {
        self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_QUIT));
    }
    pub fn set_measure(&mut self, msr: i16) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_CRNT_MSR, msr]));
    }
    pub fn send_clear(&self) {
        self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_CLEAR));
        println!("*** All data has been erased at Elapse thread! ***");
    }
    pub fn set_riten(&mut self, percent: i16) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Rit([MSG_RIT_RITEN, percent]));
    }
    #[cfg(feature = "raspi")]
    pub fn send_reconnect(&self) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_MIDI_RECONNECT));
    }
    //*************************************************************************
    pub fn put_and_get_responce(&mut self, input_text: &str) -> Option<CmndRtn> {
        match self.execute_command(input_text) {
            Ok(reply) => Some(CmndRtn(reply.text, reply.graphic)),
            Err(CmdError::InvalidInput) => None,
            Err(error) => Some(CmndRtn(cmd_error_to_text(&error), GraphicMsg::NoMsg)),
        }
    }
    pub fn execute_command(&mut self, input_text: &str) -> CmdResult {
        if input_text.is_empty() || !input_text.is_ascii() {
            return Err(CmdError::InvalidInput);
        }
        let tokens = tokenize_cmd(input_text);
        println!("Analyze Text: {:?}", tokens);
        match classify_cmd(input_text, &tokens) {
            CmdKind::TogglePlay => Ok(CmdReply::text(self.letter_dot(input_text))),
            CmdKind::Slash => Ok(CmdReply::text(self.letter_slash(input_text))),
            CmdKind::At => Ok(CmdReply::text(self.letter_at(input_text))),
            CmdKind::Bracket => self
                .apply_phrase_to_part(self.input_part, tokens)
                .map(CmdReply::text),
            CmdKind::Brace => self
                .apply_composition_to_part(self.input_part, tokens)
                .map(CmdReply::text),
            CmdKind::PartSelect => self.change_current_part(&tokens[0]).map(CmdReply::text),
            CmdKind::PartShortcut => self.shortcut_input(tokens).map(CmdReply::text),
            CmdKind::OneWord => Ok(CmdReply::text(self.one_word_command(&tokens[0].clone()))),
            CmdKind::MultiWord => match tokens[0].as_str() {
                "set" => self.parse_set_command_result(&tokens[1]).map(CmdReply::text),
                "sync" => Ok(CmdReply::text(self.cmd_sync(&tokens[1]))),
                "clear" => Ok(CmdReply::text(self.cmd_clear(&tokens[1]))),
                "fine" => Ok(CmdReply::text(self.cmd_fine(&tokens[1]))),
                "help" => Ok(CmdReply::text(self.cmd_help(&tokens[1]))),
                "graph"  => {
                    let rtn = generate_graphic_msg(tokens);
                    Ok(CmdReply {
                        text: rtn.0,
                        graphic: rtn.1,
                    })
                }
                "effect" => Ok(CmdReply::text(self.cmd_effect(&tokens[1]))),
                _ => Err(CmdError::UnknownCommand),
            },
        }
    }
    fn one_word_command(&mut self, input_text: &str) -> String {
        match input_text {
            "hello" => self.cmd_hello(),
            "play" | "p" => self.cmd_play(),
            "panic" => self.cmd_panic(),
            "stop" | "end" => self.cmd_stop(),
            "sync" => self.cmd_sync(""),
            "clear" => self.cmd_clear(""),
            "fermata" => self.cmd_fermata(),
            "fine" => self.cmd_fine(""),
            "resume" => self.cmd_resume(),
            "reconnect" => self.cmd_reconnect(),
            "help" => self.cmd_help(""),
            "right1" => {
                self.input_part = RIGHT1;
                "Changed current part to right1.".to_string()
            }
            "right2" => {
                self.input_part = RIGHT2;
                "Changed current part to right2.".to_string()
            }
            "left1" => {
                self.input_part = LEFT1;
                "Changed current part to left1.".to_string()
            }
            "left2" => {
                self.input_part = LEFT2;
                "Changed current part to left2.".to_string()
            }
            "rit" => self.cmd_rit(""),
            _ => "what?".to_string(),
        }
    }
    fn cmd_clear(&mut self, input_part: &str) -> String {
        if input_part.is_empty() {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            self.during_play = false;
            // clear
            for i in 0..MAX_KBD_PART {
                self.clear_part(i);
            }
            self.send_clear();
            "all data erased!".to_string()
        } else if let Some(pnum) = detect_part(input_part) {
            println!("clear>>{input_part}");
            self.clear_part(pnum);
            match pnum {
                LEFT1 => "part L1 data erased!".to_string(),
                LEFT2 => "part L2 data erased!".to_string(),
                RIGHT1 => "part R1 data erased!".to_string(),
                RIGHT2 => "part R2 data erased!".to_string(),
                _ => "some part data erased!".to_string(),
            }
        } else if input_part == "env" {
            self.change_key("C");
            self.change_bpm(100);
            self.change_meter(4, 4);
            for i in 0..MAX_KBD_PART {
                self.change_oct("0", i);
            }
            "Environment data erased!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn cmd_effect(&mut self, parameter: &str) -> String {
        if parameter.contains("dmp(") {
            if let Some(dmp) = extract_number_from_parentheses(parameter) {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Efct([MSG_EFCT_DMP, dmp as i16]));
                format!("Set Damper Value: {}", dmp)
            } else {
                "No Value!".to_string()
            }
        } else if parameter.contains("cc70(") {
            if let Some(cc70) = extract_number_from_parentheses(parameter) {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Efct([MSG_EFCT_CC70, cc70 as i16]));
                format!("Set CC70 Value: {}", cc70)
            } else {
                "No Value!".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    fn cmd_fermata(&mut self) -> String {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Rit([MSG_RIT_NRM, MSG2_RIT_FERMATA]));
        "Will stop!".to_string()
    }
    fn cmd_fine(&mut self, input_next: &str) -> String {
        if input_next.is_empty() {
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_FINE));
        } else if input_next == "next" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_FINE_NEXT_2BAR));
        } else if input_next == "beat(2)" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_FINE_NEXT_2BEAT));
        } else if input_next == "beat(3)" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_FINE_NEXT_3BEAT));
        } else if input_next == "beat(4)" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_FINE_NEXT_4BEAT));
        }
        self.during_play = false;
        "Fine.".to_string()
    }
    fn cmd_play(&mut self) -> String {
        if !self.during_play {
            // play
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_START));
            self.during_play = true;
            "Phrase has started!".to_string()
        } else {
            "Playing now!".to_string()
        }
    }
    fn cmd_panic(&mut self) -> String {
        // panic
        self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_PANIC));
        "All Sound Off!".to_string()
    }
    fn cmd_resume(&mut self) -> String {
        if !self.during_play {
            // resume
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_RESUME));
            self.during_play = true;
            "Resume.".to_string()
        } else {
            "Playing now!".to_string()
        }
    }
    fn cmd_rit(&mut self, input_text: &str) -> String {
        self.apply_rit(input_text)
    }
    fn cmd_reconnect(&mut self) -> String {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_MIDI_RECONNECT));
        "Send reconnect".to_string()
    }
    fn cmd_stop(&mut self) -> String {
        if self.during_play {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            self.during_play = false;
            "Stopped!".to_string()
        } else {
            "Settle down!".to_string()
        }
    }
    fn cmd_sync(&mut self, part_text: &str) -> String {
        if part_text.is_empty() {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Sync(self.input_part as i16));
            "Synchronized!".to_string()
        } else if part_text == "right" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_RGT));
            "Right Part Synchronized!".to_string()
        } else if part_text == "left" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_LFT));
            "Left Part Synchronized!".to_string()
        } else if part_text == "all" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_ALL));
            "All Part Synchronized!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn cmd_hello(&mut self) -> String {
        "Hi, hello!".to_string()
    }
    fn cmd_help(&mut self, input_text: &str) -> String {
        if input_text.is_empty() {
            "How can I assist you?".to_string()
        } else if input_text == "graph" {
            "ripple/voice/lissa/beatlissa()/sinewave/rain/fish/jumping/wavestick".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_slash(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len >= 2 && &input_text[0..2] == "//" {
            "....Gotcha.".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_at(&mut self, input_text: &str) -> String {
        let split_txt = split_by('=', input_text.to_string());
        if split_txt.len() == 2 {
            let len = split_txt[0].chars().count();
            if len >= 4 && &input_text[0..4] == "@msr" {
                let msr;
                if let Some(m) = extract_number_from_parentheses(&split_txt[0]) {
                    msr = m;
                } else {
                    msr = 1;
                }
                match self.put_phrase(
                    self.input_part,
                    PhraseAs::Measure(msr),
                    tokenize_cmd(&split_txt[1]),
                ) {
                    Ok(SetPhraseResult::BufferedAdditional) => {
                        "Keep Phrase as being unified phrase!".to_string()
                    }
                    Ok(SetPhraseResult::Applied) => "Set Phrase!".to_string(),
                    Err(error) => cmd_error_to_text(&CmdError::Phrase(error)),
                }
            } else if len == 2 {
                let ltr = split_txt[0].chars().nth(1).unwrap_or('x');
                let vari = ltr.to_digit(10).unwrap_or(0);
                if ltr == 'c' {
                    self.dtstk.set_cluster_memory(split_txt[1].to_string());
                    "Set a cluster memory!".to_string()
                } else if vari > 0 {
                    match self.put_phrase(
                        self.input_part,
                        PhraseAs::Variation(vari as usize),
                        tokenize_cmd(&split_txt[1]),
                    ) {
                        Ok(SetPhraseResult::BufferedAdditional) => {
                            "Keep Phrase as being unified phrase!".to_string()
                        }
                        Ok(SetPhraseResult::Applied) => "Set Phrase!".to_string(),
                        Err(error) => cmd_error_to_text(&CmdError::Phrase(error)),
                    }
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
    fn letter_dot(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len == 1 {
            if self.during_play {
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
                self.during_play = false;
                "Stopped!".to_string()
            } else {
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_START));
                self.during_play = true;
                "Phrase has started!".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    fn change_current_part(&mut self, part_str: &str) -> Result<String, CmdError> {
        if let Some(pnum) = detect_part(part_str) {
            self.input_part = pnum;
            Ok(match pnum {
                LEFT1 => "Changed current part to left1.".to_string(),
                LEFT2 => "Changed current part to left2.".to_string(),
                RIGHT1 => "Changed current part to right1.".to_string(),
                RIGHT2 => "Changed current part to right2.".to_string(),
                _ => return Err(CmdError::InvalidPart),
            })
        } else {
            Err(CmdError::InvalidPart)
        }
    }
    fn shortcut_input(&mut self, msg_vec: Vec<String>) -> Result<String, CmdError> {
        let part_str = msg_vec[0].as_str();
        let rest_vec = msg_vec[1..].to_vec();
        let first_letter = rest_vec[0].chars().next().unwrap_or('~');
        let mut rtn = Err(CmdError::UnknownCommand);
        match part_str {
            "L1" => rtn = self.apply_shortcut_to_part(LEFT1, first_letter, rest_vec),
            "L2" => rtn = self.apply_shortcut_to_part(LEFT2, first_letter, rest_vec),
            "L" => {
                let _ = self.apply_shortcut_to_part(LEFT1, first_letter, rest_vec.clone());
                rtn = self.apply_shortcut_to_part(LEFT2, first_letter, rest_vec);
            }
            "L1!" => {
                let _ = self.apply_shortcut_to_part(LEFT2, first_letter, rest_vec.clone());
                let _ = self.apply_shortcut_to_part(RIGHT1, first_letter, rest_vec.clone());
                rtn = self.apply_shortcut_to_part(RIGHT2, first_letter, rest_vec);
            }
            "L2!" => {
                let _ = self.apply_shortcut_to_part(LEFT1, first_letter, rest_vec.clone());
                let _ = self.apply_shortcut_to_part(RIGHT1, first_letter, rest_vec.clone());
                rtn = self.apply_shortcut_to_part(RIGHT2, first_letter, rest_vec);
            }
            "R1" => rtn = self.apply_shortcut_to_part(RIGHT1, first_letter, rest_vec),
            "R2" => rtn = self.apply_shortcut_to_part(RIGHT2, first_letter, rest_vec),
            "R" => {
                let _ = self.apply_shortcut_to_part(RIGHT1, first_letter, rest_vec.clone());
                rtn = self.apply_shortcut_to_part(RIGHT2, first_letter, rest_vec);
            }
            "R1!" => {
                let _ = self.apply_shortcut_to_part(LEFT1, first_letter, rest_vec.clone());
                let _ = self.apply_shortcut_to_part(LEFT2, first_letter, rest_vec.clone());
                rtn = self.apply_shortcut_to_part(RIGHT2, first_letter, rest_vec);
            }
            "R2!" => {
                let _ = self.apply_shortcut_to_part(LEFT1, first_letter, rest_vec.clone());
                let _ = self.apply_shortcut_to_part(LEFT2, first_letter, rest_vec.clone());
                rtn = self.apply_shortcut_to_part(RIGHT1, first_letter, rest_vec);
            }
            "FLOW" => {
                rtn = Ok(self.flow_part_command(msg_vec));
            }
            "D" | "DAMPER" => {
                if first_letter == '[' {
                    rtn = self.apply_phrase_to_part(DAMPER_PART, rest_vec);
                }
            }
            "SO" | "SOSTENUTO" => {
                if first_letter == '[' {
                    rtn = self.apply_phrase_to_part(SOSTENUTO_PART, rest_vec);
                }
            }
            "SH" | "SHIFT" => {
                if first_letter == '[' {
                    rtn = self.apply_phrase_to_part(SHIFT_PART, rest_vec);
                }
            }
            "ALL" => {
                for i in 0..MAX_KBD_PART {
                    rtn = self.apply_shortcut_to_part(i, first_letter, rest_vec.clone());
                }
            }
            _ => println!("No Part!"),
        }
        rtn
    }
    fn flow_part_command(
        &mut self,
        //first_letter: &str,
        //input_text: &str,
        msg_vec: Vec<String>,
    ) -> String {
        if &msg_vec[1][0..1] == "{" {
            self.apply_composition_to_part(FLOW_PART, msg_vec[1..].to_vec())
                .unwrap_or_else(|error| cmd_error_to_text(&error))
        } else if msg_vec[1] == "dyn" {
            let dyntxt = extract_texts_from_parentheses(&msg_vec[1]);
            let vel = if dyntxt.is_empty() {
                0
            } else {
                convert_exp2vel(dyntxt) as i16
            };
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_FLOW_VELOCITY, vel]));
            "Flow Velocity Changed!".to_string()
        } else if msg_vec[1] == "static" {
            let chord_txt = extract_texts_from_parentheses(&msg_vec[1]);
            let (_root, table) = convert_chord_to_num(chord_txt.to_string());
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_FLOW_STATIC_SCALE, table]));
            "Flow Static Scale Changed!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn apply_shortcut_to_part(
        &mut self,
        part_num: usize,
        first_letter: char,
        msg_vec: Vec<String>,
    ) -> Result<String, CmdError> {
        match first_letter {
            '[' => self.apply_phrase_to_part(part_num, msg_vec),
            '{' => self.apply_composition_to_part(part_num, msg_vec),
            _ => Err(CmdError::UnknownCommand),
        }
    }
    fn apply_phrase_to_part(
        &mut self,
        part_num: usize,
        msg_vec: Vec<String>,
    ) -> Result<String, CmdError> {
        match self.put_phrase(part_num, PhraseAs::Normal, msg_vec) {
            Ok(SetPhraseResult::BufferedAdditional) => {
                Ok("Keep Phrase as being unified phrase!".to_string())
            }
            Ok(SetPhraseResult::Applied) => Ok("Set Phrase!".to_string()),
            Err(error) => Err(CmdError::Phrase(error)),
        }
    }
    fn apply_composition_to_part(
        &mut self,
        part_num: usize,
        msg_vec: Vec<String>,
    ) -> Result<String, CmdError> {
        self.dtstk
            .set_raw_composition(part_num, msg_vec)
            .map_err(CmdError::Composition)?;
        self.sndr.send_composition_to_elapse(part_num, &self.dtstk);
        Ok("Set Composition!".to_string())
    }
    fn put_phrase(
        &mut self,
        part_num: usize,
        vari: PhraseAs,
        msg_vec: Vec<String>,
    ) -> Result<SetPhraseResult, PhraseCmdError> {
        match self.dtstk.set_raw_phrase(part_num, vari.clone(), msg_vec)? {
            SetPhraseResult::BufferedAdditional => {
                // additional なので、elapse にはまだ送らない
                Ok(SetPhraseResult::BufferedAdditional)
            }
            SetPhraseResult::Applied => {
                if part_num < MAX_KBD_PART {
                    self.sndr.send_phrase_to_elapse(part_num, vari, &self.dtstk);
                } else if (DAMPER_PART..=SHIFT_PART).contains(&part_num) {
                    self.sndr.send_pedal_to_elapse(part_num, &self.dtstk);
                }
                Ok(SetPhraseResult::Applied)
            }
        }
    }
    fn clear_part(&mut self, part_num: usize) {
        // seq stock のデータを消去
        self.dtstk.del_raw_phrase(part_num);

        // Phrase を消去する message を送る
        self.sndr.clear_phrase_to_elapse(part_num);

        if self
            .dtstk
            .set_raw_composition(part_num, vec!["{}".to_string()])
            .is_ok()
        {
            self.sndr.send_composition_to_elapse(part_num, &self.dtstk);
        }
        self.dtstk.change_oct(0, true, part_num);
    }
    fn apply_rit(&self, input_text: &str) -> String {
        let mut aft_rit: i16 = MSG2_RIT_ATMP;
        let mut strength_value: i16 = MSG_RIT_NRM;
        let mut bar_num: i16 = 0;
        let mut rit_txt = split_by('.', input_text[4..].to_string());

        while !rit_txt.is_empty() {
            if rit_txt[0].chars().any(|x| x == '(') {
                if let Some((cmd, prm)) = separate_cmnd_and_str(&rit_txt[0]) {
                    if cmd == "bar" {
                        bar_num = prm.parse::<i16>().unwrap_or(0);
                        if bar_num >= 1 {
                            // 入力値は、内部値より1大きい
                            bar_num -= 1;
                        }
                    } else if cmd == "bpm" {
                        if let Ok(tmp) = prm.parse::<i16>() {
                            aft_rit = tmp;
                        } else {
                            return "Number is wrong.".to_string();
                        }
                    }
                }
            } else if rit_txt[0] == "molto" {
                strength_value = MSG_RIT_MLT;
            } else if rit_txt[0] == "poco" {
                strength_value = MSG_RIT_POCO;
            } else if rit_txt[0] == "fermata" {
                aft_rit = MSG2_RIT_FERMATA;
            }
            rit_txt.remove(0);
        }

        println!("Rit,strength:{strength_value}, bar:{bar_num}, after:{aft_rit}",);
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Rit([strength_value + bar_num * 10, aft_rit]));

        "rit. has started!".to_string()
    }
}
