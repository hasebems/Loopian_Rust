//  Created by Hasebe Masahiko on 2024/01/27
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::elapse_note::Damper;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::lpnlib::*;

//*******************************************************************
//          Damper Part Struct
//*******************************************************************
pub struct DamperPart {
    id: ElapseId,
    priority: u32,
    during_play: bool,
    next_msr: i32,
    next_tick: i32,
    start_flag: bool,
    position: i16,

    evt: Vec<DmprEvt>,
    play_counter: usize,
    whole_tick: i32,
}
impl DamperPart {
    pub fn new(num: u32) -> Rc<RefCell<DamperPart>> {
        let new_id = ElapseId {
            pid: 0,
            sid: num,
            elps_type: ElapseType::TpDamperPart,
        };
        Rc::new(RefCell::new(Self {
            id: new_id,
            priority: PRI_DMPR,
            during_play: false,
            next_msr: 0,
            next_tick: 0,
            start_flag: false,
            position: 127,

            evt: Vec::new(),
            play_counter: 0,
            whole_tick: 0,
        }))
    }
    pub fn set_position(&mut self, pos: i16) {
        self.position = pos;
    }
    /// 次回イベントの小節、tickを算出する
    fn gen_next_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32) {
        if srtick == END_OF_DATA {
            (crnt_.msr + 1, 0)
        } else {
            let tick = srtick % crnt_.tick_for_onemsr;
            let msr = crnt_.msr + srtick / crnt_.tick_for_onemsr;
            (msr, tick)
        }
    }
    /// 1小節内にあるイベントを適切なタイミングで出力する
    fn output_event(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        elapsed_tick: i32,
    ) -> i32 {
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let evt = self.evt.to_vec();
        let max_ev = self.evt.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA; // means sequence finished
                break;
            }
            next_tick = evt[trace].tick as i32;
            if next_tick <= elapsed_tick {
                if evt[trace].mtype == TYPE_DAMPER {
                    let dmpr: Rc<RefCell<dyn Elapse>> = Damper::new(
                        (crnt_.msr as u32) * 100 + (trace as u32), //  msr&read pointer
                        self.id.sid,                               //  pedal part.sid -> damper.pid
                        estk,
                        &evt[trace],
                        self.next_msr,
                        self.next_tick,
                    );
                    estk.add_elapse(Rc::clone(&dmpr));
                }
            } else {
                break;
            }
            trace += 1;
        }

        self.play_counter = trace;
        next_tick
    }
    fn gen_events_in_msr(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> i32 {
        // 小節頭でコールされる
        let (tick_for_onemsr, tick_for_onebeat) = estk.tg().get_beat_tick();
        let beat_num: usize = (tick_for_onemsr / tick_for_onebeat) as usize;
        self.whole_tick = tick_for_onemsr;
        self.play_counter = 0;

        let mut chord_map = vec![false; beat_num];
        if let Some(_fl) = estk.get_flow() {
            chord_map = DamperPart::merge_chord_map(crnt_, estk, FLOW_PART, chord_map, beat_num);
        }
        for i in 0..MAX_KBD_PART {
            if let Some(phr) = estk.get_phr(i) {
                if phr.borrow().get_noped() {
                    // 一パートでも noped 指定があれば
                    chord_map = vec![false; beat_num];
                    break;
                } else {
                    chord_map = DamperPart::merge_chord_map(crnt_, estk, i, chord_map, beat_num);
                }
            } else {
                continue;
            }
        }
        let tick;
        (self.evt, tick) = self.gen_real_damper_track(chord_map, tick_for_onebeat, beat_num);
        tick
    }
    /// 各パートのChord情報より、Damper 情報を beat にどんどん足していく
    fn merge_chord_map(
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        part_num: usize,
        mut chord_map: Vec<bool>,
        beat_num: usize,
    ) -> Vec<bool> {
        if let Some(pt) = estk.part(part_num as u32) {
            let mut pt_borrowed = pt.borrow_mut();
            let cmp_med = pt_borrowed.get_cmps_med();
            let ba = cmp_med.get_chord_ev_map(crnt_, beat_num);
            if ba.len() != chord_map.len() {
                // もし長さが違ったら、エラー
                println!(
                    "<<< part{}/beat{}: {}->{}",
                    part_num,
                    beat_num,
                    chord_map.len(),
                    ba.len()
                );
                panic!("DamperPart::merge_chord_map: length mismatch");
            }
            for (i, x) in chord_map.iter_mut().enumerate() {
                *x |= ba[i];
            }
        }
        chord_map
    }
    fn gen_real_damper_track(
        &self,
        chord_map: Vec<bool>,
        tick_for_onebeat: i32,
        beat_num: usize,
    ) -> (Vec<DmprEvt>, i32) {
        let mut keep: usize = beat_num;
        let mut dmpr_evt: Vec<DmprEvt> = Vec::new();
        let mut first_tick = NO_DATA;
        const PDL_MARGIN_TICK: i32 = 60;
        for (j, k) in chord_map.iter().enumerate() {
            if *k {
                if keep != beat_num {
                    let tick = ((keep as i32) * tick_for_onebeat + PDL_MARGIN_TICK) as i16;
                    dmpr_evt.push(DmprEvt {
                        mtype: TYPE_DAMPER,
                        tick,
                        dur: (((j - keep) as i32) * tick_for_onebeat - PDL_MARGIN_TICK) as i16,
                        position: self.position,
                    });
                    if first_tick == NO_DATA {
                        first_tick = tick as i32
                    }
                }
                keep = j;
            }
        }
        if keep != beat_num {
            let tick = ((keep as i32) * tick_for_onebeat + PDL_MARGIN_TICK) as i16;
            dmpr_evt.push(DmprEvt {
                mtype: TYPE_DAMPER,
                tick,
                dur: (((beat_num - keep) as i32) * tick_for_onebeat - PDL_MARGIN_TICK) as i16,
                position: self.position,
            });
            if first_tick == NO_DATA {
                first_tick = tick as i32
            }
        }
        (dmpr_evt, first_tick)
    }
}
impl Elapse for DamperPart {
    fn id(&self) -> ElapseId {
        self.id
    } // id を得る
    fn prio(&self) -> u32 {
        self.priority
    } // priority を得る
    fn next(&self) -> (i32, i32) {
        // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self, msr: i32) {
        // User による start/play 時にコールされる
        self.during_play = true;
        self.start_flag = true;
        self.next_msr = msr;
        self.next_tick = 0;
    }
    fn stop(&mut self, _estk: &mut ElapseStack) {
        // User による stop 時にコールされる
        self.during_play = false;
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.evt = Vec::new();
        self.next_msr = 0;
        self.next_tick = 0;
    }
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.next_tick == 0 {
            // Damper Event を生成
            let ntick = self.gen_events_in_msr(crnt_, estk);
            if ntick != NO_DATA {
                self.next_msr = crnt_.msr;
                self.next_tick = ntick;
            } else {
                self.next_msr = crnt_.msr + 1;
                self.next_tick = 0;
            }
        }

        let elapsed_tick = crnt_.tick;
        if elapsed_tick >= self.next_tick {
            // Damper Event を再生
            let next_tick = self.output_event(crnt_, estk, elapsed_tick);
            let (msr, tick) = self.gen_next_msr_tick(crnt_, next_tick);
            self.next_msr = msr;
            self.next_tick = tick;
        }
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {
        // 自クラスが役割を終えた時に True を返す
        false
    }
}
