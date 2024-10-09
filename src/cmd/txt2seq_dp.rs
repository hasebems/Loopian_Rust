//  Created by Hasebe Masahiko on 2024/10/07.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt2seq_phr::*;
use super::txt_common::*;
use crate::lpnlib::*;

/// Note のときの fn break_up_nt_dur_vel() と同様の処理
pub fn treat_dp(text: String, base_dur: i32, rest_tick: i32) -> Option<(PhrEvt, i32)> {
    if !text.contains("C(")
        && !text.contains("Cls(")
        && !text.contains("A(")
        && !text.contains("Arp(")
    {
        return None;
    }

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, bdur, _dur_cnt) = gen_dur_info(text.clone(), base_dur, rest_tick);
    let (ntext4, _diff_vel) = gen_diff_vel(ntext3);

    if ntext4.contains("Cls(") {
        let cls_pattern = gen_cls_pattern(&text);
        println!("{:?}", cls_pattern);
    }

    // Arpeggio
    if ntext4.contains("Arp(") {
        let arp_pattern = gen_arp_pattern(&text);
        println!("{:?}", arp_pattern);
    }

    Some((PhrEvt::default(), bdur))
}

fn gen_cls_pattern(nt: &String) -> Vec<u8> {
    let params = extract_texts_from_parentheses(&nt);
    let param = split_by(',', params.to_string());
    println!("12345>{:?}", param);
    vec![0]
}
fn gen_arp_pattern(_nt: &String) -> Vec<u8> {
    vec![0]
}
