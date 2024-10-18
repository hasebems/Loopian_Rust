//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse::*;
use super::elapse_note::Note;
use super::elapse_pattern::DynamicPattern;
use super::note_translation::*;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::txt2seq_cmps::{self, NO_LOOP};
use crate::lpnlib::*;

//*******************************************************************
//          Loop Struct
//*******************************************************************
pub trait Loop: Elapse {
    fn destroy(&self) -> bool;
    fn set_destroy(&mut self);
    fn first_msr_num(&self) -> i32;
    fn calc_serial_tick(&self, crnt_: &CrntMsrTick) -> i32 {
        (crnt_.msr - self.first_msr_num()) * crnt_.tick_for_onemsr + crnt_.tick
    }
    fn gen_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32) {
        let tick = srtick % crnt_.tick_for_onemsr;
        let msr = self.first_msr_num() + srtick / crnt_.tick_for_onemsr;
        (msr, tick)
    }
}
//*******************************************************************
//          Phrase Loop Struct
//*******************************************************************
pub struct PhraseLoop {
    id: ElapseId,
    priority: u32,

    phrase: Vec<PhrEvt>,
    analys: Vec<AnaEvt>,
    keynote: u8,
    play_counter: usize,
    next_tick_in_phrase: i32,
    last_note: i16,
    noped: bool,
    turnnote: i16,
    para_root_base: i16,
    same_note_stuck: Vec<i16>,
    same_note_msr: i32,
    same_note_tick: i32,
    staccato_rate: i32,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl PhraseLoop {
    pub fn new(
        sid: u32,
        pid: u32,
        keynote: u8,
        msr: i32,
        phr: Vec<PhrEvt>,
        ana: Vec<AnaEvt>,
        whole_tick: i32,
        turnnote: i16,
    ) -> Rc<RefCell<Self>> {
        let noped = ana
            .clone()
            .iter()
            .any(|x| x.mtype == TYPE_EXP && x.atype == NOPED);
        let mut para_root_base = 0;
        ana.iter().for_each(|x| {
            if x.mtype == TYPE_EXP && x.atype == PARA_ROOT {
                para_root_base = x.note;
            }
        });
        let mut staccato_rate = 100;
        ana.iter().for_each(|x| {
            if x.mtype == TYPE_EXP && x.atype == ARTIC {
                staccato_rate = x.cnt as i32;
            }
        });
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpPhraseLoop,
            },
            priority: PRI_PHR_LOOP,
            phrase: phr,
            analys: ana,
            keynote,
            play_counter: 0,
            next_tick_in_phrase: 0,
            last_note: NO_NOTE as i16,
            noped,
            turnnote,
            para_root_base,
            same_note_stuck: Vec::new(),
            same_note_msr: 0,
            same_note_tick: 0,
            staccato_rate,
            // for super's member
            whole_tick,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,
            next_tick: 0,
        }))
    }
    pub fn get_noped(&self) -> bool {
        self.noped
    }
    fn generate_event(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        elapsed_tick: i32,
    ) -> i32 {
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let phr = self.phrase.to_vec();
        let max_ev = self.phrase.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA; // means sequence finished
                break;
            }
            next_tick = phr[trace].tick as i32;
            if next_tick <= elapsed_tick {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                let tp = self.phrase[trace].mtype;
                if tp == TYPE_NOTE {
                    if self.same_note_msr != msr || self.same_note_tick != tick {
                        // 設定されているタイミングが少しでも違えば、同タイミング重複音検出をクリア
                        self.same_note_stuck = Vec::new();
                        self.same_note_msr = msr;
                        self.same_note_tick = tick;
                    }
                    self.note_event(estk, trace, phr[trace].clone(), next_tick, msr, tick);
                } else if tp == TYPE_CLS || tp == TYPE_ARP {
                    let ptn: Rc<RefCell<dyn Elapse>> = DynamicPattern::new(
                        crnt_.msr as u32, //  read pointer
                        self.id.sid,      //  loop.sid -> note.pid
                        self.id.pid,      //  part
                        self.keynote,
                        msr,
                        self.phrase[trace].clone(),
                        self.analys.to_vec(),
                    );
                    estk.add_elapse(Rc::clone(&ptn));
                }
            } else {
                break;
            }
            trace += 1;
        }

        self.play_counter = trace;
        next_tick
    }
    fn note_event(
        &mut self,
        estk: &mut ElapseStack,
        trace: usize,
        ev: PhrEvt,
        next_tick: i32,
        msr: i32,
        tick: i32,
    ) {
        // ev: ['note', tick, duration, note, velocity]
        let mut crnt_ev = ev.clone();
        let mut deb_txt: String = "no chord".to_string();
        let (mut rt, mut ctbl) = (NO_ROOT, NO_TABLE);
        if let Some(cmps) = estk.get_cmps(self.id.pid as usize) {
            (rt, ctbl) = cmps.borrow().get_chord();
        }

        //  Note Translation
        if rt != NO_ROOT || ctbl != NO_TABLE {
            (crnt_ev.note, deb_txt) = self.translate_note(rt, ctbl, ev, next_tick);
        }

        //  同タイミング重複音を鳴らさない
        if self.same_note_stuck.iter().any(|x| *x == crnt_ev.note) {
            return;
        } else {
            self.same_note_stuck.push(crnt_ev.note);
        }

        //  Generate Note Struct
        if self.staccato_rate != 100 {
            let old = crnt_ev.dur as i32;
            crnt_ev.dur = ((old * self.staccato_rate) / 100) as i16;
        }
        let nt: Rc<RefCell<dyn Elapse>> = Note::new(
            trace as u32, //  read pointer
            self.id.sid,  //  loop.sid -> note.pid
            estk,
            &crnt_ev,
            self.keynote,
            deb_txt + &format!(" / Pt:{} Lp:{}", &self.id.pid, &self.id.sid),
            msr,
            tick,
            self.id.pid,
        );
        estk.add_elapse(Rc::clone(&nt));
    }
    fn translate_note(&mut self, rt: i16, ctbl: i16, ev: PhrEvt, next_tick: i32) -> (i16, String) {
        let deb_txt: String;
        let trans_note: i16;
        let root: i16 = ROOT2NTNUM[rt as usize];
        let (movable_scale, mut para_note) = txt2seq_cmps::is_movable_scale(ctbl, root);
        if movable_scale {
            if para_note > self.turnnote {
                para_note -= 12;
            }
            trans_note = translate_note_parascl(para_note, ctbl, ev.note);
            deb_txt = "para_sc:".to_string();
        } else {
            let option = self.specify_trans_option(next_tick, ev.note);
            if option == TRNS_PARA {
                let para_root = root - self.para_root_base;
                let mut tgt_nt = ev.note + para_root;
                if root > self.turnnote {
                    tgt_nt -= 12;
                }
                trans_note = translate_note_com(root, ctbl, tgt_nt);
                deb_txt = "para:".to_string();
            } else if option == TRNS_COM {
                trans_note = translate_note_com(root, ctbl, ev.note);
                deb_txt = "com:".to_string();
            } else if option == TRNS_NONE {
                trans_note = ev.note;
                deb_txt = "none:".to_string();
            } else {
                // Arpeggio
                //trans_note = NoteTranslation::translate_note_arp(root, ctbl, option);
                trans_note = translate_note_arp2(root, ctbl, ev.note, option, self.last_note);
                deb_txt = "arp:".to_string();
            }
        }
        self.last_note = trans_note;
        //crnt_ev[NOTE] = trans_note;
        (
            trans_note,
            deb_txt + &(root.to_string() + "-" + &ctbl.to_string()),
        )
    }
    fn specify_trans_option(&self, next_tick: i32, note: i16) -> i16 {
        for anaone in self.analys.iter() {
            if anaone.mtype == TYPE_BEAT && anaone.tick == next_tick as i16 && anaone.note == note {
                return anaone.atype;
            }
        }
        TRNS_COM
    }
}
impl Elapse for PhraseLoop {
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
    fn start(&mut self, _msr: i32) {} // User による start/play 時にコールされる
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {
        self.set_destroy();
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.phrase = Vec::new();
        self.analys = Vec::new();
        self.play_counter = 0;
        self.next_tick_in_phrase = 0;
        self.last_note = NO_NOTE as i16;
        self.same_note_stuck = Vec::new();
        self.same_note_msr = 0;
        self.same_note_tick = 0;
        self.next_msr = 0;
        self.next_tick = 0;
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

        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = FULL;
            self.destroy = true;
            return;
        }

        if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            self.next_tick_in_phrase = next_tick;
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            } else {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                self.next_msr = msr;
                self.next_tick = tick;
            }
        }
    }
}
impl Loop for PhraseLoop {
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
}

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
        let mut cm_crnt = crnt_.clone();
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
}
