//  Created by Hasebe Masahiko on 2025/05/03
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::elapse_part::PartBasicPrm;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::txt2seq_cmps::*;
use crate::lpnlib::*;

//*******************************************************************
//          Composition Loop Struct
//*******************************************************************
#[derive(Clone, Debug)]
pub struct UnfoldedComposition {
    keynote: u8,
    cmps_map: Vec<Vec<Vec<(ChordEvt, bool)>>>, // [msr][beat][num]

    // for Composition
    first_msr_num: i32,
    part_num: u32,
    chord_name: String,
    root: i16,
    translation_tbl: i16,
    vari_num: i16,
    //whole_tick: i32,
    max_msr: usize,
    max_beat: usize,
}
impl UnfoldedComposition {
    pub fn new(
        part_num: u32,
        evts: Vec<ChordEvt>,
        keynote: u8,
        whole_tick: i32,
        max_msr: usize,
        max_beat: usize,
    ) -> Self {
        let cmps_map = Self::unfold_cmp_evt(evts, max_msr, max_beat, whole_tick);
        Self {
            keynote,
            cmps_map,
            first_msr_num: 0,
            part_num,
            chord_name: String::new(),
            root: 0,
            translation_tbl: 0,
            vari_num: 0,
            //whole_tick,
            max_msr,
            max_beat,
        }
    }
    fn unfold_cmp_evt(
        evts: Vec<ChordEvt>,
        msr: usize,
        beat: usize,
        whole_tick: i32,
    ) -> Vec<Vec<Vec<(ChordEvt, bool)>>> {
        let mut cmps_map = vec![vec![Vec::new(); beat]; msr];
        let tick_for_onemsr = whole_tick as usize / msr;
        let tick_for_onebeat = tick_for_onemsr / beat;
        //for evt in evts {
        //    let crnt_msr = evt.tick as usize / tick_for_onemsr;
        //    let crnt_beat = (evt.tick as usize - crnt_msr * tick_for_onemsr) / tick_for_onebeat;
        //    cmps_map[crnt_msr][crnt_beat].push(evt);
        //}
        let mut crnt_idx = 0;
        let mut last_evt = None;
        let max_len = evts.len();
        for (i, msr_map) in cmps_map.iter_mut().enumerate() {
            for (j, beat_map) in msr_map.iter_mut().enumerate() {
                if crnt_idx < max_len
                    && evts[crnt_idx].tick <= (i * tick_for_onemsr + j * tick_for_onebeat) as i16
                {
                    beat_map.push((evts[crnt_idx].clone(), true));
                    if evts[crnt_idx].mtype == TYPE_CHORD {
                        last_evt = Some(evts[crnt_idx].clone());
                    }
                    crnt_idx += 1;
                } else if let Some(ref evt) = last_evt {
                    beat_map.push((evt.clone(), false));
                }
            }
        }
        #[cfg(feature = "verbose")]
        {
            println!("cmps_map={:?}", cmps_map);
            println!("Unfolded Composition!");
        }
        cmps_map
    }
    pub fn get_first_msr_num(&self) -> i32 {
        self.first_msr_num
    }
    pub fn set_first_msr_num(&mut self, first_msr_num: i32) {
        self.first_msr_num = first_msr_num;
    }
    pub fn get_vari_num(&self) -> i16 {
        self.vari_num
    }
    pub fn get_chord_name(&self) -> String {
        self.chord_name.clone()
    }
    pub fn loop_msr_beat(&self, crnt_: &CrntMsrTick) -> (i32, i32) {
        let beat = crnt_.tick / (crnt_.tick_for_onemsr / (self.max_beat as i32));
        let msr = crnt_.msr - self.first_msr_num;
        (msr, beat)
    }
    pub fn gen_chord_map(&self, crnt_msr: i32) -> Vec<bool> {
        let cmsr = (crnt_msr - self.first_msr_num) as usize;
        let mut chord_map = vec![false; self.max_beat];
        if self.max_msr > cmsr {
            for (j, chord) in chord_map.iter_mut().enumerate() {
                for evt in &self.cmps_map[cmsr][j] {
                    if evt.1 && evt.0.mtype == TYPE_CHORD && evt.0.tbl != NO_PED_TBL_NUM as i16 {
                        *chord = true;
                    }
                }
            }
        }
        #[cfg(feature = "verbose")]
        println!("<gen_chord_map for Damper> chord_map={:?}", chord_map);
        chord_map
    }
    pub fn gen_vari_num(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        let (msr, beat) = self.loop_msr_beat(crnt_);
        self.cmps_map[msr as usize][beat as usize]
            .iter()
            .for_each(|evt| {
                if evt.0.mtype == TYPE_VARI {
                    if let Some(pt) = estk.part(self.part_num) {
                        pt.borrow_mut().set_phrase_vari(evt.0.root as usize);
                    }
                }
            });
    }
    pub fn scan_chord(&self, crnt_msr: usize, crnt_beat: usize) -> (i16, i16) {
        let mut root: i16 = 0;
        let mut tbl: i16 = NO_PED_TBL_NUM as i16;
        let mut find: bool = false;
        let mut msr = crnt_msr as isize;
        let mut beat = crnt_beat as isize;
        loop {
            if !self.cmps_map[msr as usize][beat as usize].is_empty() {
                for evt in &self.cmps_map[msr as usize][beat as usize] {
                    if evt.0.mtype == TYPE_CHORD {
                        root = evt.0.root;
                        tbl = evt.0.tbl;
                        find = true;
                        break;
                    }
                }
            }
            if !find {
                beat -= 1;
                if beat < 0 {
                    msr -= 1;
                    if msr < 0 {
                        break;
                    }
                    beat = (self.max_beat - 1) as isize;
                }
            } else {
                break;
            }
        }
        (root, tbl)
    }
    pub fn gen_chord_name(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> String {
        let (msr, beat) = self.loop_msr_beat(crnt_);
        self.cmps_map[msr as usize][beat as usize]
            .iter()
            .for_each(|evt| {
                if evt.0.mtype == TYPE_CHORD {
                    self.root = evt.0.root;
                    self.translation_tbl = evt.0.tbl;
                }
            });
        let cd: ChordEvt = ChordEvt {
            mtype: TYPE_CHORD,
            tick: 0,
            root: self.root,
            tbl: self.translation_tbl,
        };
        self.prepare_note_translation(cd, estk);

        let num = self.get_vari_num();
        let num_str = if num == 0 {
            "".to_string()
        } else {
            "@".to_string() + self.get_vari_num().to_string().as_str()
        };
        self.get_chord_name() + &num_str
    }
    fn prepare_note_translation(&mut self, cd: ChordEvt, estk: &mut ElapseStack) {
        // generate chore name 'character' for display
        let tbl_num: usize = self.translation_tbl as usize;
        let tbl_name = get_table_name(tbl_num);
        let cname = tbl_name.to_string();
        if cname.chars().nth(0).unwrap_or(' ') == '_' {
            let root_index = ((self.root - 1) / 3) as usize;
            let alteration = (self.root + 1) % 3;
            let mut root = get_root_name(root_index).to_string();
            if alteration == 1 {
                root += "#";
            } else if alteration == 2 {
                root += "b";
            }
            self.chord_name = root.to_string() + &cname[1..];
        } else {
            self.chord_name = cname;
        }

        if self.part_num == FLOW_PART as u32 {
            // MIDI Out (keynoteも一緒に送る)
            estk.midi_out_ext(0xa0, 0x7f, self.keynote);
            estk.midi_out_ext(0xa0, cd.root as u8, cd.tbl as u8);
            #[cfg(feature = "verbose")]
            println!(
                "Flow Chord Data: {}, {}, {}",
                self.chord_name, cd.root, cd.tbl
            );
        } else {
            //#[cfg(feature = "verbose")]
            //println!("Chord Data: {}, {}, {}", self.chord_name, cd.root, cd.tbl);
        }
    }
}

//*******************************************************************
//          Composition Loop Mediator Struct
//*******************************************************************
#[derive(Clone, Debug)]
pub struct CmpsLoopMediator {
    pub state_reserve: bool,

