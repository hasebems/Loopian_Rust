//  Created by Hasebe Masahiko on 2023/02/15.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;

pub struct TextParse {}
impl TextParse {
    pub fn complement_phrase(input_text: String) -> [Vec<String>;2] {
        // 1. [] を抜き出し、２つ分の brackets を Vec に入れて戻す
        let (ni, ne) = TextParse::divide_brackets(input_text);

        // 2. ,| 重複による休符指示の補填
        let nf = TextParse::fill_omitted_note_data(ni);

        // 3. , で分割
        let mut nvec = TextParse::split_by_comma(nf);

        // 4. < >*n を展開
        loop {
            let (nvr_tmp, no_exist) = TextParse::expand_repeat(nvec.clone());
            nvec = nvr_tmp.clone();
            if no_exist {break;}
        }

        // 5. 同音繰り返しの展開
        loop {
            let (nvr_tmp, no_exist) = TextParse::note_repeat(nvec.clone());
            nvec = nvr_tmp.clone();
            if no_exist {break;}
        }

        // 6. Expression を , で分割
        let nevec = TextParse::split_by_comma(ne);

        println!("complement_phrase: {:?} exp: {:?}",nvec,nevec);
        return [nvec,nevec];
    }
    fn divide_brackets(input_text: String) -> (String, String) {
        let mut note_info: Vec<String> = Vec::new();

        // [] のセットを抜き出し、中身を note_info に入れる
        let mut isx: &str = &input_text;
        loop {
            if let Some(n2) = isx.find(']') {
                note_info.push(isx[1..n2].to_string());
                isx = &isx[n2+1..];
                if isx.len() == 0 {break;}
                if let Some(n3) = isx.find('[') {
                    if n3 != 0 {break;}
                }
                else {break;}
            }
            else {break;}
        }

        let bracket_num = note_info.len();
        if bracket_num == 1 {
            note_info.push("raw".to_string());
        }
        else if bracket_num == 0 || bracket_num > 2 {
            return ("".to_string(), "".to_string());
        }
        return (note_info[0].clone(), note_info[1].clone());
    }
    fn fill_omitted_note_data(nf: String) -> String {
        // ,| 連続入力による、休符指示の補填
        if nf.len() == 0 {return "".to_string();}

        let mut fill: String = "".to_string();
        let mut doremi = "x".to_string();
        let mut doremi_end_flag = true;
        for ltr in nf.chars() {
            if ltr == ',' {
                fill += &doremi;
                fill += ",";
                doremi = "x".to_string();
                doremi_end_flag = true;
            }
            else if ltr == '|' || ltr == '/' {
                fill += &doremi;
                fill += "|,";
                doremi = "x".to_string();
                doremi_end_flag = true;
            }
            else {
                if doremi_end_flag {
                    doremi = (ltr).to_string();
                    doremi_end_flag = false;
                }
                else {
                    doremi.push(ltr);
                }
            }
        }
        if doremi != "" {
            fill += &doremi;
        }
        fill
    }
    fn expand_repeat(nv: Vec<String>) -> (Vec<String>, bool) {
        let mut new_vec = nv.clone();
        let mut repeat_start: usize = nv.len();
        let mut first_bracket: bool = false;
        let mut no_exist: bool = true;
        for (i, one) in nv.iter().enumerate() {
            if one.contains("<") {
                if let Some(ltr) = &one.chars().nth(0) {
                    if *ltr == '<' {
                        new_vec[i] = one[1..].to_string();
                        repeat_start = i;
                        first_bracket = true;
                    }
                }
            }
            else if first_bracket && one.contains(">") {
                no_exist = false;
                let mut remain_num: usize = one.len();
                for (j, ltr) in one.chars().enumerate() {
                    if ltr == '>' {
                        new_vec[i] = one[0..j].to_string();
                        remain_num = j;
                    }
                    else if j == remain_num+1 && ltr == '*' {
                        let number: i32 = one[j+1..].parse().unwrap_or(0);
                        if number > 1 {
                            for _ in 0..number-1 {
                                for h in (repeat_start..(i+1)).rev() {
                                    new_vec.insert(i+1, new_vec[h].clone());
                                }
                            }
                        }
                        break;
                    }
                }
                break;
            }
        }
        (new_vec, no_exist)
    }
    fn note_repeat(nv: Vec<String>) -> (Vec<String>, bool) {
        let mut new_vec = nv.clone();
        let mut no_exist: bool = true;
        for (i, one) in nv.iter().enumerate() {
            if one.contains("*") {
                no_exist = false;
                for (j, ltr) in one.chars().enumerate() {
                    if ltr == '*' {
                        new_vec[i] = one[..j].to_string();
                        let number: i32 = one[j+1..].parse().unwrap_or(0);
                        if number > 1 {
                            for _ in 0..number-1 {
                                new_vec.insert(i+1, one[..j].to_string());
                            }
                        }
                        break;
                    }
                }
                break;
            }
        }
        (new_vec, no_exist)
    }
    //=========================================================================
    fn get_exp_info(expvec: &Vec<String>) -> (i32, &Vec<String>) {
        (100, expvec)
    }
    fn break_up_nt_dur_vel(oct_setting: i32, note_text: String, last_nt: i32, _imd: lpnlib::InputMode)
      -> (u8, bool, i32, i32, u8, i32) {
        (0,true,0,0,0,0)
    }
    fn get_real_dur(base_dur: i32, dur_info: i32, tick_for_onemsr: i32) -> i32 {
        0
    }
    fn trans_dur(real_dur: i32, exp_others: &Vec<String>) -> i32 {
        0
    }
    fn add_note(rcmb: Vec<u16>, tick: i32, notes: u8, note_dur: i32, last_vel: u8) -> Vec<u16> {rcmb}
    pub fn recombine_to_internal_format(ntvec: &Vec<String>, expvec: &Vec<String>, imd: lpnlib::InputMode,
      oct_setting: i32, tick_for_onemsr: i32) -> (i32, Vec<u16>) {
        let max_read_ptr = ntvec.len();
        let (exp_vel, exp_others) = TextParse::get_exp_info(expvec);
        let mut read_ptr = 0;
        let mut last_nt: i32 = 0;
        let mut tick: i32 = 0;
        let mut msr: i32 = 1;
        let mut rcmb: Vec<u16> = Vec::new();

        while read_ptr < max_read_ptr {
            let note_text = ntvec[read_ptr].clone();

            let (notes, mes_end, base_dur, dur_cnt, diff_vel, nt)
              = TextParse::break_up_nt_dur_vel(oct_setting, note_text, last_nt, imd);

            if nt <= 127 {last_nt = nt;}    // 次回の音程の上下判断のため
            let next_msr_tick = tick_for_onemsr*msr;
            if tick < next_msr_tick {
                let real_dur = TextParse::get_real_dur(base_dur, dur_cnt,
                    next_msr_tick - tick);

                // duration
                let note_dur = TextParse::trans_dur(real_dur, exp_others);
    
                // velocity
                let mut last_vel: i32 = exp_vel + diff_vel as i32;
                if last_vel > 127 {last_vel = 127;}
                else if last_vel < 1 {last_vel = 1;}

                // add to recombined data
                rcmb = TextParse::add_note(rcmb, tick, notes, note_dur, last_vel as u8);
                tick += real_dur;
            }
            if mes_end {// 小節線があった場合
                tick = next_msr_tick;
                msr += 1;
            }
            read_ptr += 1;  // out from repeat
        }
        (tick, rcmb)
    }
    //=========================================================================
    fn split_by_comma(txt: String) -> Vec<String> {
        let mut splited: Vec<String> = Vec::new();
        let mut old_locate: usize = 0;
        for (i, ltr) in txt.chars().enumerate() {
            if ltr == ',' {
                splited.push((&txt[old_locate..i]).to_string());
                old_locate = i+1;
            }
        }
        splited.push((&txt[old_locate..txt.len()]).to_string());
        splited
    }
    pub fn split_by_slash(txt: String) -> Vec<String> {
        let mut splited: Vec<String> = Vec::new();
        let mut old_locate: usize = 0;
        for (i, ltr) in txt.chars().enumerate() {
            if ltr == '/' {
                splited.push((&txt[old_locate..i]).to_string());
                old_locate = i+1;
            }
        }
        splited.push((&txt[old_locate..txt.len()]).to_string());
        splited
    }
}