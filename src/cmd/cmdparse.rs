//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;
use std::sync::{mpsc, mpsc::*};
use super::seq_stock::*;

//  LoopianCmd の責務
//  1. Command を受信し中身を調査
//  2. 解析に送る/elapseに送る
//  3. eguiに返事を返す
pub struct LoopianCmd {
    indicator: Vec<String>,
    msg_hndr: mpsc::Sender<Vec<u16>>,
    ui_hndr: mpsc::Receiver<String>,
    input_part: usize,
    gendt: SeqDataStock,
}

impl LoopianCmd {
    const MAX_INDICATOR: u32 = 8;

    pub fn new(msg_hndr: mpsc::Sender<Vec<u16>>, ui_hndr: mpsc::Receiver<String>) -> Self {
        let mut indc: Vec<String> = Vec::new();
        for _ in 0..Self::MAX_INDICATOR {indc.push("---".to_string());}
        indc[0] = "C".to_string();
        indc[1] = lpnlib::DEFAULT_BPM.to_string();
        indc[3] = "1 : 1 : 000".to_string();
        Self {
            indicator: indc,
            msg_hndr,
            ui_hndr,
            input_part: lpnlib::RIGHT1,
            gendt: SeqDataStock::new(),
        }
    }
    fn read_from_ui_hndr(&mut self) {
        loop {
            match self.ui_hndr.try_recv() {
                Ok(mut uitxt)  => {
                    if let Some(letter) = uitxt.chars().nth(0) {
                        let ind_num = letter.to_digit(10).unwrap();
                        let len = uitxt.chars().count();
                        if len >= 2 {
                            let txt = uitxt.split_off(1);
                            if ind_num < Self::MAX_INDICATOR {
                                self.indicator[ind_num as usize] = txt;
                            }
                        }
                    }
                },
                Err(TryRecvError::Disconnected) => break,// Wrong!
                Err(TryRecvError::Empty) => break,
            }
        }
    }
    pub fn get_indicator(&mut self, num: usize) -> &str {
        self.read_from_ui_hndr();
        &self.indicator[num]
    }
    fn send_msg_to_elapse(&self, msg: Vec<u16>) {
        match self.msg_hndr.send(msg) {
            Err(e) => println!("Something happened on MPSC! {}",e),
            _ => {},
        }
    }
    fn send_phrase_to_elapse(&self, part: usize) {
        let pdstk = self.gendt.get_pdstk(part);
        let mut pdt: Vec<u16> = pdstk.get_final();
        if pdt.len() > 1 {
            let mut msg: Vec<u16> = vec![lpnlib::MSG_PHR+self.input_part as u16];
            msg.append(&mut pdt);
            //println!("msg check: {:?}",msg);
            self.send_msg_to_elapse(msg);
        }
        else {println!("Part {}: No Data!",part)}
    }
    fn send_composition_to_elapse(&self, part: usize) {
        let cdstk = self.gendt.get_cdstk(part);
        let mut cdt: Vec<u16> = cdstk.get_final();
        if cdt.len() > 1 {
            let mut msg: Vec<u16> = vec![lpnlib::MSG_CMP+self.input_part as u16];
            msg.append(&mut cdt);
            //println!("msg check: {:?}",msg);
            self.send_msg_to_elapse(msg);
        }
        else {println!("Part {}: No Data!",part)}
    }
    fn change_key(&mut self, key_text: &str) -> bool {
        let mut key = lpnlib::END_OF_DATA;
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
        if key != lpnlib::END_OF_DATA {
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
            self.send_msg_to_elapse(vec![lpnlib::MSG_SET, lpnlib::MSG2_KEY, key as u16]);
            self.indicator[0] = key_text.to_string();
            true
        }
        else {
            false
        }
    }
    fn change_oct(&mut self, oct_txt: &str) -> bool {
        let mut oct = lpnlib::FULL;
        if let Ok(oct_num) = oct_txt.parse::<i32>() {oct = oct_num;}
        if oct != lpnlib::FULL {
            if self.gendt.change_oct(oct, true, self.input_part) {
                self.send_phrase_to_elapse(self.input_part);
                true
            }
            else {false}
        }
        else {false}
    }
    fn parse_set_command(&mut self, input_text: &str) -> String {
        let cmnd = &input_text[4..];
        let _len = cmnd.chars().count();
        let cv = cmnd.split('=').fold(Vec::new(), |mut s, i| {
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
            match cv[1].parse::<u16>() {
                Ok(msg) => {
                    self.gendt.change_bpm(msg);
                    self.send_msg_to_elapse(vec![lpnlib::MSG_SET, lpnlib::MSG2_BPM, msg]);
                    for i in 0..lpnlib::MAX_USER_PART {
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
            let numvec = lpnlib::split_by('/', beat.to_string());
            match (numvec[0].parse::<u16>(), numvec[1].parse::<u16>()) {
                (Ok(up),Ok(low)) => {
                    self.gendt.change_beat(up, low);
                    self.send_msg_to_elapse(vec![lpnlib::MSG_SET, lpnlib::MSG2_BEAT, up, low]);
                    for i in 0..lpnlib::MAX_USER_PART {
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
    fn letter_p(&self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "play" {
            // play
            self.send_msg_to_elapse(vec![lpnlib::MSG_START]);
            Some("Phrase has started!".to_string())
        } else if len >= 5 && &input_text[0..5] == "panic" {
            // panic
            Some("All Sound Off!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_s(&mut self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "stop" {
            // stop
            self.send_msg_to_elapse(vec![lpnlib::MSG_STOP]);
            Some("Phrase has stopped!".to_string())
        } else if len >= 3 && &input_text[0..3] == "set" {
            // set
            let responce = self.parse_set_command(input_text);
            Some(responce)
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
    fn letter_a(&self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 2 && &input_text[0..2] == "aa" {
            self.send_msg_to_elapse(vec![lpnlib::MSG_PHR,lpnlib::TYPE_NOTE,0,480,64,100]);
            Some("Test Phrase1".to_string())
        } else if len >= 2 && &input_text[0..2] == "ab" {
            self.send_msg_to_elapse(vec![lpnlib::MSG_PHR,lpnlib::TYPE_NOTE,0,480,64,100,lpnlib::TYPE_NOTE,0,480,68,100]);
            Some("Test Phrase2".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    pub fn set_and_responce(&mut self, input_text: &str) -> Option<String> {
        println!("Set Text: {}",input_text);
        let first_letter = &input_text[0..1];
        if first_letter == "q" {
            if &input_text[..] == "quit" {
                self.send_msg_to_elapse(vec![lpnlib::MSG_QUIT]);
                None    //  The End of the App
            }
            else {Some("what?".to_string())}
        }
        //<<DoItLater>>
        else if first_letter == "[" {self.letter_bracket(input_text)}
        else if first_letter == "{" {self.letter_brace(input_text)}
        else if first_letter == "a" {self.letter_a(input_text)}
        //else if first_letter == "b" {self.letter_b(input_text)}
        //else if first_letter == "c" {self.letter_c(input_text)}
        //else if first_letter == "f" {self.letter_f(input_text)}
        //else if first_letter == "i" {self.letter_i(input_text)}
        //else if first_letter == "l" {self.letter_l(input_text)}
        else if first_letter == "p" {self.letter_p(input_text)}
        //else if first_letter == "r" {self.letter_r(input_text)}
        else if first_letter == "s" {self.letter_s(input_text)}
        //else if first_letter == "m" {self.letter_m(input_text)}
        else                        {Some("what?".to_string())}
    }
}