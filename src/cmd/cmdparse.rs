//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;

use super::send_msg::*;
use super::seq_stock::*;
use super::txt_common::*;
use crate::lpnlib::*;

//  LoopianCmd の責務
//  1. Command を受信し中身を調査
//  2. 解析に送る/elapseに送る
//  3. eguiに返事を返す
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
    #[cfg(feature = "raspi")]
    pub fn send_reconnect(&self) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_MIDI_RECONNECT));
    }
    //*************************************************************************
    pub fn set_and_responce(&mut self, input_text: &str) -> Option<CmndRtn> {
        if input_text.is_empty() {
            return None;
        }
        println!("Set Text: {}", input_text);
        let first_letter = &input_text[0..1];
        if first_letter == "@" {
            Some(CmndRtn(self.letter_at(input_text), NO_MSG))
        } else if first_letter == "[" {
            Some(CmndRtn(self.letter_bracket(input_text), NO_MSG))
        } else if first_letter == "{" {
            Some(CmndRtn(self.letter_brace(input_text), NO_MSG))
        } else if first_letter == "." {
            Some(CmndRtn(self.letter_dot(input_text), NO_MSG))
        } else if first_letter == "c" {
            Some(CmndRtn(self.letter_c(input_text), NO_MSG))
        } else if first_letter == "e" {
            Some(CmndRtn(self.letter_e(input_text), NO_MSG))
        } else if first_letter == "f" {
            Some(CmndRtn(self.letter_f(input_text), NO_MSG))
        } else if first_letter == "g" {
            Some(self.letter_g(input_text))
        } else if first_letter == "l" {
            Some(CmndRtn(self.letter_l(input_text), NO_MSG))
        } else if first_letter == "p" {
            Some(CmndRtn(self.letter_p(input_text), NO_MSG))
        } else if first_letter == "r" {
            Some(CmndRtn(self.letter_r(input_text), NO_MSG))
        } else if first_letter == "s" {
            Some(CmndRtn(self.letter_s(input_text), NO_MSG))
        } else if first_letter == "L"
            || first_letter == "R"
            || first_letter == "F"
            || first_letter == "A"
        {
            Some(CmndRtn(self.letter_part(input_text), NO_MSG))
        } else {
            Some(CmndRtn("what?".to_string(), NO_MSG))
        }
    }
    fn letter_c(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len >= 5 && &input_text[0..5] == "clear" {
            if !self.recursive && len == 5 {
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
                let part_letter = &input_text[6..];
                println!("clear>>{}", part_letter);
                if let Some(pnum) = Self::detect_part(part_letter) {
                    self.clear_part(pnum);
                    match pnum {
                        LEFT1 => "part L1 data erased!".to_string(),
                        LEFT2 => "part L2 data erased!".to_string(),
                        RIGHT1 => "part R1 data erased!".to_string(),
                        RIGHT2 => "part R2 data erased!".to_string(),
                        _ => "some part data erased!".to_string(),
                    }
                } else {
                    "what?".to_string()
                }
            }
        } else {
            "what?".to_string()
        }
    }
    fn letter_e(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len == 3 && &input_text[0..3] == "end" {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            self.during_play = false;
            "Fine.".to_string()
        } else if len >= 5 && &input_text[0..5] == "efct." {
            let efct = &input_text[5..];
            if efct.contains("dmp(") {
                if let Some(dmp) = extract_number_from_parentheses(efct) {
                    self.sndr
                        .send_msg_to_elapse(ElpsMsg::Efct([MSG_EFCT_DMP, dmp as i16]));
                    format!("Set Damper Value: {}", dmp)
                } else {
                    "No Value!".to_string()
                }
            } else if efct.contains("cc70(") {
                if let Some(cc70) = extract_number_from_parentheses(efct) {
                    self.sndr
                        .send_msg_to_elapse(ElpsMsg::Efct([MSG_EFCT_CC70, cc70 as i16]));
                    format!("Set CC70 Value: {}", cc70)
                } else {
                    "No Value!".to_string()
                }
            } else {
                "what?".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    fn letter_f(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "fine" {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            self.during_play = false;
            "Fine.".to_string()
        } else if len == 7 && &input_text[0..7] == "fermata" {
            // fermata
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Rit([MSG_RIT_NRM, MSG2_RIT_FERMATA]));
            "Will stop!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_g(&mut self, input_text: &str) -> CmndRtn {
        let len = input_text.chars().count();
        if len >= 6 && &input_text[0..5] == "graph" {
            if len == 11 && &input_text[6..11] == "light" {
                CmndRtn("Changed Graphic!".to_string(), LIGHT_MODE)
            } else if len == 10 && &input_text[6..10] == "dark" {
                CmndRtn("Changed Graphic!".to_string(), DARK_MODE)
            } else if len == 12 && &input_text[6..12] == "ripple" {
                CmndRtn("Changed Graphic Note Pattern!".to_string(), RIPPLE_PATTERN)
            } else if len == 11 && &input_text[6..11] == "voice" {
                CmndRtn("Changed Graphic Note Pattern!".to_string(), VOICE_PATTERN)
            } else if len == 11 && &input_text[6..11] == "lissa" {
                CmndRtn(
                    "Changed Graphic Note Pattern!".to_string(),
                    LISSAJOUS_PATTERN,
                )
            } else {
                CmndRtn("what?".to_string(), 0)
            }
        } else {
            CmndRtn("what?".to_string(), 0)
        }
    }
    fn letter_l(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len == 5 && &input_text[0..5] == "left1" {
            self.input_part = LEFT1;
            "Changed current part to left1.".to_string()
        } else if len == 5 && &input_text[0..5] == "left2" {
            self.input_part = LEFT2;
            "Changed current part to left2.".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_p(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if (len == 4 && &input_text[0..4] == "play") || (len == 1 && &input_text[0..1] == "p") {
            if !self.during_play {
                // play
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_START));
                self.during_play = true;
                "Phrase has started!".to_string()
            } else {
                "Playing now!".to_string()
            }
        } else if len == 5 && &input_text[0..5] == "panic" {
            // panic
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_PANIC));
            "All Sound Off!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_r(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len >= 6 && &input_text[0..6] == "resume" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_RESUME));
            self.during_play = true;
            "Resume.".to_string()
        } else if len >= 6 && &input_text[0..6] == "right1" {
            self.input_part = RIGHT1;
            "Changed current part to right1.".to_string()
        } else if len >= 6 && &input_text[0..6] == "right2" {
            self.input_part = RIGHT2;
            "Changed current part to right2.".to_string()
        } else if len >= 4 && &input_text[0..4] == "rit." {
            self.set_rit(input_text)
        } else if len >= 9 && &input_text[0..9] == "reconnect" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_MIDI_RECONNECT));
            "Send reconnect".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_s(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "stop" {
            if self.during_play {
                // stop
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
                self.during_play = false;
                "Stopped!".to_string()
            } else {
                "Settle down!".to_string()
            }
        } else if len >= 4 && &input_text[0..4] == "set." {
            // set
            self.parse_set_command(input_text)
        } else if len >= 4 && &input_text[0..4] == "sync" {
            if len == 4 {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Sync(self.input_part as i16));
                "Synchronized!".to_string()
            } else if len >= 6 {
                let prttxt = &input_text[5..];
                if prttxt == "right" {
                    self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_RGT));
                    "Right Part Synchronized!".to_string()
                } else if prttxt == "left" {
                    self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_LFT));
                    "Left Part Synchronized!".to_string()
                } else if prttxt == "all" {
                    self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_ALL));
                    "All Part Synchronized!".to_string()
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
                    self.set_phrase(self.input_part, PhraseAs::Measure(msr), &split_txt[1])
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
                    return "Set a cluster memory!".to_string();
                } else if vari > 0 {
                    if let Some(additional) = self.set_phrase(
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
        if let Some(addtional) = self.set_phrase(self.input_part, PhraseAs::Normal, input_text) {
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
        for (i, ltr) in input_text.chars().enumerate() {
            if ltr == '.' {
                let first_letter = &input_text[i + 1..i + 2]; // '{' '['
                let part_str = &input_text[0..i];
                let rest_text = &input_text[i + 1..];
                match part_str {
                    "L1" => rtn_str = self.call_bracket_brace(LEFT1, first_letter, rest_text),
                    "L2" => rtn_str = self.call_bracket_brace(LEFT2, first_letter, rest_text),
                    "L12" => {
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
                    "R12" => {
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
                        if first_letter == "{" {
                            rtn_str = self.call_bracket_brace(FLOW_PART, first_letter, rest_text);
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
        if let Some(ans) = self.set_and_responce(input_text) {
            rtn_str = ans.0;
        }
        self.input_part = org_part;
        self.recursive = false;

        /*if first_letter == "[" {
            if self.set_phrase(part_num, 0, rest_text) {
                rtn_str = "Set Phrase!".to_string();
            }
        }
        else if first_letter == "{" {
            if self.dtstk.set_raw_composition(part_num, rest_text.to_string()) {
                self.send_composition_to_elapse(part_num);
                rtn_str = "Set Composition!".to_string();
            }
        }*/
        rtn_str
    }
    fn set_phrase(&mut self, part_num: usize, vari: PhraseAs, input_text: &str) -> Option<bool> {
        if let Some(additional) =
            self.dtstk
                .set_raw_phrase(part_num, vari.clone(), input_text.to_string())
        {
            if additional {
                // additional なので、elapse にはまだ送らない
                Some(true)
            } else {
                self.sndr.send_phrase_to_elapse(part_num, vari, &self.dtstk);
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

        let empty_cmp = "{}".to_string();
        if self.dtstk.set_raw_composition(part_num, empty_cmp) {
            self.sndr.send_composition_to_elapse(part_num, &self.dtstk);
        }
        self.dtstk.change_oct(0, true, part_num);
    }
    fn set_rit(&self, input_text: &str) -> String {
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
                        if prm == "fermata" {
                            aft_rit = MSG2_RIT_FERMATA;
                        } else if let Ok(tmp) = prm.parse::<i16>() {
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
            }
            rit_txt.remove(0);
        }

        println!(
            "Rit,strength:{}, bar:{}, after:{}",
            strength_value, bar_num, aft_rit
        );
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Rit([strength_value + bar_num * 10, aft_rit]));

        "rit. has started!".to_string()
    }
}
