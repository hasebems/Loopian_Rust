//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use std::sync::{mpsc, mpsc::*};
use crate::graphic::graphic;
use super::seq_stock::*;
use super::send_msg::*;

//  LoopianCmd の責務
//  1. Command を受信し中身を調査
//  2. 解析に送る/elapseに送る
//  3. eguiに返事を返す
pub struct LoopianCmd {
    indicator: Vec<String>,
    indicator_key_stock: String,
    ui_hndr: mpsc::Receiver<String>,
    input_part: usize,
    dtstk: SeqDataStock,
    graphic_ev: Vec<String>,
    graphic_msg: i16,
    sndr: MessageSender,
}

impl LoopianCmd {
    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>, ui_hndr: mpsc::Receiver<String>) -> Self {
        let mut indicator = vec![String::from("---"); graphic::MAX_INDICATOR];
        indicator[0] = "C".to_string();
        indicator[1] = DEFAULT_BPM.to_string();
        indicator[3] = "1 : 1 : 000".to_string();
        Self {
            indicator,
            indicator_key_stock: "C".to_string(),
            ui_hndr,
            input_part: RIGHT1,
            dtstk: SeqDataStock::new(),
            graphic_ev: Vec::new(),
            graphic_msg: NO_MSG,
            sndr: MessageSender::new(msg_hndr),
        }
    }
    pub fn move_ev_from_gev(&mut self) -> Option<String> {
        if self.graphic_ev.len() > 0 {
            let gev = self.graphic_ev[0].clone();
            self.graphic_ev.remove(0);
            return Some(gev);
        }
        else {None}
    }
    pub fn get_input_part(&self) -> usize {self.input_part}
    pub fn get_part_txt(&self) -> &str {
        match self.input_part {
            LEFT1 => "L1",            
            LEFT2 => "L2",
            RIGHT1 => "R1",
            RIGHT2 => "R2",
            _ => "__",
        }
    }
    pub fn get_indicator(&self, num: usize) -> &str {
        &self.indicator[num]
    }
    pub fn get_graphic_msg(&self) -> i16 {self.graphic_msg}
    pub fn read_from_ui_hndr(&mut self) {
        // Play Thread からの、8indicator表示用メッセージを受信する処理
        loop {
            match self.ui_hndr.try_recv() {
                Ok(mut uitxt)  => {
                    if let Some(letter) = uitxt.chars().nth(0) {
                        let ind_num: usize = letter.to_digit(10).unwrap() as usize;
                        let len = uitxt.chars().count();
                        if len >= 2 {
                            let txt = uitxt.split_off(1);
                            if ind_num == 0 && txt == "_" {
                                self.indicator[0] = self.indicator_key_stock.clone();
                            }
                            else if ind_num < graphic::MAX_INDICATOR {
                                self.indicator[ind_num] = txt;
                            }
                            else if ind_num == 9 {
                                // ElapseStack からの発音 Note Number/Velocity
                                self.graphic_ev.push(txt);
                            }
                        }
                    }
                },
                Err(TryRecvError::Disconnected) => break,// Wrong!
                Err(TryRecvError::Empty) => break,
            }
        }
    }
    //*************************************************************************
    pub fn set_and_responce(&mut self, input_text: &str) -> Option<String> {
        println!("Set Text: {}",input_text);
        let first_letter = &input_text[0..1];
        if first_letter == "q" {
            if &input_text[0..4] == "quit" {
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_QUIT));
                let option = input_text[4..].to_string();
                if option.trim() == "nosave" {
                    Some("nosave".to_string())
                }
                else {None} //  The End of the App
            }
            else {Some("what?".to_string())}
        }
        else if first_letter == "@" {self.letter_at(input_text)}
        else if first_letter == "[" {self.letter_bracket(input_text)}
        else if first_letter == "{" {self.letter_brace(input_text)}
        else if first_letter == "c" {self.letter_c(input_text)}
        else if first_letter == "e" {self.letter_e(input_text)}
        else if first_letter == "f" {self.letter_f(input_text)}
        else if first_letter == "g" {self.letter_g(input_text)}
        else if first_letter == "l" {self.letter_l(input_text)}
        else if first_letter == "p" {self.letter_p(input_text)}
        else if first_letter == "r" {self.letter_r(input_text)}
        else if first_letter == "s" {self.letter_s(input_text)}
        else if first_letter == "L" {self.letter_part(input_text)}
        else if first_letter == "R" {self.letter_part(input_text)}
        else if first_letter == "A" {self.letter_part(input_text)}
        else                        {Some("what?".to_string())}
    }
    fn letter_c(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 5 && &input_text[0..5] == "clear" {
            if len == 5 {
                // stop
                self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
                // clear
                for i in 0..MAX_USER_PART {
                    self.clear_part(i);
                }
                Some("all data erased!".to_string())
            }
            else {
                let part_letter = &input_text[6..];
                println!("clear>>{}",part_letter);
                let pnum = Self::detect_part(part_letter);
                if pnum != MAX_USER_PART {self.clear_part(pnum);}
                match pnum {
                    LEFT1  => Some("part L1 data erased!".to_string()),
                    LEFT2  => Some("part L2 data erased!".to_string()),
                    RIGHT1 => Some("part R1 data erased!".to_string()),
                    RIGHT2 => Some("part R2 data erased!".to_string()),
                    _ => Some("what?".to_string()),
                }
            }
        }
        else if len >= 2 && &input_text[0..2] == "c=" {
            self.dtstk.set_cluster_memory(input_text[2..].to_string());
            Some("Set a cluster memory!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_e(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len == 3 && &input_text[0..3] == "end" {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            Some("Fine.".to_string())
        } else if len == 7 && &input_text[0..7] == "endflow" {
            // fermata
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_ENDFLOW+self.input_part as i16));
            Some("End MIDI in flow!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_f(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "fine" {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            Some("Fine.".to_string())
        } else if len == 7 && &input_text[0..7] == "fermata" {
            // fermata
            self.sndr.send_msg_to_elapse(ElpsMsg::Rit([MSG_RIT_NRM, MSG2_RIT_FERMATA]));
            Some("Will be longer!".to_string())
        } else if len == 4 && &input_text[0..4] == "flow" {
            // flow
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_FLOW+self.input_part as i16));
            let res = format!("MIDI in flows on Part {}!", self.get_part_txt());
            Some(res.to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_g(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 6 && &input_text[0..6] == "graph " {
            if len >= 11 && &input_text[6..11] == "light" {
                self.graphic_msg = LIGHT_MODE;
                Some("Changed Graphic!".to_string())
            }
            else if len >= 10 && &input_text[6..10] == "dark" {
                self.graphic_msg = DARK_MODE;
                Some("Changed Graphic!".to_string())
            }
            else {
                Some("what?".to_string())
            }
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_l(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len == 5 && &input_text[0..5] == "left1" {
            self.input_part = LEFT1;
            Some("Changed current part to left1.".to_string())
        } else if len == 5 && &input_text[0..5] == "left2" {
            self.input_part = LEFT2;
            Some("Changed current part to left2.".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_p(&self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if (len == 4 && &input_text[0..4] == "play") || (len == 1 && &input_text[0..1] == "p") {
            // play
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_START));
            Some("Phrase has started!".to_string())
        } else if len == 5 && &input_text[0..5] == "panic" {
            // panic
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_PANIC));
            Some("All Sound Off!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_r(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 6 && &input_text[0..6] == "resume" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_RESUME));
            Some("Resume.".to_string())
        } else if len >= 6 && &input_text[0..6] == "right1" {
            self.input_part = RIGHT1;
            Some("Changed current part to right1.".to_string())
        } else if len >= 6 && &input_text[0..6] == "right2" {
            self.input_part = RIGHT2;
            Some("Changed current part to right2.".to_string())
        } else if len >= 4 && &input_text[0..4] == "rit." {
            let strength_txt: String;
            let mut aft_rit: i16 = MSG2_RIT_ATMP;
            let mut strength_value: i16 = MSG_RIT_NRM;
            if input_text.chars().any(|x| x=='/') {
                let rit_txt_raw = &input_text[4..];
                let rit_txt = split_by('/', rit_txt_raw.to_string());
                let nxt_msr_txt = &rit_txt[1];
                if nxt_msr_txt == "fermata" {aft_rit = MSG2_RIT_FERMATA;}
                else {
                    match nxt_msr_txt.parse::<i16>() {
                        Ok(tmp) => aft_rit = tmp,
                        Err(e) => {
                            println!("{:?}",e);
                            "Number is wrong.".to_string();
                        },
                    }
                }
                strength_txt = rit_txt[0].clone();
            }
            else {
                strength_txt = input_text[4..].to_string();
            }
            if strength_txt == "poco" {strength_value = MSG_RIT_POCO;}
            else if strength_txt == "molto" {strength_value = MSG_RIT_MLT;}
            self.sndr.send_msg_to_elapse(ElpsMsg::Rit([strength_value, aft_rit]));
            Some("rit. has started!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_s(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "stop" {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MSG_CTRL_STOP));
            Some("Stopped!".to_string())
        } else if len >= 3 && &input_text[0..3] == "set" {
            // set
            let responce = self.parse_set_command(input_text);
            Some(responce)
        } else if len >= 4 && &input_text[0..4] == "sync" {
            // sync
            let vectxt = input_text.split(' ')
                .fold(Vec::new(), |mut s, i| {
                    s.push(i.to_string());
                    s
                });
            if vectxt.len() < 2 {
                self.sndr.send_msg_to_elapse(ElpsMsg::Sync(self.input_part as i16));
                Some("Synchronized!".to_string())
            } else if vectxt[1] == "right" {
                self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_RGT));
                Some("Right Part Synchronized!".to_string())
            } else if vectxt[1] == "left" {
                self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_LFT));
                Some("Left Part Synchronized!".to_string())
            } else if vectxt[1] == "all" {
                self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MSG_SYNC_ALL));
                Some("All Part Synchronized!".to_string())
            } else {
                Some("what?".to_string())
            }
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_at(&mut self, input_text: &str) -> Option<String> {
        if let Some(ltr) = input_text.chars().nth(1) {
            if let Some(vari) = ltr.to_digit(10) {
                let itxt = input_text.trim();
                if let Some(ltr2) = itxt.chars().nth(2) {
                    if ltr2 == '=' {    // @n= 
                        let brachet_text = &itxt[3..];
                        if self.set_phrase(self.input_part, vari as usize, brachet_text) {
                            return Some("Set Phrase!".to_string())
                        }
                    }
                }
            }
        }
        Some("what?".to_string())
    }
    fn letter_bracket(&mut self, input_text: &str) -> Option<String> {
        if self.set_phrase(self.input_part, 0, input_text) {
            Some("Set Phrase!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_brace(&mut self, input_text: &str) -> Option<String> {
        if self.dtstk.set_raw_composition(self.input_part, input_text.to_string()) {
            self.sndr.send_composition_to_elapse(self.input_part, &self.dtstk);
            Some("Set Composition!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_part(&mut self, input_text: &str) -> Option<String> {
        let pnum = Self::detect_part(input_text);
        if pnum != MAX_USER_PART {self.input_part = pnum;}
        match pnum {
            LEFT1  => Some("Changed current part to left1.".to_string()),
            LEFT2  => Some("Changed current part to left2.".to_string()),
            RIGHT1 => Some("Changed current part to right1.".to_string()),
            RIGHT2 => Some("Changed current part to right2.".to_string()),
            _ => self.shortcut_input(input_text),
        }
    }
    fn shortcut_input(&mut self, input_text: &str) -> Option<String> {
        // shortcut input
        let mut rtn_str = "what?".to_string();
        for (i, ltr) in input_text.chars().enumerate() {
            if ltr == '>' {
                let first_letter = &input_text[i+1..i+2];
                let part_str = &input_text[0..i];
                let rest_text = &input_text[i+1..];
                match part_str {
                    "L1" => rtn_str = self.call_bracket_brace(LEFT1, first_letter, rest_text),
                    "L2" => rtn_str = self.call_bracket_brace(LEFT2, first_letter, rest_text),
                    "L12" => {
                        rtn_str = self.call_bracket_brace(LEFT1, first_letter, rest_text);
                        if rtn_str != "what?" {
                            rtn_str = self.call_bracket_brace(LEFT2, first_letter, rest_text);
                        }
                    },
                    "R1" => rtn_str = self.call_bracket_brace(RIGHT1, first_letter, rest_text),
                    "R2" => rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text),
                    "R12" => {
                        rtn_str = self.call_bracket_brace(RIGHT1, first_letter, rest_text);
                        if rtn_str != "what?" {
                            rtn_str = self.call_bracket_brace(RIGHT2, first_letter, rest_text);
                        }
                    },
                    "ALL" => {
                        for i in 0..MAX_USER_PART {
                            rtn_str = self.call_bracket_brace(i, first_letter, rest_text);
                        }
                    },
                    _ => println!("No Part!"),
                }
                break;
            }
        }
        Some(rtn_str)
    }
    fn detect_part(part_str: &str) -> usize {
        let len = part_str.chars().count();
        if len == 5 {
            if      &part_str[0..5] == "left1" {LEFT1}
            else if &part_str[0..5] == "left2" {LEFT2}
            else {MAX_USER_PART}
        }
        else if len == 6 {
            if      &part_str[0..6] == "right1" {RIGHT1}
            else if &part_str[0..6] == "right2" {RIGHT2}
            else {MAX_USER_PART}
        }
        else if len == 2 {
            if      &part_str[0..2] == "L1" {LEFT1}
            else if &part_str[0..2] == "L2" {LEFT2}
            else if &part_str[0..2] == "R1" {RIGHT1}
            else if &part_str[0..2] == "R2" {RIGHT2}
            else {MAX_USER_PART}
        }
        else {MAX_USER_PART}
    }
    fn call_bracket_brace(&mut self, part_num: usize, _first_letter: &str, rest_text: &str) -> String {
        let mut rtn_str = "what?".to_string();
        let org_part = self.input_part;
        self.input_part = part_num;
        if let Some(ans) = self.set_and_responce(rest_text) {
            rtn_str = ans;
        }
        self.input_part = org_part;

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
    fn set_phrase(&mut self, part_num: usize, vari: usize, input_text: &str) -> bool {
        if self.dtstk.set_raw_phrase(part_num, vari, input_text.to_string()) {
            self.sndr.send_phrase_to_elapse(part_num, vari, &self.dtstk);
            true
        }
        else {false}
    }
    fn clear_part(&mut self, part_num: usize) {
        for j in 0..MAX_PHRASE {
            let empty_phr = "[]";
            self.set_phrase(part_num, j, empty_phr);
        }
        let empty_cmp = "{}".to_string();
        if self.dtstk.set_raw_composition(part_num, empty_cmp) {
            self.sndr.send_composition_to_elapse(part_num, &self.dtstk);
        }
        self.dtstk.change_oct(0, true, part_num);
    }
    //*************************************************************************
    fn parse_set_command(&mut self, input_text: &str) -> String {
        let cmnd = &input_text[4..];
        let _len = cmnd.chars().count();
        let cv = cmnd.split('=')
            .fold(Vec::new(), |mut s, i| {
                s.push(i.to_string());
                s
            });
        if cv[0] == "key".to_string() {
            if self.change_key(&cv[1]) {
                "Key has changed!".to_string()
            }
            else {
                "what?".to_string()
            }
        }
        else if cv[0] == "oct" {
            if self.change_oct(&cv[1]) {
                "Octave has changed!".to_string()
            }
            else {
                "what?".to_string()
            }
        }
        else if cv[0] == "bpm" {
            match cv[1].parse::<i16>() {
                Ok(msg) => {
                    self.dtstk.change_bpm(msg);
                    self.sndr.send_msg_to_elapse(ElpsMsg::Set([MSG_SET_BPM, msg]));
                    self.sndr.send_all_vari_and_phrase(self.input_part, &self.dtstk);
                    "BPM has changed!".to_string()
                },
                Err(e) => {
                    println!("{:?}",e);
                    "Number is wrong.".to_string()
                },
            }
        }
        else if cv[0] == "beat" {
            let beat = &cv[1];
            let numvec = split_by('/', beat.to_string());
            match (numvec[0].parse::<i16>(), numvec[1].parse::<i16>()) {
                (Ok(numerator),Ok(denomirator)) => {
                    self.dtstk.change_beat(numerator, denomirator);
                    self.sndr.send_msg_to_elapse(ElpsMsg::SetBeat([numerator, denomirator]));
                    self.sndr.send_all_vari_and_phrase(self.input_part, &self.dtstk);
                    "Beat has changed!".to_string()
                },
                _ => "Number is wrong.".to_string()
            }
        }
        else if cv[0] == "input" {
            if self.change_input_mode(&cv[1]) {
                "Input mode has changed!".to_string()
            }
            else {"what?".to_string()}
        }
        else if cv[0] == "samenote" {
            "what?".to_string()
        }
        else if cv[0] == "turnnote" {
            if self.change_turnnote(&cv[1]) {
                "Turn note has changed!".to_string()
            }
            else {"what?".to_string()}            
        }
        else {
            "what?".to_string()
        }
    }
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
                        '#' => {key += 1; num_txt = key_text[2..].to_string();},
                        'b' => {key -= 1; num_txt = key_text[2..].to_string();},
                        _ => {num_txt = key_text[1..].to_string();},
                    }
                }
                if let Ok(oct_num) = num_txt.parse::<i32>() {oct = oct_num;}
            }
            if key < 0 {key+=12;}
            else if key >= 12 {key-=12;}
            println!("CHANGE KEY: {}, {}",key, oct);
            // phrase 再生成(新oct込み)
            if oct != 0 {
                if self.dtstk.change_oct(oct, false, self.input_part) {
                    self.sndr.send_all_vari_and_phrase(self.input_part, &self.dtstk);
                }
            }
            // elapse に key を送る
            self.sndr.send_msg_to_elapse(ElpsMsg::Set([MSG_SET_KEY, key as i16]));
            self.indicator_key_stock = key_text.to_string();
            true
        }
        else {
            false
        }
    }
    fn change_oct(&mut self, oct_txt: &str) -> bool {
        let mut oct = FULL;
        if let Ok(oct_num) = oct_txt.parse::<i32>() {oct = oct_num;}
        if oct != FULL {
            if self.dtstk.change_oct(oct, true, self.input_part) {
                self.sndr.send_all_vari_and_phrase(self.input_part, &self.dtstk);
                true
            }
            else {false}
        }
        else {false}
    }                                  
    fn change_input_mode(&mut self, imd: &str) -> bool {
        if imd == "fixed" {
            self.dtstk.change_input_mode(InputMode::Fixed);
            true
        }
        else if imd == "closer" {
            self.dtstk.change_input_mode(InputMode::Closer);
            true
        }
        else {false}
    }
    fn change_turnnote(&mut self, ntnum: &str) -> bool {
        if let Ok(turn_note) = ntnum.parse::<i16>() {
            self.sndr.send_msg_to_elapse(ElpsMsg::Set([MSG_SET_TURN, turn_note]));
            true
        }
        else {false}
    }
}