//  Created by Hasebe Masahiko on 2024/10/07.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt2seq_phr::*;
use super::txt_common::*;
use crate::lpnlib::*;

pub fn available_for_dp(text: &String) -> bool {
    if !text.contains("C(")
        && !text.contains("Cls(")
        && !text.contains("A(")
        && !text.contains("Arp(")
    {
        false
    } else {
        true
    }
}
/// Note のときの fn break_up_nt_dur_vel() と同様の処理
pub fn treat_dp(
    text: String,   // Dynamic Pattern のテキスト
    base_note: i32, // octave などのセッティング
    base_dur: i32,  // 前の duration
    crnt_tick: i32, // 小節内の現在 tick
    rest_tick: i32, // 小節内の残り tick
    exp_vel: i32,   // dynなどを反省した velocity
) -> (PhrEvt, i32) {
    // Cluster or Arpeggio?
    let mut case_arp = true;
    if text.contains("C") {
        case_arp = false;
    }

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, bdur) = decide_dur(text.clone(), base_dur);
    let mut duration = bdur;
    if text == ntext3 {
        duration = rest_tick; // dur情報がない場合、小節残り全体とする
    }
    let (ntext4, diff_vel) = gen_diff_vel(ntext3);

    let mut ev = PhrEvt::default();
    let dp_pattern = gen_dp_pattern(&ntext4, case_arp);
    ev.mtype = dp_pattern[0];
    ev.tick = crnt_tick as i16;
    ev.each_dur = dp_pattern[3];
    ev.note = dp_pattern[1] + base_note as i16;
    ev.trns = dp_pattern[2];
    ev.vel = velo_limits(exp_vel + diff_vel, 1);
    ev.dur = duration as i16;

    (ev, bdur)
}
fn gen_dp_pattern(nt: &String, case_arp: bool) -> Vec<i16> {
    let params = extract_texts_from_parentheses(nt);
    let param = split_by('@', params.to_string());
    let pnum = param.len();
    let mut mtype = TYPE_CLS;
    if case_arp {
        mtype = TYPE_ARP;
    }

    let mut note = 0;
    let mut trns = 4;
    let mut each_dur = DEFAULT_TICK_FOR_QUARTER as i16;
    if pnum > 0 {
        each_dur = calc_dur(&param[0]);
    }
    if pnum > 1 {
        if case_arp {
            trns = arp_pattern(&param[1]);
        } else {
            trns = param[1].parse::<i16>().unwrap_or(4);
        }
    }
    if pnum > 2 {
        note = param[2].parse::<i16>().unwrap_or(0);
    }

    vec![mtype, note, trns, each_dur]
}
fn calc_dur(durstr: &String) -> i16 {
    let mut dur = 480;
    let ch0 = durstr.chars().nth(0).unwrap_or(' ');
    let dot = if durstr.len() > 1 {
        let c = durstr.chars().nth(1).unwrap_or(' ');
        if c == '\'' {
            3
        } else {
            2
        }
    } else {
        2
    };
    if ch0 == 'h' {
        dur = 960;
    } else if ch0 == 'q' {
        dur = 480;
    } else if ch0 == 'e' {
        dur = 240;
    } else if ch0 == 'v' {
        dur = 120;
    }
    dur * dot / 2
}
fn arp_pattern(ptn: &str) -> i16 {
    match ptn {
        "u" => 0,
        "d" => 1,
        "ux" => 2,
        "ud" => 3,
        _ => 0,
    }
}
