//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;

//  LoopianCmd の責務
//  1. Command を受信し中身を調査
//  2. 解析に送る/elapseに送る
//  3. eguiに返事を返す
pub struct LoopianCmd {
    indicator: Vec<String>,
    msg_hndr: mpsc::Sender<String>,
    _ui_hndr: mpsc::Receiver<String>,
}

impl LoopianCmd {
    pub fn new(msg_hndr: mpsc::Sender<String>, _ui_hndr: mpsc::Receiver<String>) -> Self {
        let mut indc: Vec<String> = Vec::new();
        for _ in 0..8 {indc.push("---".to_string());}
        indc[3] = "1 : 1 : 000".to_string();
        Self {
            indicator: indc,
            msg_hndr,
            _ui_hndr,
        }
    }
    pub fn get_indicator(&self, num: usize) -> &str {&self.indicator[num]}
    fn send_msg_to_elapse(&self, msg: &str) {
        match self.msg_hndr.send(msg.to_string()) {
            Err(e) => println!("Something happened! {}",e),
            _ => {},
        }
    }
    fn letter_p(&self, input_text: &str) -> Option<String> {
        let len = input_text.chars().count();
        if len >= 4 && &input_text[0..4] == "play" {
            // play
            self.send_msg_to_elapse("play");
            Some("Phrase has started!".to_string())
        } else if len >= 5 && &input_text[0..5] == "panic" {
            // panic
            Some("Phrase has started!".to_string())
        } else {
            Some("what?".to_string())
        }
    }
    pub fn set_and_responce(&mut self, input_text: &str) -> Option<String> {
        println!("Set Text: {}",input_text);
        let first_letter = &input_text[0..1];
        if first_letter == "q" {
            if &input_text[..] == "quit" {
                self.send_msg_to_elapse("quit");
                None    //  The End of the App
            }
            else {Some("what?".to_string())}
        }
        //else if first_letter == "[" {self.letter_bracket(input_text)}
        //else if first_letter == "{" {self.letter_brace(input_text)}
        //else if first_letter == "a" {self.letter_a(input_text)}
        //else if first_letter == "b" {self.letter_b(input_text)}
        //else if first_letter == "c" {self.letter_c(input_text)}
        //else if first_letter == "f" {self.letter_f(input_text)}
        //else if first_letter == "i" {self.letter_i(input_text)}
        //else if first_letter == "l" {self.letter_l(input_text)}
        else if first_letter == "p" {self.letter_p(input_text)}
        //else if first_letter == "r" {self.letter_r(input_text)}
        //else if first_letter == "s" {self.letter_s(input_text)}
        //else if first_letter == "m" {self.letter_m(input_text)}
        else                        {Some("what?".to_string())}
    }
}