    part_num: u32,
    first_msr_num: i32,
    whole_tick: i32,
    loop_id: u32, // loop sid
    cmps: Option<Box<UnfoldedComposition>>,
    do_loop: bool,
    keynote: u8,
    max_msr: i32,
    max_beat: i32,

    next_whole_tick: i32,
    next_cmps: Option<Box<UnfoldedComposition>>,
}
impl CmpsLoopMediator {
    pub fn new(part_num: u32) -> Self {
        Self {
            state_reserve: false,
            part_num,
            first_msr_num: 0,
            whole_tick: 0,
            loop_id: 0,
            cmps: None,
            do_loop: true,
            keynote: 0,
            max_msr: 0,
            max_beat: 0,
            next_whole_tick: 0,
            next_cmps: None,
        }
    }
    pub fn _set_keynote(&mut self, keynote: u8) {
        self.keynote = keynote;
    }
    pub fn start(&mut self) {
        self.first_msr_num = 0;
        self.state_reserve = true;
    }
    pub fn msrtop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else if self.max_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else if self.max_msr != 0 && (crnt_.msr - self.first_msr_num) % (self.max_msr) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else if self.max_msr != 0 && pbp.sync_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else {
                // 現在の Loop Obj が終了していない時
                // 現在の Phrase より新しい Phrase の whole_tick が大きい場合、
                // 新しい Phrase に更新する
                if self.next_whole_tick >= self.whole_tick {
                    self.state_reserve = false;
                    self.proc_forward_cmps_by_evt(estk);
                }
            }
        } else if self.max_msr != 0 && (crnt_.msr - self.first_msr_num) % self.max_msr == 0 {
            // Loop Obj が終了した時
            if self.do_loop {
                // 同じ Loop.Obj を再生する
                self.stay_loop(crnt_);
            } else {
                self.clear_cmp_prm();
            }
        } else if let Some(cmp) = &mut self.cmps {
            // Loop Obj が終了していない時、variation を再生する
            cmp.gen_vari_num(crnt_, estk);
        }
    }
    /// Composition Event を受け取り、UnfoldedComposition を生成する
    pub fn rcv_cmp(&mut self, msg: ChordData, tick_for_onemsr: i32, tick_for_onebeat: i32) {
        if msg.evts.is_empty() && msg.whole_tick == 0 {
            self.next_cmps = None;
        } else {
            let max_msr = (msg.whole_tick as i32 / tick_for_onemsr) as usize;
            let max_beat = (tick_for_onemsr / tick_for_onebeat) as usize;
            self.next_cmps = Some(Box::new(UnfoldedComposition::new(
                self.part_num,
                msg.evts,
                self.keynote, // 現在のCompositionのKeynote
                msg.whole_tick as i32,
                max_msr,
                max_beat,
            )));
        }
        #[cfg(feature = "verbose")]
        println!(
            "Received next_cmps >next_cmps is {:?}",
            self.next_cmps.is_some()
        );
        self.do_loop = msg.do_loop;
        self.state_reserve = true;
        self.next_whole_tick = msg.whole_tick as i32;
    }
    fn clear_cmp_prm(&mut self) {
        self.first_msr_num = 0;
        self.max_msr = 0;
        self.max_beat = 0;
        self.whole_tick = 0;
        self.cmps = None;
        self.next_cmps = None;
        self.do_loop = true;
    }
    /// 新たに Loop Obj.を生成
    fn new_loop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.next_cmps.is_some() {
            #[cfg(feature = "verbose")]
            println!("New Composition Loop! M:{:?},T:{:?}", crnt_.msr, crnt_.tick);
            self.first_msr_num = crnt_.msr; // 計測開始の更新
            self.whole_tick = self.next_whole_tick;
            let (tick_for_onemsr, tick_for_beat) = estk.tg().get_beat_tick();
            (self.max_msr, self.max_beat) =
                self.calc_msr_beat(self.whole_tick, tick_for_onemsr, tick_for_beat);

            if self.whole_tick == 0 {
                self.state_reserve = true; // 次小節冒頭で呼ばれるように
                self.cmps = None;
            } else {
                self.loop_id += 1;
                self.cmps = self.next_cmps.take();
                if let Some(ref mut cmp) = self.cmps {
                    cmp.set_first_msr_num(self.first_msr_num);
                }
            }
        } else {
            // 新しい Composition が空のとき
            self.max_msr = 0;
            self.max_beat = 0;
            self.whole_tick = 0;
            self.state_reserve = true;
            self.cmps = None;
        }
    }
    fn stay_loop(&mut self, crnt_: &CrntMsrTick) {
        self.first_msr_num = crnt_.msr;
        if let Some(ref mut cmp) = self.cmps {
            cmp.set_first_msr_num(self.first_msr_num);
        }
    }
    /// 新しい Phrase を早送りして更新する
    fn proc_forward_cmps_by_evt(&mut self, estk: &mut ElapseStack) {
        // その時の beat 情報で、whole_tick を loop_measure に換算
        self.whole_tick = self.next_whole_tick;
        let (tick_for_onemsr, tick_for_beat) = estk.tg().get_beat_tick();
        (self.max_msr, self.max_beat) =
            self.calc_msr_beat(self.whole_tick, tick_for_onemsr, tick_for_beat);

        // Composition の更新
        self.loop_id += 1;
        let mut fmsrnum = 0;
        if let Some(ref mut cmp) = self.cmps {
            fmsrnum = cmp.get_first_msr_num();
        }
        self.cmps = self.next_cmps.take();
        if let Some(ref mut cmp) = self.cmps {
            cmp.set_first_msr_num(fmsrnum);
        }
        #[cfg(feature = "verbose")]
        println!(
            "Replace Composition Loop! --whole tick: {}",
            self.whole_tick
        );
    }
    fn calc_msr_beat(
        &mut self,
        whole_tick: i32,
        tick_for_onemsr: i32,
        tick_for_beat: i32,
    ) -> (i32, i32) {
        // その時の beat 情報で、whole_tick を loop_measure に換算
        let plus_one = if whole_tick % tick_for_onemsr == 0 {
            0
        } else {
            1
        };
        (
            self.whole_tick / tick_for_onemsr + plus_one,
            tick_for_onemsr / tick_for_beat,
        )
    }
    //pub fn get_cmps(&self, crnt_: &CrntMsrTick) -> Option<Box<UnfoldedComposition>> {
    //self.cmps.as_ref().map(|cmps| (*cmps).clone())
    //    self.cmps.as_ref().cloned()
    //}

    /// 一度 Mediator（仲介者）を通してから、UnfoldedComposition のサービスを利用する
    /// Not Yet: cmps か cmps_next のどちらかを選択する
    pub fn get_chord_map(&self, crnt_: &CrntMsrTick) -> Vec<bool> {
        if let Some(ref cmp) = self.cmps {
            cmp.gen_chord_map(crnt_.msr)
        } else {
            let beat = if self.max_beat == 0 {
                (crnt_.tick_for_onemsr / DEFAULT_TICK_FOR_QUARTER) as usize
            } else {
                self.max_beat as usize
            };
            vec![false; beat]
        }
    }
    pub fn get_chord(&self, crnt_: &CrntMsrTick) -> (i16, i16) {
        if let Some(ref cmp) = self.cmps {
            let (msr, beat) = cmp.loop_msr_beat(crnt_);
            cmp.scan_chord(msr as usize, beat as usize)
        } else {
            (0, NO_PED_TBL_NUM as i16)
        }
    }
    pub fn get_chord_name(&self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> String {
        self.cmps
            .as_ref()
            .map(|cmp| {
                let mut cmp_owned = cmp.clone();
                cmp_owned.gen_chord_name(crnt_, estk)
            })
            .unwrap_or("-".to_string())
    }
}
