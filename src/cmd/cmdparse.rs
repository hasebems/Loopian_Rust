//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::LoopianApp;
use crate::lpnlib::*;
use std::sync::{mpsc, mpsc::*};
use super::seq_stock::*;

//  LoopianCmd の責務
//  1. Command を受信し中身を調査
//  2. 解析に送る/elapseに送る
//  3. eguiに返事を返す
pub struct LoopianCmd {
    indicator: Vec<String>,
    indicator_key_stock: String,
    msg_hndr: mpsc::Sender<Vec<i16>>,
    ui_hndr: mpsc::Receiver<String>,
    input_part: usize,
    gendt: SeqDataStock,
}

impl LoopianCmd {
    pub fn new(msg_hndr: mpsc::Sender<Vec<i16>>, ui_hndr: mpsc::Receiver<String>) -> Self {
        let mut indicator = vec![String::from("---"); LoopianApp::MAX_INDICATOR];
        indicator[0] = "C".to_string();
        indicator[1] = DEFAULT_BPM.to_string();
        indicator[3] = "1 : 1 : 000".to_string();
        Self {
            indicator,
            indicator_key_stock: "C".to_string(),
            msg_hndr,
            ui_hndr,
            input_part: RIGHT1,
            gendt: SeqDataStock::new(),
        }
    }
    pub fn get_input_part(&self) -> usize {self.input_part}
    pub fn get_part_txt(&self) -> &str {
        match self.input_part {
            LEFT1 => "L1>",            
            LEFT2 => "L2>",
            RIGHT1 => "R1>",
            RIGHT2 => "R2>",
            _ => "__>",
        }
    }
    pub fn get_indicator(&mut self, num: usize) -> &str {
        self.read_from_ui_hndr();
        &self.indicator[num]
    }
    fn read_from_ui_hndr(&mut self) {
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
                            else if ind_num < LoopianApp::MAX_INDICATOR {
                                self.indicator[ind_num] = txt;
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
            if &input_text[..] == "quit" {
                self.send_msg_to_elapse(vec![MSG_QUIT]);
                None    //  The End of the App
            }
            else {Some("what?".to_string())}
        }
        else if first_letter == "[" {self.letter_bracket(input_text)}
        else if first_letter == "{" {self.letter_brace(input_text)}
        else if first_letter == "f" {self.letter_f(input_text)}
        else if first_letter == "l" {self.letter_l(input_text)}
        else if first_letter == "p" {self.letter_p(input_text)}
        else if first_letter == "r" {self.letter_r(input_text)}
        else if first_letter == "s" {self.letter_s(input_text)}
        else                        {Some("what?".to_string())}
    }
    fn letter_f(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len == 7 && &input_text[0..7] == "fermata" {
            // fermata
            self.send_msg_to_elapse(vec![MSG_FERMATA]);
            Some("Will be longer!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_l(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 5 && &input_text[0..5] == "left1" {
            self.input_part = LEFT1;
            Some("Changed current part to left1.".to_string())
        } else if len >= 5 && &input_text[0..5] == "left2" {
            self.input_part = LEFT2;
            Some("Changed current part to left2.".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_p(&self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "play" {
            // play
            self.send_msg_to_elapse(vec![MSG_START]);
            Some("Phrase has started!".to_string())
        } else if len >= 5 && &input_text[0..5] == "panic" {
            // panic
            self.send_msg_to_elapse(vec![MSG_PANIC]);
            Some("All Sound Off!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_r(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 6 && &input_text[0..6] == "right1" {
            self.input_part = RIGHT1;
            Some("Changed current part to right1.".to_string())
        } else if len >= 6 && &input_text[0..6] == "right2" {
            self.input_part = RIGHT2;
            Some("Changed current part to right2.".to_string())
        } else if len >= 4 && &input_text[0..4] == "rit." {
            let strength_txt: String;
            let mut aft_rit: i16 = MSG3_ATP;
            let mut strength_value: i16 = MSG2_NRM;
            if input_text.chars().any(|x| x=='/') {
                let rit_txt_raw = &input_text[4..];
                let rit_txt = split_by('/', rit_txt_raw.to_string());
                let nxt_msr_txt = &rit_txt[1];
                if nxt_msr_txt == "fermata" {aft_rit = MSG3_FERMATA;}
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
            if strength_txt == "poco" {strength_value = MSG2_POCO;}
            else if strength_txt == "molto" {strength_value = MSG2_MLT;}
            self.send_msg_to_elapse(vec![MSG_RIT, strength_value, aft_rit]);
            Some("rit. has started!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_s(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "stop" {
            // stop
            self.send_msg_to_elapse(vec![MSG_STOP]);
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
                self.send_msg_to_elapse(vec![MSG_SYNC, self.input_part as i16]);
                Some("Synchronized!".to_string())
            } else if vectxt[1] == "right" {
                self.send_msg_to_elapse(vec![MSG_SYNC, MSG2_RGT]);
                Some("Right Part Synchronized!".to_string())
            } else if vectxt[1] == "left" {
                self.send_msg_to_elapse(vec![MSG_SYNC, MSG2_LFT]);
                Some("Left Part Synchronized!".to_string())
            } else if vectxt[1] == "all" {
                self.send_msg_to_elapse(vec![MSG_SYNC, MSG2_ALL]);
                Some("All Part Synchronized!".to_string())
            } else {
                Some("what?".to_string())
            }
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_bracket(&mut self, input_text: &str) -> Option<String> {
        if self.gendt.set_raw_phrase(self.input_part, input_text.to_string()) {
            self.send_phrase_to_elapse(self.input_part);
            Some("Set Phrase!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_brace(&mut self, input_text: &str) -> Option<String> {
        if self.gendt.set_raw_composition(self.input_part, input_text.to_string()) {
            self.send_composition_to_elapse(self.input_part);
            Some("Set Composition!".to_string())
        } else {
            Some("what?".to_string())
        }
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
                    self.gendt.change_bpm(msg);
                    self.send_msg_to_elapse(vec![MSG_SET, MSG2_BPM, msg]);
                    for i in 0..MAX_USER_PART {
                        self.send_phrase_to_elapse(i);
                    }
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
                    self.gendt.change_beat(numerator, denomirator);
                    self.send_msg_to_elapse(vec![MSG_SET, MSG2_BEAT, numerator, denomirator]);
                    for i in 0..MAX_USER_PART {
                        self.send_phrase_to_elapse(i);
                    }
                    "Beat has changed!".to_string()
                },
                _ => "Number is wrong.".to_string()
            }
        }
        else if cv[0] == "input" {
            "what?".to_string()
        }
        else if cv[0] == "samenote" {
            "what?".to_string()
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
                if self.gendt.change_oct(oct, false, self.input_part) {
                    self.send_phrase_to_elapse(self.input_part);
                }
            }
            // elapse に key を送る
            self.send_msg_to_elapse(vec![MSG_SET, MSG2_KEY, key as i16]);
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
            if self.gendt.change_oct(oct, true, self.input_part) {
                self.send_phrase_to_elapse(self.input_part);
                true
            }
            else {false}
        }
        else {false}
    }
    //*************************************************************************
    fn send_msg_to_elapse(&self, msg: Vec<i16>) {
        match self.msg_hndr.send(msg) {
            Err(e) => println!("Something happened on MPSC for Elps! {}",e),
            _ => {},
        }
    }
    fn send_phrase_to_elapse(&self, part: usize) {
        let (mut pdt, mut ana): (Vec<i16>, Vec<i16>) = self.gendt.get_pdstk(part).get_final();
        if pdt.len() > 1 {
            let mut msg: Vec<i16> = vec![MSG_PHR+part as i16];
            msg.append(&mut pdt);
            //println!("msg check: {:?}",msg);
            self.send_msg_to_elapse(msg);
            if ana.len() > 1 {
                let mut msgana: Vec<i16> = vec![MSG_ANA+part as i16];
                msgana.append(&mut ana);
                //println!("msg check ana: {:?}",msgana);
                self.send_msg_to_elapse(msgana);                
            }
        }
        else {
            self.send_msg_to_elapse(vec![MSG_PHR_X+part as i16]);
            if ana.len() == 0 {
                self.send_msg_to_elapse(vec![MSG_ANA_X+part as i16]);
            }
            println!("Part {} Phrase: No Data!",part);
        }
    }
    fn send_composition_to_elapse(&self, part: usize) {
        let mut cdt: Vec<i16> = self.gendt.get_cdstk(part).get_final();
        if cdt.len() > 1 {
            let mut msg: Vec<i16> = vec![MSG_CMP+self.input_part as i16];
            msg.append(&mut cdt);
            //println!("msg check: {:?}",msg);
            self.send_msg_to_elapse(msg);
        }
        else {
            self.send_msg_to_elapse(vec![MSG_CMP_X+self.input_part as i16]);
            println!("Part {} Composition: No Data!",part)
        }
    }
}