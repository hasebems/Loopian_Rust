//  Created by Hasebe Masahiko on 2023/02/24.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

pub struct TextParseCmps {}
impl TextParseCmps {
    pub fn something_todo(){}
    pub fn complement_composition(input_text: String) -> [Vec<String>;2] {
        let mut cmps_info: Vec<String> = Vec::new();

        // {} のセットを抜き出し、中身を cmps_info に入れる
        let mut isx: &str = &input_text;
        loop {
            if let Some(n2) = isx.find('}') {
                cmps_info.push(isx[1..n2].to_string());
                isx = &isx[n2+1..];
                if isx.len() == 0 {break;}
                if let Some(n3) = isx.find('{') {
                    if n3 != 0 {break;}
                }
                else {break;}
            }
            else {break;}
        }

        let mut cmps_vec = [vec!["".to_string()],vec!["".to_string()]];
        if cmps_info.len() != 0 {
            cmps_vec[0] = TextParseCmps::fill_omitted_chord_data(cmps_info[0].clone());
            if cmps_info.len() >= 2 {
                cmps_vec[1] = vec![cmps_info[1].clone()];
            }
        }
        cmps_vec
    }
    fn fill_omitted_chord_data(_cmps: String) -> Vec<String> {vec!["".to_string()]}

    pub fn recombine_to_internal_format(_comp: &Vec<String>, _tick_for_onemsr: i32) -> (i32, Vec<Vec<u16>>) {
        (0, vec![vec![0]])
    }
}