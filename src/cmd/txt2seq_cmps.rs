//  Created by Hasebe Masahiko on 2023/02/24.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

pub struct TextParseCmps {}
impl TextParseCmps {
    pub fn something_todo(){}
    pub fn complement_phrase(_input_text: String) -> Vec<String>{
        vec!["".to_string()]
    }
    pub fn recombine_to_internal_format(_comp: &Vec<String>, _tick_for_onemsr: i32) -> (i32, Vec<Vec<u16>>) {
        (0, vec![vec![0]])
    }
}