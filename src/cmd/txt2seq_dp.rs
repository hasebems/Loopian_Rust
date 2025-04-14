//  Created by Hasebe Masahiko on 2024/10/07.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use super::txt2seq_phr::*;
use crate::lpnlib::*;

pub fn available_for_dp(text: &str) -> bool {
    !(!text.contains("C(")
        && !text.contains("Cls(")
        && !text.contains("A(")
        && !text.contains("Arp("))
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

    // タイを探して追加する tick を算出
    let (tie_dur, bdur_tie, ntext2) = decide_tie_dur(text.clone());

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, mut bdur) = decide_dur(ntext2, base_dur);
    let mut duration = bdur + tie_dur;
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

    if bdur_tie != 0 {
        bdur = bdur_tie;
    }
    (ev, bdur)
}
fn gen_dp_pattern(nt: &str, case_arp: bool) -> Vec<i16> {
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
        if case_arp && ((trns % 2) == 1) {
            note += 12; // - note;
        }
    }

    vec![mtype, note, trns, each_dur]
}
fn calc_dur(durstr: &str) -> i16 {
    let mut dur = 480;
    let mut val = durstr.chars().next().unwrap_or(' ');
    let mut dot = 2;
    let mut div = 2;
    if val == '5' {
        div = 5;
        val = durstr.chars().nth(1).unwrap_or(' ');
    } else if val == '3' {
        div = 3;
        val = durstr.chars().nth(1).unwrap_or(' ');
    } else if durstr.len() > 1 {
        let c = durstr.chars().nth(1).unwrap_or(' ');
        if c == '\'' {
            dot = 3;
        }
    }
    match val {
        'h' => dur = 960,
        'q' => dur = 480,
        'e' => dur = 240,
        'v' => dur = 120,
        'w' => dur = 60,
        _ => {}
    }
    dur * dot / div
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
