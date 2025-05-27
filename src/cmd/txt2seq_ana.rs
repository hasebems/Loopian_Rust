//  Created by Hasebe Masahiko on 2023/03/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use crate::lpnlib::*;

//*******************************************************************
//          analyse_data
//*******************************************************************
pub fn analyse_data(generated: &[PhrEvtx], exps: &[String]) -> Vec<AnaEvt> {
    let mut exp_analysis = put_exp_data(exps);
    let mut beat_analysis = analyse_beat(generated);
    exp_analysis.append(&mut beat_analysis);
    let mut crispy_analysis = crispy_tick(exps);
    exp_analysis.append(&mut crispy_analysis);
    arp_translation(exp_analysis, exps)
}
//*******************************************************************
// other music expression data format:
//      1st     TYPE_BEAT:  TYPE_EXP
//      2nd     EXP:        NOPED
//                          PARA_ROOT(noteに値を入れる)
//*******************************************************************
fn put_exp_data(exps: &[String]) -> Vec<AnaEvt> {
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
fn analyse_beat(phr_evts: &[PhrEvtx]) -> Vec<AnaEvt> {
    let get_hi = |na: Vec<u8>| -> u8 {
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
    let mut note_all: Vec<u8> = Vec::new();
    let mut beat_analysis = Vec::new();
    for phr in phr_evts.iter() {
        match phr {
            PhrEvtx::Note(e) => {
                if e.tick == crnt_tick {
                    note_cnt += 1;
                    note_all.push(e.note);
                    if crnt_trns != TRNS_COM {
                        crnt_trns = e.trns; // 和音で一つに限定
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
                            note: get_hi(note_all.clone()) as i16,
                            cnt: note_cnt,
                            atype: arp,
                        })
                    }
                    crnt_tick = e.tick;
                    crnt_dur = e.dur;
                    crnt_trns = e.trns;
                    note_cnt = 1;
                    note_all = vec![e.note];
                }
            }
            PhrEvtx::Info(i) => {
                if i.info == RPT_HEAD as i16 {
                    repeat_head_tick = i.tick;
                }
            }
            _ => (),
        }
    }
    if note_cnt > 0 {
        let (arp, _rht) = get_arp(crnt_tick, repeat_head_tick, crnt_trns);
        beat_analysis.push(AnaEvt {
            mtype: TYPE_BEAT,
            tick: crnt_tick,
            dur: crnt_dur,
            note: get_hi(note_all) as i16,
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
fn arp_translation(beat_analysis: Vec<AnaEvt>, exps: &[String]) -> Vec<AnaEvt> {
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
        #[cfg(feature = "verbose")]
        println!(
            "ana_dbg: {},{},{},{}",
            crnt_cnt, crnt_note, last_cnt, last_note
        );
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
    if para {
        // Note情報がない場合、Dynamic Pattern 用にpara指定メッセージを作成
        let mut ae = AnaEvt::new();
        ae.mtype = TYPE_EXP; // 上では TYPE_BEAT （音符単位）に TRNS_PARA が付く
        ae.atype = TRNS_PARA;
        all_dt.push(ae);
    }
    all_dt
}
//*******************************************************************
//  fn crispy_tick()
//      mtype = TYPE_EXP, atype = ARTIC
//      cnt: Staccato Rate
//*******************************************************************
pub fn crispy_tick(exp_others: &[String]) -> Vec<AnaEvt> {
    let mut ana: Vec<AnaEvt> = vec![];
    exp_others.iter().for_each(|x| {
        if x.contains("stacc(") {
            let mut rate;
            if let Some(r) = extract_number_from_parentheses(x) {
                rate = r;
            } else {
                rate = 50;
            }
            if rate >= 100 {
                rate = 100;
            }
            let mut anev = AnaEvt::new();
            anev.mtype = TYPE_EXP;
            anev.cnt = rate as i16;
            anev.atype = ARTIC;
            ana.push(anev);
        }
    });
    exp_others.iter().for_each(|x| {
        if x.contains("legato(") {
            let mut rate;
            if let Some(r) = extract_number_from_parentheses(x) {
                rate = r;
            } else {
                rate = 120;
            }
            rate = rate.clamp(100, 200);
            let mut anev = AnaEvt::new();
            anev.mtype = TYPE_EXP;
            anev.cnt = rate as i16;
            anev.atype = ARTIC;
            ana.push(anev);
        }
    });
    ana
}
//*******************************************************************
//          beat_filter
//*******************************************************************
const EFFECT: i16 = 20; // bigger(1..100), stronger
const MIN_BPM: i16 = 60;
const MIN_AVILABLE_VELO: i16 = 30;
const TICK_1BT: f32 = DEFAULT_TICK_FOR_QUARTER as f32;
pub fn beat_filter(
    rcmb: &[PhrEvtx],
    bpm: i16,
    tick_for_onemsr: i32,
    tick_for_beat: i32,
) -> Vec<PhrEvtx> {
    if bpm < MIN_BPM {
        return rcmb.to_owned();
    }

    // 4/4拍子、3/4拍子、3n/8拍子に対応
    let mut all_dt = rcmb.to_vec();
    if tick_for_onemsr == TICK_4_4 as i32 {
        for dt in all_dt.iter_mut() {
            if let PhrEvtx::Note(e) = dt {
                e.vel = calc_vel_for4(e.vel, e.tick as f32, bpm);
            }
        }
    } else if tick_for_onemsr == TICK_3_4 as i32 && tick_for_beat == DEFAULT_TICK_FOR_QUARTER {
        for dt in all_dt.iter_mut() {
            if let PhrEvtx::Note(e) = dt {
                e.vel = calc_vel_for3(e.vel, e.tick as f32, bpm);
            }
        }
    } else if (tick_for_onemsr % (DEFAULT_TICK_FOR_QUARTER / 2)) % 3 == 0
        && tick_for_beat == DEFAULT_TICK_FOR_QUARTER / 2
    {
        for dt in all_dt.iter_mut() {
            if let PhrEvtx::Note(dt) = dt {
                dt.vel = calc_vel_for3_8(dt.vel, dt.tick as f32, bpm);
            }
        }
    }
    all_dt
}
pub fn calc_vel_for4(input_vel: i16, tick: f32, bpm: i16) -> i16 {
    let base_bpm = (bpm - MIN_BPM) * EFFECT / 100;
    let tm: f32 = (tick % TICK_4_4) / TICK_1BT;
    let mut vel = input_vel;
    if tm == 0.0 {
        vel += base_bpm;
    } else if tm == 2.0 {
        vel += base_bpm / 4;
    } else {
        vel -= base_bpm / 4;
    }
    velo_limits(vel as i32, MIN_AVILABLE_VELO as i32)
}
pub fn calc_vel_for3(input_vel: i16, tick: f32, bpm: i16) -> i16 {
    const TICK_1BT: f32 = DEFAULT_TICK_FOR_QUARTER as f32;
    let base_bpm = (bpm - MIN_BPM) * EFFECT / 100;
    let tm: f32 = (tick % TICK_3_4) / TICK_1BT;
    let mut vel = input_vel;
    if tm == 0.0 {
        vel += base_bpm;
    } else if tm == 1.0 {
        vel += base_bpm / 4;
    } else {
        vel -= base_bpm / 4;
    }
    velo_limits(vel as i32, MIN_AVILABLE_VELO as i32)
}
pub fn calc_vel_for3_8(input_vel: i16, tick: f32, bpm: i16) -> i16 {
    const TICK_1BT: f32 = DEFAULT_TICK_FOR_QUARTER as f32 / 2.0;
    let base_bpm = if bpm < MIN_BPM * 2 {
        2
    } else {
        (bpm - MIN_BPM * 2) * EFFECT / 200
    };
    let tm: f32 = (tick % (TICK_1BT * 3.0)) / TICK_1BT;
    let mut vel = input_vel;
    if tm == 0.0 {
        vel += base_bpm;
    } else {
        vel -= base_bpm / 4;
    }
    velo_limits(vel as i32, MIN_AVILABLE_VELO as i32)
}
