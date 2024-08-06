//  Created by Hasebe Masahiko on 2024/05/02.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use chrono::Local;

//*******************************************************************
//          Func
//*******************************************************************
pub fn convert_exp2vel(vel_text: &str) -> i32 {
    match vel_text {
        "ff" => 127,
        "f" => 114,
        "mf" => 100,
        "mp" => 84,
        "p" => 64,
        "pp" => 48,
        "ppp" => 24,
        "pppp" => 12,
        "ppppp" => 1,
        _ => END_OF_DATA,
    }
}
pub fn split_by(splitter: char, txt: String) -> Vec<String> {
    let mut splited: Vec<String> = Vec::new();
    let mut old_locate: usize = 0;
    for (i, ltr) in txt.chars().enumerate() {
        if ltr == splitter {
            splited.push((&txt[old_locate..i]).to_string());
            old_locate = i + 1;
        }
    }
    splited.push((&txt[old_locate..txt.len()]).to_string());
    splited
}
#[allow(dead_code)]
pub fn split_by_by(sp1: char, sp2: char, txt: String) -> Vec<String> {
    let mut splited: Vec<String> = Vec::new();
    let mut old_locate: usize = 0;
    for (i, ltr) in txt.chars().enumerate() {
        if ltr == sp1 || ltr == sp2 {
            splited.push((&txt[old_locate..i]).to_string());
            old_locate = i + 1;
        }
    }
    splited.push((&txt[old_locate..txt.len()]).to_string());
    splited
}
pub fn doremi_number(ltr: char, mut base_note: i32) -> i32 {
    match ltr {
        'd' => base_note += 0,
        'r' => base_note += 2,
        'm' => base_note += 4,
        'f' => base_note += 5,
        's' => base_note += 7,
        'l' => base_note += 9,
        't' => base_note += 11,
        _ => base_note = NO_NOTE as i32,
    }
    base_note
}
pub fn doremi_semi_number(ltr: char, mut base_note: i32) -> i32 {
    match ltr {
        'i' => base_note += 1,
        'a' => base_note -= 1,
        _ => (),
    }
    base_note
}
//*******************************************************************
//          extract_xxx_from_parentheses
//*******************************************************************
pub fn extract_number_from_parentheses(ne: &str) -> usize {
    if let Some(i) = ne.find('(') {
        if let Some(e) = ne.find(')') {
            if i < e {
                let numtxt = if i + 1 < e {
                    ne[(i + 1)..e].to_string()
                } else {
                    '1'.to_string()
                };
                return numtxt.parse().unwrap_or(0);
            } else {
                return 1;
            }
        }
    }
    0
}
pub fn extract_texts_from_parentheses(ne: &str) -> &str {
    if let Some(i) = ne.find('(') {
        if let Some(e) = ne.find(')') {
            if i <= e {
                return &ne[(i + 1)..e];
            }
        }
    }
    ""
}
pub fn separate_cmnd_and_str(cn: &str) -> Option<(&str, &str)> {
    if let Some(i) = cn.find('(') {
        if let Some(e) = cn.find(')') {
            if i <= e {
                return Some((&cn[0..i], &cn[(i + 1)..e]));
            }
        }
    }
    None
}
//*******************************************************************
//          Data Time Text
//*******************************************************************
pub fn get_crnt_date_txt() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S ").to_string()
}
