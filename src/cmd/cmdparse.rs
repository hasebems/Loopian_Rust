//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

pub struct LoopianCmd {
    indicator: Vec<String>,
}

impl LoopianCmd {
    pub fn new() -> Self {
        let mut indc: Vec<String> = Vec::new();
        for _ in 0..8 {indc.push("---".to_string());}
        Self {
            indicator: indc,
        }
    }
    fn letter_p(&self, _input_text: &str) -> Option<String> {
        Some("Something wrong!".to_string())
    }
    pub fn get_indicator(&self, num: usize) -> &str {&self.indicator[num]}
    pub fn set_and_responce(&mut self, input_text: &str) -> Option<String> {
        println!("Set Text: {}",input_text);
        let first_letter = &input_text[0..1];
        if first_letter == "q" {
            if &input_text[..] == "quit" {None}    //  The End of the App
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