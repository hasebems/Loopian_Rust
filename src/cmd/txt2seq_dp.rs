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
    pr: &mut PhraseRecombined,
    text: String,   // Dynamic Pattern のテキスト
    base_note: i32, // octave などのセッティング
    crnt_tick: i32, // 小節内の現在 tick
    exp_vel: i32,   // dynなどを反映した velocity
    exp_amp: i16,   // dynなどを反映した amplitude
) -> PhrEvt {
    // Cluster or Arpeggio?
    let mut case_arp = true;
    if text.contains("C") {
        case_arp = false;
    }

    // タイを探して追加する tick を算出
    let (tie_dur, bdur_tie, ntext2) = decide_tie_dur(text.clone());

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, mut bdur) = decide_dur(ntext2, pr.base_dur);
    let mut duration = bdur + tie_dur;
    if text == ntext3 {
        duration = pr.rest_tick(crnt_tick); // dur情報がない場合、小節残り全体とする
    }
    let (ntext4, diff_vel, diff_amp) = extrude_diff_vel(ntext3);

    let vel = velo_limits(exp_vel + diff_vel, 1);
    let ev = gen_dp_pattern(
        &ntext4,
        case_arp,
        base_note as i16,
        crnt_tick as i16,
        vel,
        Amp { note_amp: diff_amp, phrase_amp: exp_amp },
        duration as i16,
    );
    //let mut ev = NoteEvt::default();
    //ev.mtype = dp_pattern[0];
    //ev.tick = crnt_tick as i16;
    //ev.each_dur = dp_pattern[3];
    //ev.note = dp_pattern[1] + base_note as i16;
    //ev.trns = dp_pattern[2];
    //ev.vel = velo_limits(exp_vel + diff_vel, 1);
    //ev.dur = duration as i16;

    if bdur_tie != 0 {
        bdur = bdur_tie;
    }
    pr.base_dur = bdur; // 次の音符のために保存しておく
    ev
}
fn gen_dp_pattern(
    nt: &str,
    case_arp: bool,
    base_note: i16,
    tick: i16,
    vel: i16,
    amp: Amp,
    dur: i16,
) -> PhrEvt {
    let arpeggio = if nt.contains("$") && !case_arp {
        if_arpgio(nt)
    } else {
        0
    };
    let params = extract_texts_from_parentheses(nt);
    let param = split_by('@', params.to_string());
    let pnum = param.len();
    let each_dur = if pnum == 0 {
        DEFAULT_TICK_FOR_QUARTER as i16
    } else {
        calc_dur(&param[0])
    };
    let mut lowest = if pnum > 2 {
        param[2].parse::<i16>().unwrap_or(0)
    } else {
        0 // default note
    } + base_note;

    let evt: PhrEvt = if case_arp {
        let figure = if pnum > 1 {
            arp_pattern(&param[1])
        } else {
            4 // default arp pattern
        };
        if (figure % 2) == 1 {
            lowest += 12; // - note;
        }
        PhrEvt::BrkPtn(BrkPatternEvt {
            tick,
            vel,
            amp,
            dur,
            lowest,
            figure,
            each_dur,
            ..BrkPatternEvt::default()
        })
    } else {
        let max_count = if pnum > 1 {
            param[1].parse::<i16>().unwrap_or(4)
        } else {
            4 // default chord count
        };
        PhrEvt::ClsPtn(ClsPatternEvt {
            tick,
            vel,
            dur,
            amp,
            lowest,
            max_count,
            arpeggio,
            each_dur,
            ..ClsPatternEvt::default()
        })
    };
    evt
}
fn if_arpgio(nt: &str) -> i16 {
    let clsltr = split_by('C', nt.to_string());
    if clsltr.len() > 1 {
        if clsltr[0] == "$Q" {
            return 1; // Arpeggio
        } else if clsltr[0] == "$" {
            return 2; // Arpeggio Slow
        } else if clsltr[0] == "$S" {
            return 3; // Arpeggio Quick
        }
    }
    0 // Cluster ではない
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
