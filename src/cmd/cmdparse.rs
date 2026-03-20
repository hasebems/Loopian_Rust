//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;

use super::input_txt::CmndRtn;
use super::send_msg::*;
use super::seq_stock::*;
use super::txt_common::*;
use super::txt2seq_cmps::*;
use crate::graphic::generative_view::{GraphicMsg, generate_graphic_msg};
use crate::lpnlib::*;

pub fn reply_to_cmd(reply: String) -> Option<CmndRtn> {
    Some(CmndRtn(reply, GraphicMsg::NoMsg))
}

//  LoopianCmd の責務
//  1. Command を受信し中身を調査
//  2. 解析に送る/elapseに送る
//  3. guiに返事を返す
pub struct LoopianCmd {
    during_play: bool,
    recursive: bool,
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
            recursive: false,
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
        if input_text.is_empty() || !input_text.is_ascii() {
            // option + space などの無効な文字列
            return None;
        }

        let msg_vec = self.analyze_and_divide_to_msg(input_text);
        println!("Analyze Text: {:?}", msg_vec);
        let first_letter = input_text.chars().next().unwrap();
        if input_text.len() == 1 && first_letter == '.' {
            reply_to_cmd(self.letter_dot(input_text))
        } else if first_letter == '/' {
            reply_to_cmd(self.letter_slash(input_text))
        } else if first_letter == '@' {
            reply_to_cmd(self.letter_at(input_text))
        } else if first_letter == '[' {
            reply_to_cmd(self.letter_bracket(input_text))
        } else if first_letter == '{' {
            reply_to_cmd(self.letter_brace(input_text))
        } else if first_letter == 'L'
            || first_letter == 'R'
            || first_letter == 'F'
            || first_letter == 'A'
            || first_letter == 'D'
            || first_letter == 'S'
        {
            reply_to_cmd(self.letter_part(input_text))
        } else if msg_vec.len() == 1 {
            // one word command
            reply_to_cmd(self.one_word_command(&msg_vec[0]))
        } else {
            match msg_vec[0].as_str() {
                "set" => reply_to_cmd(self.parse_set_command(&msg_vec[1])),
                "sync" => reply_to_cmd(self.cmd_sync(&msg_vec[1])),
                "clear" => reply_to_cmd(self.cmd_clear(&msg_vec[1])),
                "fine" => reply_to_cmd(self.cmd_fine(&msg_vec[1])),
                "help" => reply_to_cmd(self.cmd_help(&msg_vec[1])),
                "graph" => {
                    let rtn = generate_graphic_msg(msg_vec);
                    Some(CmndRtn(rtn.0, rtn.1))
                }
                "effect" => reply_to_cmd(self.cmd_effect(&msg_vec[1])),
                _ => reply_to_cmd("what?".to_string()),
            }
        }
    }
    fn analyze_and_divide_to_msg(&self, input_text: &str) -> Vec<String> {
        // コマンドを解析して、Elapse に送るべきメッセージのベクタを返す
        // 例えば、"set.key(C)" なら、["set", "key(C)"] を返す
        // "L1.{CDE}" なら、["L1", "{CDE}"] を返す
        // (),{},[] を考慮して、ピリオドで分割する
        let mut msg_vec: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut paren_depth = 0i32;
        let mut brace_depth = 0i32;
        let mut bracket_depth = 0i32;

        for ch in input_text.chars() {
            match ch {
                '(' => {
                    paren_depth += 1;
                    current.push(ch);
                }
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                    current.push(ch);
                }
                '{' => {
                    brace_depth += 1;
                    current.push(ch);
                }
                '}' => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    current.push(ch);
                }
                '[' => {
                    bracket_depth += 1;
                    current.push(ch);
                }
                ']' => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                    current.push(ch);
                }
                '.' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                    if !current.is_empty() {
                        msg_vec.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            msg_vec.push(current);
        }

        msg_vec
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
            if !self.recursive {
                // stop
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
                self.during_play = false;
                // clear
                for i in 0..MAX_KBD_PART {
                    self.clear_part(i);
                }
                self.send_clear();
                "all data erased!".to_string()
            } else if self.recursive {
                self.clear_part(self.input_part);
                "designated part data erased!".to_string()
            } else {
                "what?".to_string()
            }
        } else if let Some(pnum) = Self::detect_part(input_part) {
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
                if let Some(additional) =
                    self.put_phrase(self.input_part, PhraseAs::Measure(msr), &split_txt[1])
                {
                    if additional {
                        "Keep Phrase as being unified phrase!".to_string()
                    } else {
                        "Set Phrase!".to_string()
                    }
                } else {
                    "what?".to_string()
                }
            } else if len == 2 {
                let ltr = split_txt[0].chars().nth(1).unwrap_or('x');
                let vari = ltr.to_digit(10).unwrap_or(0);
                if ltr == 'c' {
                    self.dtstk.set_cluster_memory(split_txt[1].to_string());
                    "Set a cluster memory!".to_string()
                } else if vari > 0 {
                    if let Some(additional) = self.put_phrase(
                        self.input_part,
                        PhraseAs::Variation(vari as usize),
                        &split_txt[1],
                    ) {
                        if additional {
                            "Keep Phrase as being unified phrase!".to_string()
                        } else {
                            "Set Phrase!".to_string()
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
        } else {
            "what?".to_string()
        }
    }
    fn letter_bracket(&mut self, input_text: &str) -> String {
        if let Some(addtional) = self.put_phrase(self.input_part, PhraseAs::Normal, input_text) {
            if addtional {
                "Keep Phrase as being unified phrase!".to_string()
            } else {
                "Set Phrase!".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    fn letter_brace(&mut self, input_text: &str) -> String {
        if self
            .dtstk
            .set_raw_composition(self.input_part, input_text.to_string())
        {
            self.sndr
                .send_composition_to_elapse(self.input_part, &self.dtstk);
            "Set Composition!".to_string()
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
    fn letter_part(&mut self, input_text: &str) -> String {
        if let Some(pnum) = Self::detect_part(input_text) {
            self.input_part = pnum;
            match pnum {
                LEFT1 => "Changed current part to left1.".to_string(),
                LEFT2 => "Changed current part to left2.".to_string(),
                RIGHT1 => "Changed current part to right1.".to_string(),
                RIGHT2 => "Changed current part to right2.".to_string(),
                _ => "what?".to_string(),
            }
        } else {
            self.shortcut_input(input_text)
        }
    }
    fn shortcut_input(&mut self, input_text: &str) -> String {
        // shortcut input
        let mut rtn_str = "what?".to_string();
        for (i, ltr) in input_text.char_indices() {
            if ltr == '.' {
                let first_letter = &input_text[i + 1..i + 2]; // '{' '['
                let part_str = &input_text[0..i];
                let rest_text = &input_text[i + 1..];
                match part_str {
                    "L1" => rtn_str = self.call_bracket_brace(LEFT1, first_letter, rest_text),
                    "L2" => rtn_str = self.call_bracket_brace(LEFT2, first_letter, rest_text),
                    "L" => {
                        self.call_bracket_brace(LEFT1, first_letter, rest_text);
                        rtn_str = self.call_bracket_brace(LEFT2, first_letter, rest_text);
                    }
                    "L1!" => {
                        self.call_bracket_brace(LEFT2, first_letter, rest_text);
                        self.call_bracket_brace(RIGHT1, first_letter, rest_text);
                        rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text);
                    }
                    "L2!" => {
                        self.call_bracket_brace(LEFT1, first_letter, rest_text);
                        self.call_bracket_brace(RIGHT1, first_letter, rest_text);
                        rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text);
                    }
                    "R1" => rtn_str = self.call_bracket_brace(RIGHT1, first_letter, rest_text),
                    "R2" => rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text),
                    "R" => {
                        self.call_bracket_brace(RIGHT1, first_letter, rest_text);
                        rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text);
                    }
                    "R1!" => {
                        self.call_bracket_brace(LEFT1, first_letter, rest_text);
                        self.call_bracket_brace(LEFT2, first_letter, rest_text);
                        rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text);
                    }
                    "R2!" => {
                        self.call_bracket_brace(LEFT1, first_letter, rest_text);
                        self.call_bracket_brace(LEFT2, first_letter, rest_text);
                        rtn_str = self.call_bracket_brace(RIGHT1, first_letter, rest_text);
                    }
                    "FLOW" => {
                        rtn_str = self.flow_part_command(first_letter, rest_text);
                    }
                    "D" | "DAMPER" => {
                        if first_letter == "[" {
                            rtn_str = self.call_bracket_brace(DAMPER_PART, first_letter, rest_text);
                        }
                    }
                    "SO" | "SOSTENUTO" => {
                        if first_letter == "[" {
                            rtn_str =
                                self.call_bracket_brace(SOSTENUTO_PART, first_letter, rest_text);
                        }
                    }
                    "SH" | "SHIFT" => {
                        if first_letter == "[" {
                            rtn_str = self.call_bracket_brace(SHIFT_PART, first_letter, rest_text);
                        }
                    }
                    "ALL" => {
                        for i in 0..MAX_KBD_PART {
                            rtn_str = self.call_bracket_brace(i, first_letter, rest_text);
                        }
                    }
                    _ => println!("No Part!"),
                }
                break;
            }
        }
        rtn_str
    }
    fn flow_part_command(&mut self, first_letter: &str, input_text: &str) -> String {
        let len = input_text.chars().count();
        if first_letter == "{" {
            self.call_bracket_brace(FLOW_PART, first_letter, input_text)
        } else if len >= 3 && &input_text[0..3] == "dyn" {
            let dyntxt = extract_texts_from_parentheses(input_text);
            let vel = if dyntxt.is_empty() {
                0
            } else {
                convert_exp2vel(dyntxt) as i16
            };
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_FLOW_VELOCITY, vel]));
            "Flow Velocity Changed!".to_string()
        } else if len >= 6 && &input_text[0..6] == "static" {
            let chord_txt = extract_texts_from_parentheses(input_text);
            let (_root, table) = convert_chord_to_num(chord_txt.to_string());
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set([MSG_SET_FLOW_STATIC_SCALE, table]));
            "Flow Static Scale Changed!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn detect_part(part_str: &str) -> Option<usize> {
        let len = part_str.chars().count();
        if len == 5 {
            let pt = &part_str[0..5];
            if pt == "left1" {
                Some(LEFT1)
            } else if pt == "left2" {
                Some(LEFT2)
            } else {
                None
            }
        } else if len == 6 {
            let pt = &part_str[0..6];
            if pt == "right1" {
                Some(RIGHT1)
            } else if pt == "right2" {
                Some(RIGHT2)
            } else {
                None
            }
        } else if len == 2 {
            let pt = &part_str[0..2];
            if pt == "L1" {
                Some(LEFT1)
            } else if pt == "L2" {
                Some(LEFT2)
            } else if pt == "R1" {
                Some(RIGHT1)
            } else if pt == "R2" {
                Some(RIGHT2)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn call_bracket_brace(
        &mut self,
        part_num: usize,
        _first_letter: &str,
        rest_text: &str,
    ) -> String {
        let mut input_text = rest_text;
        let itx: String;
        if let Some(rs) = self
            .dtstk
            .check_if_additional_phrase(input_text.to_string())
        {
            itx = rs;
            input_text = &itx;
        } else {
            return "Invalid Syntax!".to_string();
        }

        let mut rtn_str = "what?".to_string();
        let org_part = self.input_part;
        self.recursive = true;
        self.input_part = part_num;
        if let Some(ans) = self.dup_bracket_brace(input_text) {
            rtn_str = ans.0;
        }
        self.input_part = org_part;
        self.recursive = false;
        rtn_str
    }
    fn dup_bracket_brace(&mut self, input_text: &str) -> Option<CmndRtn> {
        let first_letter = &input_text[0..1];
        if first_letter == "[" {
            reply_to_cmd(self.letter_bracket(input_text))
        } else if first_letter == "{" {
            reply_to_cmd(self.letter_brace(input_text))
        } else {
            None
        }
    }
    fn put_phrase(&mut self, part_num: usize, vari: PhraseAs, input_text: &str) -> Option<bool> {
        if let Some(additional) =
            self.dtstk
                .set_raw_phrase(part_num, vari.clone(), input_text.to_string())
        {
            if additional {
                // additional なので、elapse にはまだ送らない
                Some(true)
            } else {
                if part_num < MAX_KBD_PART {
                    self.sndr.send_phrase_to_elapse(part_num, vari, &self.dtstk);
                } else if (DAMPER_PART..=SHIFT_PART).contains(&part_num) {
                    self.sndr.send_pedal_to_elapse(part_num, &self.dtstk);
                }
                Some(false)
            }
        } else {
            None
        }
    }
    fn clear_part(&mut self, part_num: usize) {
        // seq stock のデータを消去
        self.dtstk.del_raw_phrase(part_num);

        // Phrase を消去する message を送る
        self.sndr.clear_phrase_to_elapse(part_num);

        if self.dtstk.set_raw_composition(part_num, "{}".to_string()) {
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
