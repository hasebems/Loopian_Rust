//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;
use std::sync::{mpsc, mpsc::*};
use super::cmdstkseq::*;

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
    fn set_raw_phrase(&mut self, input_text: &str) -> bool {
        self.gendt.set_raw_phrase(self.input_part, input_text.to_string())
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
    fn letter_s(&self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "stop" {
            // stop
            self.send_msg_to_elapse(vec![lpnlib::MSG_STOP]);
            Some("Phrase has stopped!".to_string())
        } else if len >= 3 && &input_text[0..3] == "set" {
            // set
            self.send_msg_to_elapse(
                vec![lpnlib::MSG_SET, input_text[4..].parse::<u16>().unwrap()]);
            Some("BPM has changed!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    fn letter_bracket(&mut self, input_text: &str) -> Option<String> {
        if self.set_raw_phrase(input_text) {
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