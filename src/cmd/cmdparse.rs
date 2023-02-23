//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;
use std::sync::{mpsc, mpsc::*};
use super::seq_stock::*;
use super::txt2seq::TextParse;

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
    fn parse_set_command(&mut self, input_text: &str) -> String {
        let cmnd = &input_text[4..];
        let len = cmnd.chars().count();
        if len > 4 && &cmnd[0..4] == "bpm=" {
            match cmnd[4..].parse::<u16>() {
                Ok(msg) => {
                    self.send_msg_to_elapse(vec![lpnlib::MSG_SET, lpnlib::MSG2_BPM, msg]);
                    "BPM has changed!".to_string()
                },
                Err(_e) => {
                    println!("{:?}",_e);
                    "Number is wrong.".to_string()
                },
            }
        }
        else if len > 5 && &cmnd[0..5] == "beat=" {
            let beat = &cmnd[5..];
            let numvec = TextParse::split_by('/', beat.to_string());
            match (numvec[0].parse::<u16>(), numvec[1].parse::<u16>()) {
                (Ok(up),Ok(low)) => {
                    self.gendt.change_beat(up, low);
                    self.send_msg_to_elapse(vec![lpnlib::MSG_SET, lpnlib::MSG2_BEAT, up, low]);
                    "Beat has changed!".to_string()
                },
                _ => "Number is wrong.".to_string()
            }
        }
        else if len > 4 && &cmnd[0..4] == "key=" {
            "what?".to_string()
        }
        else if len > 4 && &cmnd[0..4] == "oct=" {
            "what?".to_string()
        }
        else if len > 6 && &cmnd[0..6] == "input=" {
            "what?".to_string()
        }
        else if len > 9 && &cmnd[0..9] == "samenote=" {
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
            let mut msg: Vec<u16> = vec![lpnlib::MSG_PHR+self.input_part as u16];
            let pdstk = self.gendt.get_pdstk(self.input_part);
            let mut pdt: Vec<u16> = pdstk.get_final();
            msg.append(&mut pdt);
            self.send_msg_to_elapse(msg);
            Some("Set Phrase!".to_string())
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
        //else if first_letter == "{" {self.letter_brace(input_text)}
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