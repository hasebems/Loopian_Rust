//  Created by Hasebe Masahiko on 2023/03/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
//use crate::elapse::ug_content::*;

//*******************************************************************
//          analyse_data
//*******************************************************************
pub fn analyse_data(generated: &Vec<PhrEvt>, exps: &Vec<String>) -> Vec<AnaEvt> {
    let mut exp_analysis = put_exp_data(exps);
    let beat_analysis = analyse_beat(&generated);
    exp_analysis.append(&mut beat_analysis.clone()); //.mix_with(&mut );
    let rcmb = arp_translation(beat_analysis, exps);
    rcmb
}
//*******************************************************************
// other music expression data format: 
//      1st     TYPE_BEAT:  TYPE_EXP
//      2nd     EXP:        NOPED
//*******************************************************************
fn put_exp_data(exps: &Vec<String>) -> Vec<AnaEvt> {
    let noped = exps.iter().any(|exp| exp == "dmp(off)");
    let mut exp = Vec::new();
    if noped {
        //exp.add_dt(vec![TYPE_EXP, NOPED]);
        let mut anev = AnaEvt::new();
        anev.mtype = TYPE_EXP;
        anev.atype = NOPED;
        exp.push(anev);
    }
    exp //.copy_to()
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
fn analyse_beat(gen: &Vec<PhrEvt>) -> Vec<AnaEvt> {
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
    let mut beat_analysis = Vec::new();
    for nt in gen.iter() {
        if nt.mtype != TYPE_NOTE {
            if nt.mtype == TYPE_INFO && nt.note == RPT_HEAD as i16 {
                repeat_head_tick = nt.tick;
            }
        }
        else if nt.tick == crnt_tick {
            note_cnt += 1;
            note_all.push(nt.note);
        }
        else {
            if note_cnt > 0 {
                let (arp, rht) = get_arp(crnt_tick, repeat_head_tick);
                repeat_head_tick = rht;
                beat_analysis.push(AnaEvt{
                    mtype: TYPE_BEAT,
                    tick: crnt_tick,
                    dur: crnt_dur,
                    note: get_hi(note_all.clone()),
                    cnt: note_cnt,
                    atype: arp
                }) //vec![TYPE_BEAT, crnt_tick, crnt_dur, get_hi(note_all.clone()), note_cnt, arp]
            }
            crnt_tick = nt.tick;
            crnt_dur = nt.dur;
            note_cnt = 1;
            note_all = vec![nt.note];
        }
    }
    if note_cnt > 0 {
        let (arp, _rht) = get_arp(crnt_tick, repeat_head_tick);
        beat_analysis.push(AnaEvt {
            mtype: TYPE_BEAT,
            tick: crnt_tick,
            dur: crnt_dur, 
            note: get_hi(note_all), 
            cnt: note_cnt,
            atype: arp,
        });
        //add_dt(vec![TYPE_BEAT, crnt_tick, crnt_dur, get_hi(note_all), note_cnt, arp])
    }
    beat_analysis
}
fn arp_translation(beat_analysis: Vec<AnaEvt>, exps: &Vec<String>) -> Vec<AnaEvt> {
    let para = exps.iter().any(|exp| exp == "para()" || exp == "trns(para)");
    let mut last_note = REST;
    let mut last_cnt = 0;
    let mut crnt_note;
    let mut crnt_cnt;
    let mut total_tick = 0;
    let mut all_dt = beat_analysis.clone();
    for ana in all_dt.iter_mut() {
        if ana.mtype != TYPE_BEAT {continue;}
        // total_tick の更新
        if total_tick != ana.tick {
            total_tick = ana.tick;
            last_note = REST;
            last_cnt = 0;
        }
        else if ana.dur as i32 >= DEFAULT_TICK_FOR_QUARTER {
            total_tick = ana.tick;
            last_note = REST;
            last_cnt = 0;
        }
        else {
            total_tick += ana.dur;
        }

        // crnt_note の更新
        crnt_note = NO_NOTE;
        crnt_cnt = ana.cnt;
        if crnt_cnt == 1 {
            crnt_note = ana.note as u8;
        }

        // 条件の確認と、ana への情報追加
        //println!("ana_dbg: {},{},{},{}",crnt_cnt,crnt_note,last_cnt,last_note);
        if para {
            ana.atype = ARP_PARA;    // para
        }
        else if ana.atype != ARP_COM && // RPT_HEAD のとき、ARP_COM になるので対象外
          last_note <= MAX_NOTE_NUMBER &&
          last_cnt == 1 &&
          crnt_note <= MAX_NOTE_NUMBER &&
          crnt_cnt == 1 &&
          (last_note as i32)-(crnt_note as i32) < 10 &&
          (crnt_note as i32)-(last_note as i32) < 10 {
            // 過去＆現在を比較：単音、かつ、ノート適正、差が10半音以内
            ana.atype = crnt_note as i16 -last_note as i16; // arp
        }
        else {
            ana.atype = ARP_COM;    // com
        }
        last_cnt = crnt_cnt;
        last_note = crnt_note;
    }
    all_dt
    //UgContent::new_with_dt()
}
//*******************************************************************
//          beat_filter
//*******************************************************************
pub fn beat_filter(rcmb: &Vec<PhrEvt>, bpm: i16, tick_for_onemsr: i32) -> Vec<PhrEvt> {
    const EFFECT: i16 = 20;     // bigger(1..100), stronger
    const MIN_BPM: i16 = 60;
    const MIN_AVILABLE_VELO: i16 = 30;
    const TICK_4_4: f32 = (DEFAULT_TICK_FOR_QUARTER*4) as f32;
    const TICK_3_4: f32 = (DEFAULT_TICK_FOR_QUARTER*3) as f32;
    const TICK_1BT: f32 = DEFAULT_TICK_FOR_QUARTER as f32;
    if bpm < MIN_BPM {return rcmb.clone();}

    // 純粋な四拍子、三拍子のみ対応
    let base_bpm: i16 = (bpm - MIN_BPM)*EFFECT/100;
    let mut all_dt = rcmb.clone();
    if tick_for_onemsr == TICK_4_4 as i32 {
        for dt in all_dt.iter_mut() {
            if dt.mtype != TYPE_NOTE {continue;}
            let tm: f32 = (dt.tick as f32 % TICK_4_4)/TICK_1BT;
            let mut vel = dt.vel;
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
            dt.vel = vel;
        }
    }
    else if tick_for_onemsr == TICK_3_4 as i32 {
        for dt in all_dt.iter_mut() {
            if dt.mtype != TYPE_NOTE {continue;}
            let tm: f32 = (dt.tick as f32 % TICK_3_4)/TICK_1BT;
            let mut vel = dt.vel;
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
            dt.vel = vel;
        }
    }
    all_dt
    //UgContent::new_with_dt(all_dt)
}
pub fn crispy_tick(rcmb: &Vec<PhrEvt>, exp_others: &Vec<String>) -> Vec<PhrEvt> {
    let mut stacc = false;
    if exp_others.iter().any(|x| x=="stacc()" || x=="artic(stacc)") {
        stacc = true;
    }
    let mut all_dt = rcmb.clone();
    for dt in all_dt.iter_mut() {
        if dt.mtype != TYPE_NOTE {continue;}
        let mut return_dur = dt.dur;
        if stacc {
            return_dur = return_dur/2;
        }
        else if return_dur > 40 {  // 一律 duration 40 を引く
            return_dur -= 40;
        }
        dt.dur = return_dur;
    }
    all_dt
    //UgContent::new_with_dt(all_dt)
}