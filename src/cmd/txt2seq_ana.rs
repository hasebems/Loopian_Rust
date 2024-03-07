//  Created by Hasebe Masahiko on 2023/03/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;

//*******************************************************************
//          analyse_data
//*******************************************************************
pub fn analyse_data(generated: &Vec<PhrEvt>, exps: &Vec<String>) -> Vec<AnaEvt> {
    let mut exp_analysis = put_exp_data(exps);
    let mut beat_analysis = analyse_beat(&generated);
    beat_analysis.append(&mut exp_analysis);
    let rcmb = arp_translation(beat_analysis, exps);
    rcmb
}
//*******************************************************************
// other music expression data format:
//      1st     TYPE_BEAT:  TYPE_EXP
//      2nd     EXP:        NOPED
//                          PARA_ROOT(noteに値を入れる)
//*******************************************************************
fn put_exp_data(exps: &Vec<String>) -> Vec<AnaEvt> {
    let noped = exps.iter().any(|exp| exp == "dmp(off)");
    let asmin = exps.iter().any(|exp| exp == "asMin()" || exp == "as(VI)");
    let mut exp = Vec::new();
    if noped {
        let mut anev = AnaEvt::new();
        anev.mtype = TYPE_EXP;
        anev.atype = NOPED;
        exp.push(anev);
    }
    if asmin {
        let mut anev = AnaEvt::new();
        anev.mtype = TYPE_EXP;
        anev.note = -3; // VI
        anev.atype = PARA_ROOT;
        exp.push(anev);
    }
    exp //.copy_to()
}
//*******************************************************************
/// Beat 分析
///     - TYPE_BEAT 情報を追加
///     - 和音の場合は高音のみをそのタイミングに記載
///     - phr.trns が、TRNS_COM 以外の場合、意図的なのでそのまま atype に入れる
///     - RPT_HEAD のタイミングは、atype = TRNS_COM (arp_translation()で評価しない)
///     - 上記以外は、ARP の可能性があるので、atype = NOTHING に書き換える
// fn analyse_beat()
//      mtype   TYPE_BEAT
//      tick
//      dur
//      note    note num,      : highest
//      cnt     note count,    : at same tick
//              note count が１より大きい時、note num には最も高い音程の音が記録される
//      atype   NOTHING
//*******************************************************************
fn analyse_beat(phr_evts: &Vec<PhrEvt>) -> Vec<AnaEvt> {
    let get_hi = |na: Vec<i16>| -> i16 {
        match na.iter().max() {
            Some(x) => *x,
            None => 0,
        }
    };
    let get_arp = |crnt_t: i16, repeat_head_t: i16, trns: i16| -> (i16, i16) {
        if trns != TRNS_COM {
            (trns, NOTHING)
        } else if crnt_t == repeat_head_t {
            (TRNS_COM, NOTHING)
        }
        // RPT_HEAD の直後には、TRNS_COM を記録しておく
        else {
            (NOTHING, repeat_head_t)
        } // Arpeggio 候補
    };
    let mut crnt_tick = NOTHING;
    let mut note_cnt = 0;
    let mut crnt_dur = 0;
    let mut crnt_trns = TRNS_COM;
    let mut repeat_head_tick: i16 = NOTHING;
    let mut note_all: Vec<i16> = Vec::new();
    let mut beat_analysis = Vec::new();
    for phr in phr_evts.iter() {
        if phr.mtype != TYPE_NOTE {
            if phr.mtype == TYPE_INFO && phr.note == RPT_HEAD as i16 {
                repeat_head_tick = phr.tick;
            }
        } else if phr.tick == crnt_tick {
            note_cnt += 1;
            note_all.push(phr.note);
            if crnt_trns != TRNS_COM {
                crnt_trns = phr.trns; // 和音で一つに限定
            }
        } else {
            if note_cnt > 0 {
                // 一つ前の Note （あるいは和音の最高音）を記録
                let (arp, rht) = get_arp(crnt_tick, repeat_head_tick, crnt_trns);
                repeat_head_tick = rht;
                beat_analysis.push(AnaEvt {
                    mtype: TYPE_BEAT,
                    tick: crnt_tick,
                    dur: crnt_dur,
                    note: get_hi(note_all.clone()),
                    cnt: note_cnt,
                    atype: arp,
                })
            }
            crnt_tick = phr.tick;
            crnt_dur = phr.dur;
            crnt_trns = phr.trns;
            note_cnt = 1;
            note_all = vec![phr.note];
        }
    }
    if note_cnt > 0 {
        let (arp, _rht) = get_arp(crnt_tick, repeat_head_tick, crnt_trns);
        beat_analysis.push(AnaEvt {
            mtype: TYPE_BEAT,
            tick: crnt_tick,
            dur: crnt_dur,
            note: get_hi(note_all),
            cnt: note_cnt,
            atype: arp,
        });
    }
    beat_analysis
}
//*******************************************************************
/// analyse_beat() で準備した beat_analysis の後ろに、arpeggio 用の解析データを追記
//  fn arp_translation()
//      atype   TRNS_COM / $DIFF:arp / TRNS_PARA
//       arp:   arpeggio 用 Note変換を発動させる（前の音と連続している）
//       $DIFF: arp の場合の、前の音との音程の差分
//*******************************************************************
fn arp_translation(beat_analysis: Vec<AnaEvt>, exps: &Vec<String>) -> Vec<AnaEvt> {
    let para = exps
        .iter()
        .any(|exp| exp == "para()" || exp == "trns(para)");
    let mut last_note = REST;
    let mut last_cnt = 0;
    let mut crnt_note;
    let mut crnt_cnt;
    let mut total_tick = 0;
    let mut all_dt = beat_analysis.clone();
    for ana in all_dt.iter_mut() {
        if ana.mtype != TYPE_BEAT {
            continue;
        }

        // total_tick の更新
        if total_tick != ana.tick {
            // 前の音符の間に休符がある
            total_tick = ana.tick;
            last_note = REST;
            last_cnt = 0;
        } else if ana.dur as i32 >= DEFAULT_TICK_FOR_QUARTER {
            total_tick = ana.tick;
            last_note = REST;
            last_cnt = 0;
        } else {
            total_tick += ana.dur;
        }

        // crnt_note の更新
        crnt_note = NO_NOTE;
        crnt_cnt = ana.cnt;
        if crnt_cnt == 1 {
            // 和音でなければ
            crnt_note = ana.note as u8;
        }

        // 条件の確認と、ana への情報追加
        // RPT_HEAD のとき、TRNS_COM になるので対象外
        //println!("ana_dbg: {},{},{},{}",crnt_cnt,crnt_note,last_cnt,last_note);
        if para {
            // 強制的に para
            ana.atype = TRNS_PARA; // para
        } else if ana.atype == NOTHING {
            if last_note <= MAX_NOTE_NUMBER
                && last_cnt == 1
                && crnt_note <= MAX_NOTE_NUMBER
                && crnt_cnt == 1
                && (last_note as i32) - (crnt_note as i32) < 10
                && (crnt_note as i32) - (last_note as i32) < 10
            {
                // 過去＆現在を比較：単音、かつ、ノート適正、差が10半音以内
                ana.atype = crnt_note as i16 - last_note as i16; // arp
            } else {
                // NOTHING で ARP にならなかったものは TRNS_COM
                ana.atype = TRNS_COM;
            }
        }
        last_cnt = crnt_cnt;
        last_note = crnt_note;
    }
    all_dt
}
//*******************************************************************
//          beat_filter
//*******************************************************************
pub fn beat_filter(rcmb: &Vec<PhrEvt>, bpm: i16, tick_for_onemsr: i32) -> Vec<PhrEvt> {
    const EFFECT: i16 = 20; // bigger(1..100), stronger
    const MIN_BPM: i16 = 60;
    const MIN_AVILABLE_VELO: i16 = 30;
    const TICK_4_4: f32 = (DEFAULT_TICK_FOR_QUARTER * 4) as f32;
    const TICK_3_4: f32 = (DEFAULT_TICK_FOR_QUARTER * 3) as f32;
    const TICK_1BT: f32 = DEFAULT_TICK_FOR_QUARTER as f32;
    if bpm < MIN_BPM {
        return rcmb.clone();
    }

    // 純粋な四拍子、三拍子のみ対応
    let base_bpm: i16 = (bpm - MIN_BPM) * EFFECT / 100;
    let mut all_dt = rcmb.clone();
    if tick_for_onemsr == TICK_4_4 as i32 {
        for dt in all_dt.iter_mut() {
            if dt.mtype != TYPE_NOTE {
                continue;
            }
            let tm: f32 = (dt.tick as f32 % TICK_4_4) / TICK_1BT;
            let mut vel = dt.vel;
            if tm == 0.0 {
                vel += base_bpm;
            } else if tm == 2.0 {
                vel += base_bpm / 4;
            } else {
                vel -= base_bpm / 4;
            }
            if vel > 127 {
                vel = 127;
            } else if vel < MIN_AVILABLE_VELO {
                vel = MIN_AVILABLE_VELO;
            }
            dt.vel = vel;
        }
    } else if tick_for_onemsr == TICK_3_4 as i32 {
        for dt in all_dt.iter_mut() {
            if dt.mtype != TYPE_NOTE {
                continue;
            }
            let tm: f32 = (dt.tick as f32 % TICK_3_4) / TICK_1BT;
            let mut vel = dt.vel;
            if tm == 0.0 {
                vel += base_bpm;
            } else if tm == 1.0 {
                vel += base_bpm / 4;
            } else {
                vel -= base_bpm / 4;
            }
            if vel > 127 {
                vel = 127;
            } else if vel < MIN_AVILABLE_VELO {
                vel = MIN_AVILABLE_VELO;
            }
            dt.vel = vel;
        }
    }
    all_dt
}
pub fn crispy_tick(rcmb: &Vec<PhrEvt>, exp_others: &Vec<String>) -> Vec<PhrEvt> {
    let mut stacc = false;
    if exp_others
        .iter()
        .any(|x| x == "stacc()" || x == "artic(stacc)")
    {
        stacc = true;
    }
    let mut all_dt = rcmb.clone();
    for dt in all_dt.iter_mut() {
        if dt.mtype != TYPE_NOTE {
            continue;
        }
        let mut return_dur = dt.dur;
        if stacc {
            return_dur = return_dur / 2;
        }
        dt.dur = return_dur;
    }
    all_dt
}
