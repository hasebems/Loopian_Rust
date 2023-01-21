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
    fn letter_p(_cmd: &str) -> Option<String> {
        Some("Something wrong!".to_string())
    }
    pub fn get_indicator(&self, num: usize) -> &str {&self.indicator[num]}
    pub fn set_and_responce(&mut self, cmd: &str) -> Option<String> {
        println!("Set Text: {}",cmd);
        let first_letter = &cmd[0..1];
        if first_letter == "q" {
            if &cmd[..] == "quit" {None}    //  The End of the App
            else                  {Some("what?".to_string())}
        }
        else if first_letter == "p" {Self::letter_p(cmd)}
        else                        {Some("what?".to_string())}
    }
}