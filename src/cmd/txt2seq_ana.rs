//  Created by Hasebe Masahiko on 2023/03/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use crate::elapse::ug_content::*;

//*******************************************************************
//          analyse_data
//*******************************************************************
pub fn analyse_data(generated: &UgContent, exps: &Vec<String>) -> UgContent {
    let mut exp_analysis = put_exp_data(exps);
    let mut beat_analysis = analyse_beat(&generated);
    beat_analysis.mix_with(&mut exp_analysis);
    let rcmb = arp_translation(beat_analysis.clone(), exps);
    rcmb
}
//*******************************************************************
// other music expression data format: 
//      1st     TYPE_BEAT:  TYPE_EXP
//      2nd     EXP:        NOPED
//*******************************************************************
fn put_exp_data(exps: &Vec<String>) -> UgContent {
    let noped = exps.iter().any(|exp| exp == "dmp(off)");
    let mut exp = UgContent::new();
    if noped {
        exp.add_dt(vec![TYPE_EXP, NOPED]);
    }
    exp.copy_to()
}
//*******************************************************************
// beat analysis data format: 
// fn analyse_beat()
//      1st     TYPE_BEAT
//      2nd     tick,
//      3rd     dur,
//      4th     note num,      : highest
//      5th     note count,    : at same tick
//          note count が１より大きい時、note num には最も高い音程の音が記録される
//      6th     -1
//
// fn arp_translation()
//  上記で準備した beat_analysis の後ろに、arpeggio 用の解析データを追記
//      6th     0:com, $DIFF:arp,  PARA:para 
//       arp:   arpeggio 用 Note変換を発動させる（前の音と連続している）
//       $DIFF: arp の場合の、前の音との音程の差分
//*******************************************************************
fn analyse_beat(gen: &UgContent) -> UgContent {
    let get_hi = |na:Vec<i16>| -> i16 {
        match na.iter().max() {
            Some(x) => *x,
            None => 0,
        }
    };
    let get_arp = |crnt_t:i16, repeat_head_t:i16| -> (i16,i16) {
        // RPT_HEAD の直後には、ARP_COM を記録しておく
        if crnt_t == repeat_head_t {(ARP_COM, NOTHING)}
        else {(NOTHING,repeat_head_t)}
    };
    let mut crnt_tick = NOTHING;
    let mut note_cnt = 0;
    let mut crnt_dur = 0;
    let mut repeat_head_tick: i16 = NOTHING;
    let mut note_all: Vec<i16> = Vec::new();
    let mut beat_analysis = UgContent::new();
    for nt in gen.naked().iter() {
        if nt[TYPE] != TYPE_NOTE {
            if nt[TYPE] == TYPE_INFO && nt[INFOTP] == RPT_HEAD as i16 {
                repeat_head_tick = nt[TICK];
            }
        }
        else if nt[TICK] == crnt_tick {
            note_cnt += 1;
            note_all.push(nt[NOTE]);
        }
        else {
            if note_cnt > 0 {
                let (arp, rht) = get_arp(crnt_tick, repeat_head_tick);
                repeat_head_tick = rht;
                beat_analysis.add_dt(vec![TYPE_BEAT, crnt_tick, crnt_dur, get_hi(note_all.clone()), note_cnt, arp])
            }
            crnt_tick = nt[TICK];
            crnt_dur = nt[DURATION];
            note_cnt = 1;
            note_all = vec![nt[NOTE]];
        }
    }
    if note_cnt > 0 {
        let (arp, _rht) = get_arp(crnt_tick, repeat_head_tick);
        beat_analysis.add_dt(vec![TYPE_BEAT, crnt_tick, crnt_dur, get_hi(note_all), note_cnt, arp])
    }
    beat_analysis
}
fn arp_translation(beat_analysis: UgContent, exps: &Vec<String>) -> UgContent {
    let para = exps.iter().any(|exp| exp == "para()" || exp == "trns(para)");
    let mut last_note = REST;
    let mut last_cnt = 0;
    let mut crnt_note;
    let mut crnt_cnt;
    let mut total_tick = 0;
    let mut all_dt = beat_analysis.get_all();
    for ana in all_dt.iter_mut() {
        if ana[TYPE] != TYPE_BEAT {continue;}
        // total_tick の更新
        if total_tick != ana[TICK] {
            total_tick = ana[TICK];
            last_note = REST;
            last_cnt = 0;
        }
        else if ana[DURATION] as i32 >= DEFAULT_TICK_FOR_QUARTER {
            total_tick = ana[TICK];
            last_note = REST;
            last_cnt = 0;
        }
        else {
            total_tick += ana[DURATION];
        }

        // crnt_note の更新
        crnt_note = NO_NOTE;
        crnt_cnt = ana[ARP_NTCNT];
        if crnt_cnt == 1 {
            crnt_note = ana[NOTE] as u8;
        }

        // 条件の確認と、ana への情報追加
        //println!("ana_dbg: {},{},{},{}",crnt_cnt,crnt_note,last_cnt,last_note);
        if para {
            ana[ARP_DIFF] = ARP_PARA;    // para
        }
        else if ana[ARP_DIFF] != ARP_COM && // RPT_HEAD のとき、ARP_COM になるので対象外
          last_note <= MAX_NOTE_NUMBER &&
          last_cnt == 1 &&
          crnt_note <= MAX_NOTE_NUMBER &&
          crnt_cnt == 1 &&
          (last_note as i32)-(crnt_note as i32) < 10 &&
          (crnt_note as i32)-(last_note as i32) < 10 {
            // 過去＆現在を比較：単音、かつ、ノート適正、差が10半音以内
            ana[ARP_DIFF] = crnt_note as i16 -last_note as i16; // arp
        }
        else {
            ana[ARP_DIFF] = ARP_COM;    // com
        }
        last_cnt = crnt_cnt;
        last_note = crnt_note;
    }
    UgContent::new_with_dt(all_dt)
}
//*******************************************************************
//          beat_filter
//*******************************************************************
pub fn beat_filter(rcmb: &UgContent, bpm: i16, tick_for_onemsr: i32) -> UgContent {
    const EFFECT: i16 = 20;     // bigger(1..100), stronger
    const MIN_BPM: i16 = 60;
    const MIN_AVILABLE_VELO: i16 = 30;
    const TICK_4_4: f32 = (DEFAULT_TICK_FOR_QUARTER*4) as f32;
    const TICK_3_4: f32 = (DEFAULT_TICK_FOR_QUARTER*3) as f32;
    const TICK_1BT: f32 = DEFAULT_TICK_FOR_QUARTER as f32;
    if bpm < MIN_BPM {return rcmb.clone();}

    // 純粋な四拍子、三拍子のみ対応
    let base_bpm: i16 = (bpm - MIN_BPM)*EFFECT/100;
    let mut all_dt = rcmb.get_all();
    if tick_for_onemsr == TICK_4_4 as i32 {
        for dt in all_dt.iter_mut() {
            if dt[TYPE] != TYPE_NOTE {continue;}
            let tm: f32 = (dt[TICK] as f32 % TICK_4_4)/TICK_1BT;
            let mut vel = dt[VELOCITY];
            if tm == 0.0 {
                vel += base_bpm;
            }
            else if tm == 2.0 {
                vel += base_bpm/4;
            }
            else {
                vel -= base_bpm/4;
            }
            if vel>127 {vel=127;}
            else if vel < MIN_AVILABLE_VELO {vel=MIN_AVILABLE_VELO;}
            dt[VELOCITY] = vel;
        }
    }
    else if tick_for_onemsr == TICK_3_4 as i32 {
        for dt in all_dt.iter_mut() {
            if dt[TYPE] != TYPE_NOTE {continue;}
            let tm: f32 = (dt[TICK] as f32 % TICK_3_4)/TICK_1BT;
            let mut vel = dt[VELOCITY];
            if tm == 0.0 {
                vel += base_bpm;
            }
            else if tm == 1.0 {
                vel += base_bpm/4;
            }
            else {
                vel -= base_bpm/4;
            }
            if vel>127 {vel=127;}
            else if vel < MIN_AVILABLE_VELO {vel=MIN_AVILABLE_VELO;}
            dt[VELOCITY] = vel;
        }
    }
    UgContent::new_with_dt(all_dt)
}
pub fn crispy_tick(rcmb: &UgContent, exp_others: &Vec<String>) -> UgContent {
    let mut stacc = false;
    if exp_others.iter().any(|x| x=="stacc()" || x=="artic(stacc)") {
        stacc = true;
    }
    let mut all_dt = rcmb.get_all();
    for dt in all_dt.iter_mut() {
        if dt[TYPE] != TYPE_NOTE {continue;}
        let mut return_dur = dt[DURATION];
        if stacc {
            return_dur = return_dur/2;
        }
        else if return_dur > 40 {  // 一律 duration 40 を引く
            return_dur -= 40;
        }
        dt[DURATION] = return_dur;
    }
    UgContent::new_with_dt(all_dt)
}