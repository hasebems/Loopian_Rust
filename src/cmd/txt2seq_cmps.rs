//  Created by Hasebe Masahiko on 2023/02/24.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;

pub struct TextParseCmps {}
impl TextParseCmps {
    pub fn something_todo(){}
    pub fn complement_composition(input_text: String) -> [Vec<String>;2] {
        // 1. {} を抜き出し、２つ分の brackets を Vec に入れて戻す
        let (cd, ce) = TextParseCmps::divide_brace(input_text);

        // 2. 重複補填と ',' で分割
        let cmps_vec = TextParseCmps::fill_omitted_chord_data(cd);

        // 3. Expression を ',' で分割
        let ex_vec = lpnlib::split_by(',', ce);

        [cmps_vec, ex_vec]
    }
    pub fn divide_brace(input_text: String) -> (String, String) {
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

        let blk_cnt = cmps_info.len();
        if blk_cnt >= 2 {
            (cmps_info[0].clone(), cmps_info[1].clone())
        }
        else if blk_cnt == 1 {
            (cmps_info[0].clone(), "".to_string())
        }
        else {
            ("".to_string(), "".to_string())
        }
    }
    fn fill_omitted_chord_data(cmps: String) -> Vec<String> {
        //  省略を thru で補填
        const NO_CHORD: &str = "thru";
        let mut end_flag: bool = false;
        let mut fill: String = "".to_string();
        let mut chord: String = NO_CHORD.to_string();
        for ltr in cmps.chars() {
            if ltr == ',' {
                fill += &chord;
                fill += ",";
                chord = NO_CHORD.to_string();
                end_flag = true;
            }
            else if ltr == '/' || ltr == '|' {
                fill += &chord;
                fill += "|,";
                chord = NO_CHORD.to_string();
                end_flag = true;
            }
            else {
                if end_flag {
                    chord = ltr.to_string();
                    end_flag = false;
                }
                else {
                    chord.push(ltr);
                }
            }
        }
        // space を削除
        fill.retain(|c| !c.is_whitespace());

        // ',' で分割
        lpnlib::split_by(',', fill)
    }
    //=========================================================================
    pub fn recombine_to_chord_loop(comp: &Vec<String>, tick_for_onemsr: i32, tick_for_onebeat: i32)
      -> (i32, Vec<Vec<u16>>) {
        if comp.len() == 0 {
            return (0, vec![vec![0]]);
        }
        let btcnt = tick_for_onemsr/tick_for_onebeat;
        let max_read_ptr = comp.len();
        let mut read_ptr = 0;
        let mut tick: i32 = 0;
        let mut msr: i32 = 0;
        let mut rcmb: Vec<Vec<u16>> = Vec::new();
        let mut same_chord: String = "x".to_string();

        while read_ptr < max_read_ptr {
            let (chord, dur) = TextParseCmps::divide_chord_info(comp[read_ptr].clone(), btcnt);
            if tick < tick_for_onemsr*msr {
                if same_chord != chord {
                    same_chord = chord.clone();
                    let (root, table) = TextParseCmps::convert_chord_to_num(chord);
                    rcmb.push(vec![tick as u16, root, table]);
                }
                tick += tick_for_onebeat;
            }
            if dur == btcnt {
                tick = tick_for_onemsr*msr;
                same_chord = "x".to_string();
                msr += 1;
            }
            read_ptr += 1;  // out from repeat
        }        

        tick = msr*tick_for_onemsr;
        (tick, rcmb)
    }
    fn divide_chord_info(mut chord: String, btcnt: i32) -> (String, i32) {
        let mut dur: i32 = 1;
        let mut ltr_count = chord.len();
        if ltr_count == 0 {
            return (chord, dur);
        }

        let mut last_ltr = chord.chars().last().unwrap_or(' ');
        if last_ltr == '|' {
            dur =  btcnt;
            chord = chord[0..ltr_count-1].to_string();
        }
        else {
            while ltr_count >= 1 && last_ltr == '.' {
                dur += 1;
                chord = chord[0..ltr_count-1].to_string();
                last_ltr = chord.chars().last().unwrap_or(' ');
                ltr_count = chord.len();
            }
        }
        (chord, dur)
    }
    fn convert_chord_to_num(_chord: String) -> (u16, u16) {
        (0,0)
    }
}