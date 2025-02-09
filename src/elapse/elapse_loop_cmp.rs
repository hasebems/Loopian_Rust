//  Created by Hasebe Masahiko on 2025/02/09
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use crate::lpnlib::*;
use super::stack_elapse::ElapseStack;
use crate::cmd::txt2seq_cmps::{self, NO_LOOP};
use super::tickgen::CrntMsrTick;

//*******************************************************************
//          Composition Loop Struct
//*******************************************************************
pub struct CompositionLoop {
    id: ElapseId,
    priority: u32,

    cmps_dt: Vec<ChordEvt>,
    keynote: u8,
    play_counter: usize,
    next_tick_in_cmps: i32,

    // for Composition
    chord_name: String,
    root: i16,
    translation_tbl: i16,
    just_after_start: bool,
    already_end: bool,
    no_loop: bool,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl CompositionLoop {
    pub fn new(
        sid: u32,
        pid: u32,
        knt: u8,
        msr: i32,
        msg: Vec<ChordEvt>,
        whole_tick: i32,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpCompositionLoop,
            },
            priority: PRI_CMPS_LOOP,
            cmps_dt: msg,
            keynote: knt,
            play_counter: 0,
            next_tick_in_cmps: 0,

            chord_name: "".to_string(),
            root: NO_ROOT,
            translation_tbl: NO_TABLE,
            just_after_start: false,
            already_end: false,
            no_loop: false,

            // for super's member
            whole_tick,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0, //   次に呼ばれる小節番号が保持される
            next_tick: 0,
        }))
    }
    pub fn get_chord(&self) -> (i16, i16) {
        (self.root, self.translation_tbl)
    }
    pub fn get_chord_name(&self) -> String {
        self.chord_name.clone()
    }
    pub fn get_chord_map(
        &self,
        msr: i32,
        tick_for_onemsr: i32,
        tick_for_onebeat: i32,
    ) -> Vec<bool> {
        // for Damper
        let first_tick = (msr - self.first_msr_num) * tick_for_onemsr;
        let end_tick = (msr - self.first_msr_num + 1) * tick_for_onemsr;
        let beat_num = tick_for_onemsr / tick_for_onebeat;
        let mut chord_map: Vec<bool> = vec![false; beat_num as usize];
        if self.no_loop {
            return chord_map;
        }
        let mut trace: usize = 0;
        let cmps = self.cmps_dt.to_vec();
        let max_ev: usize = cmps.len();
        loop {
            if max_ev <= trace {
                break;
            }
            let tick = cmps[trace].tick as i32;
            if first_tick <= tick
                && tick < end_tick
                && cmps[trace].tbl != txt2seq_cmps::NO_PED_TBL_NUM as i16
            {
                // Chord Table が "X" で無ければ
                chord_map[((tick % tick_for_onemsr) / tick_for_onebeat) as usize] = true;
            } else if tick > end_tick {
                break;
            }
            trace += 1;
        }
        chord_map
    }
    fn generate_event(
        &mut self,
        _crnt_: &CrntMsrTick,
        _estk: &mut ElapseStack,
        elapsed_tick: i32,
    ) -> i32 {
        let mut trace: usize = self.play_counter;
        let mut next_tick: i32;
        let cmps = self.cmps_dt.to_vec();
        loop {
            if cmps.len() <= trace {
                next_tick = END_OF_DATA; // means sequence finished
                break;
            }
            next_tick = cmps[trace].tick as i32;
            if next_tick <= elapsed_tick {
                let cd = cmps[trace].clone();
                if cd.mtype == TYPE_CONTROL {
                    if cd.tbl == NO_LOOP {
                        _estk.set_loop_end(self.id.pid as usize);
                        self.no_loop = true;
                    }
                } else if cd.mtype == TYPE_CHORD {
                    self.prepare_note_translation(cd, _estk);
                } else if cd.mtype == TYPE_VARI {
                    _estk.set_phrase_vari(self.id.pid as usize, cd.root as usize);
                }
            } else {
                break;
            }
            trace += 1;
        }
        self.play_counter = trace;
        next_tick
    }
    fn prepare_note_translation(&mut self, cd: ChordEvt, _estk: &mut ElapseStack) {
        self.root = cd.root;
        self.translation_tbl = cd.tbl;

        let tbl_num: usize = self.translation_tbl as usize;
        let tbl_name = txt2seq_cmps::get_table_name(tbl_num);
        let cname = tbl_name.to_string();
        if cname.chars().nth(0).unwrap_or(' ') == '_' {
            let root_index = ((self.root - 1) / 3) as usize;
            let alteration = (self.root + 1) % 3;
            let mut root = txt2seq_cmps::get_root_name(root_index).to_string();
            if alteration == 1 {
                root += "#";
            } else if alteration == 2 {
                root += "b";
            }
            self.chord_name = root.to_string() + &cname[1..];
        } else {
            self.chord_name = cname;
        }
        if self.id.pid == FLOW_PART as u32 {
            // MIDI Out (keynoteも一緒に送る)
            _estk.midi_out_ext(0xa0, 0x7f, self.keynote);
            _estk.midi_out_ext(0xa0, cd.root as u8, cd.tbl as u8);
            #[cfg(feature = "verbose")]
            println!(
                "Flow Chord Data: {}, {}, {}",
                self.chord_name, cd.root, cd.tbl
            );
        } else {
            #[cfg(feature = "verbose")]
            println!("Chord Data: {}, {}, {}", self.chord_name, cd.root, cd.tbl);
        }
    }
    fn _reset_note_translation(&mut self) { /*<<DoItLater>>*/
    }
}
impl Elapse for CompositionLoop {
    /// id を得る
    fn id(&self) -> ElapseId {
        self.id
    }
    /// priority を得る
    fn prio(&self) -> u32 {
        self.priority
    }
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32) {
        (self.next_msr, self.next_tick)
    }
    /// User による start/play 時にコールされる
    fn start(&mut self, _msr: i32) {
        self.just_after_start = true;
    }
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {
        self.set_destroy();
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        CompositionLoop::new(
            self.id.sid,
            self.id.pid,
            self.keynote,
            self.first_msr_num,
            Vec::new(),
            0,
        );
        self.next_msr = FULL;
        self.destroy = true;
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {
        self.destroy()
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.destroy {
            return;
        }

        //  現在の tick を 1tick 後ろにずらす（Play直後以外）
        let mut cm_crnt = *crnt_;
        if !self.just_after_start {
            if cm_crnt.tick == crnt_.tick_for_onemsr - 1 {
                cm_crnt.msr += 1;
                cm_crnt.tick = 0;
            } else {
                cm_crnt.tick += 1;
            }
        } else {
            self.just_after_start = false;
        }

        //  経過 tick の算出
        let elapsed_tick = self.calc_serial_tick(&cm_crnt);
        if elapsed_tick >= self.whole_tick {
            // =をつけないと、loop終了直後の小節頭で無限ループになる
            self.next_msr = FULL;
            self.destroy = true;
        } else if !self.already_end && (elapsed_tick >= self.next_tick_in_cmps) {
            let next_tick = self.generate_event(&cm_crnt, estk, elapsed_tick);
            if next_tick == END_OF_DATA {
                self.already_end = true;
                self.next_tick_in_cmps = self.whole_tick;
            } else {
                self.next_tick_in_cmps = next_tick;
            }

            // 次回 msr, tick の算出
            let (nxtmsr, nxttick) = self.gen_msr_tick(&cm_crnt, self.next_tick_in_cmps);
            // next_tick を 1tick 前に設定
            if nxttick == 0 {
                self.next_msr = nxtmsr - 1;
                self.next_tick = crnt_.tick_for_onemsr - 1;
            } else {
                self.next_msr = nxtmsr;
                self.next_tick = nxttick - 1;
            }
        }
    }
}
impl Loop for CompositionLoop {
    fn destroy(&self) -> bool {
        self.destroy
    }
    fn set_destroy(&mut self) {
        self.next_tick = 0;
        self.next_msr = FULL;
        self.destroy = true;
    }
    fn first_msr_num(&self) -> i32 {
        self.first_msr_num
    }
    /// Loopの途中から再生するための小節数を設定
    fn set_forward(&mut self, crnt_: &CrntMsrTick, elapsed_msr: i32) {
        let elapsed_tick = elapsed_msr * crnt_.tick_for_onemsr;
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let cmps = self.cmps_dt.to_vec();
        let max_ev = self.cmps_dt.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA; // means sequence finished
                break;
            }
            next_tick = cmps[trace].tick as i32;
            if next_tick >= elapsed_tick {
                break;
            }
            trace += 1;
        }
        self.play_counter = trace;
        self.next_tick_in_cmps = next_tick;
        let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_cmps);
        // next_tick を 1tick 前に設定
        if tick == 0 {
            self.next_msr = msr - 1;
            self.next_tick = crnt_.tick_for_onemsr - 1;
        } else {
            self.next_msr = msr;
            self.next_tick = tick - 1;
        }
    }
}
