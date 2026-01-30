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
    for (i, ltr) in txt.char_indices() {
        if ltr == splitter {
            splited.push(txt[old_locate..i].to_string());
            old_locate = i + 1;
        }
    }
    splited.push(txt[old_locate..txt.len()].to_string());
    splited
}
#[allow(dead_code)]
pub fn split_by_by(sp1: char, sp2: char, txt: String) -> Vec<String> {
    let mut splited: Vec<String> = Vec::new();
    let mut old_locate: usize = 0;
    for (i, ltr) in txt.char_indices() {
        if ltr == sp1 || ltr == sp2 {
            splited.push(txt[old_locate..i].to_string());
            old_locate = i + 1;
        }
    }
    splited.push(txt[old_locate..txt.len()].to_string());
    splited
}
pub fn doremi_to_notenum(doremi: String, mut base_note: i32) -> i32 {
    if doremi.is_empty() {
        base_note = NO_NOTE as i32;
    } else {
        // d,r,m,f,s,l,t
        base_note = doremi_number(doremi.chars().next().unwrap_or(' '), base_note);
        if doremi.len() == 2 {
            // i,a
            base_note = doremi_semi_number(doremi.chars().nth(1).unwrap_or(' '), base_note);
        } else if doremi.len() == 3 {
            // ii,aa
            base_note = doremi_semi_number(doremi.chars().nth(1).unwrap_or(' '), base_note);
            base_note = doremi_semi_number(doremi.chars().nth(2).unwrap_or(' '), base_note);
        }
    }
    base_note
}
fn doremi_number(ltr: char, mut base_note: i32) -> i32 {
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
fn doremi_semi_number(ltr: char, mut base_note: i32) -> i32 {
    match ltr {
        'i' => base_note += 1,
        'a' => base_note -= 1,
        _ => (),
    }
    base_note
}
pub fn get_pure_doremi(org_nt: i32) -> i32 {
    let mut pure_doremi = org_nt;
    while pure_doremi >= 12 {
        pure_doremi -= 12;
    }
    while pure_doremi < 0 {
        pure_doremi += 12;
    }
    pure_doremi
}
//*******************************************************************
//          extract_xxx_from_parentheses
//*******************************************************************
pub fn extract_number_from_parentheses(ne: &str) -> Option<usize> {
    match (ne.find('('), ne.find(')')) {
        (Some(i), Some(e)) if i + 1 < e => ne[(i + 1)..e].to_string().parse().ok(),
        _ => None,
    }
}
#[allow(dead_code)]
pub fn extract_anynumber_from_parentheses<T: std::str::FromStr>(ne: &str) -> Option<T> {
    match (ne.find('('), ne.find(')')) {
        (Some(i), Some(e)) if i + 1 < e => ne[(i + 1)..e].parse::<T>().ok(),
        _ => None,
    }
}
pub fn extract_texts_from_parentheses(ne: &str) -> &str {
    match (ne.find('('), ne.find(')')) {
        (Some(i), Some(e)) if i < e => &ne[(i + 1)..e],
        _ => "",
    }
}
pub fn separate_cmnd_and_str(cn: &str) -> Option<(&str, &str)> {
    match (cn.find('('), cn.find(')')) {
        (Some(i), Some(e)) if i < e => Some((&cn[0..i], &cn[(i + 1)..e])),
        _ => None,
    }
}
//*******************************************************************
//          Data Time Text
//*******************************************************************
pub fn get_crnt_date_txt() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S ").to_string()
}
