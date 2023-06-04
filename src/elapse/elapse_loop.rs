//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib::*;
use super::elapse::*;
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;
use super::elapse_note::{Note, Damper};
use crate::cmd::txt2seq_cmps;
use super::note_translation::*;

//*******************************************************************
//          Loop Struct
//*******************************************************************
pub trait Loop: Elapse {
    fn destroy(&self) -> bool;
    fn set_destroy(&mut self);
    fn first_msr_num(&self) -> i32;
    fn calc_serial_tick(&self, crnt_: &CrntMsrTick) -> i32 {
        (crnt_.msr - self.first_msr_num())*crnt_.tick_for_onemsr + crnt_.tick
    }
    fn gen_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32) {
        let tick = srtick%crnt_.tick_for_onemsr;
        let msr = self.first_msr_num() + srtick/crnt_.tick_for_onemsr;
        (msr, tick)
    }
}

//*******************************************************************
//          Phrase Loop Struct
//*******************************************************************
pub struct PhraseLoop {
    id: ElapseId,
    priority: u32,

    phrase_dt: Vec<Vec<i16>>,
    analys_dt: Vec<Vec<i16>>,
    keynote: u8,
    play_counter: usize,
    next_tick_in_phrase: i32,
    last_note: i16,
    noped: bool,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
}
impl PhraseLoop {
    pub fn new(sid: u32, pid: u32, keynote: u8, msr: i32, msg: Vec<Vec<i16>>, ana: Vec<Vec<i16>>, whole_tick: i32) 
      -> Rc<RefCell<Self>> {
        let noped = ana.iter().any(|x| x[TYPE]==TYPE_EXP && x[EXP]==NOPED);
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpPhraseLoop,},
            priority: PRI_PHR_LOOP,
            phrase_dt: msg,
            analys_dt: ana,
            keynote,
            play_counter: 0,
            next_tick_in_phrase: 0,
            last_note: NO_NOTE as i16,
            noped,
            // for super's member
            whole_tick,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,
            next_tick: 0,
        }))
    }
    pub fn get_noped(&self) -> bool {self.noped}
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let phr = self.phrase_dt.to_vec();
        let max_ev = self.phrase_dt.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA;   // means sequence finished
                break;
            }
            next_tick = phr[trace][TICK] as i32;
            if next_tick <= elapsed_tick {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                if self.phrase_dt[trace][TYPE] == TYPE_NOTE {
                    self.note_event(estk, trace, phr[trace].clone(), next_tick, msr, tick);
                }
            }
            else {break;}
            trace += 1;
        }

        self.play_counter = trace;
        next_tick
    }
    const ROOT2NTNUM: [i16; 22] = [0,-1,0,1,1,2,3,3,4,5,4,5,6,6,7,8,8,9,10,10,11,12,];
    fn note_event(&mut self, estk: &mut ElapseStack, trace: usize, ev: Vec<i16>, next_tick: i32, msr: i32, tick: i32) {
        // ev: ['note', tick, duration, note, velocity]
        let mut crnt_ev = ev.clone();
        let mut deb_txt: String = "no chord".to_string();
        let (mut rt, mut ctbl) = (NO_ROOT, NO_TABLE);
        if let Some(cmps) = estk.get_cmps(self.id.pid as usize) {
            (rt, ctbl) = cmps.borrow().get_chord();
        }

        if rt != NO_ROOT || ctbl != NO_TABLE  {
            let trans_note: i16;
            let root: i16 = Self::ROOT2NTNUM[rt as usize];
            let (movable_scale, para_note) = txt2seq_cmps::is_movable_scale(ctbl, root);
            if  movable_scale {
                trans_note = translate_note_parascl(para_note, ctbl, ev[NOTE]);
                deb_txt = "para_sc:".to_string();
            }
            else {
                let option = self.specify_trans_option(next_tick, ev[NOTE]);
                if option == ARP_PARA {
                    let mut tgt_nt = ev[NOTE] + root;
                    if root > 5 {tgt_nt -= 12;}
                    trans_note = translate_note_com(root, ctbl, tgt_nt);
                    deb_txt = "para:".to_string();
                }
                else if option == ARP_COM {
                    trans_note = translate_note_com(root, ctbl, ev[NOTE]);
                    deb_txt = "com:".to_string();
                }
                else { // Arpeggio
                    //trans_note = NoteTranslation::translate_note_arp(root, ctbl, option);
                    trans_note = translate_note_arp2(root, ctbl, ev[NOTE], option, self.last_note);
                    deb_txt = "arp:".to_string();
                }
            }
            self.last_note = trans_note;
            crnt_ev[NOTE] = trans_note;
            deb_txt += &(root.to_string() + "-" + &ctbl.to_string());
        }

        let nt: Rc<RefCell<dyn Elapse>> = Note::new(
            trace as u32,   //  read pointer
            self.id.sid,    //  loop.sid -> note.pid
            estk,
            &crnt_ev,
            self.keynote,
            deb_txt,
            msr,
            tick);
        estk.add_elapse(Rc::clone(&nt));
    }
    fn specify_trans_option(&self, next_tick: i32, note: i16) -> i16 {
        for anaone in self.analys_dt.iter() {
            if anaone[TYPE] == TYPE_BEAT &&
               anaone[TICK] == next_tick as i16 && 
               anaone[NOTE] == note {
                return anaone[ARP_DIFF];
            }
        }
        ARP_COM
    }
}
impl Elapse for PhraseLoop {
    fn id(&self) -> ElapseId {self.id}     // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {}      // User による start/play 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {} // User による stop 時にコールされる
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {self.destroy()}   // 自クラスが役割を終えた時に True を返す
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        if self.destroy {return;}

        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = FULL;
            self.destroy = true;
            return
        }

        if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            self.next_tick_in_phrase = next_tick;
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            }
            else {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                self.next_msr = msr;
                self.next_tick = tick;
            }
        }
    }
}
impl Loop for PhraseLoop {
    fn destroy(&self) -> bool {self.destroy}
    fn set_destroy(&mut self) {
        self.next_tick = 0;
        self.next_msr = FULL;
        self.destroy = true;
    }
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}

//*******************************************************************
//          Composition Loop Struct
//*******************************************************************
pub struct CompositionLoop {
    id: ElapseId,
    priority: u32,

    cmps_dt: Vec<Vec<i16>>,
    _keynote: u8,
    play_counter: usize,
    next_tick_in_cmps: i32,
    // for Composition
    chord_name: String,
    root: i16,
    translation_tbl: i16,
    already_end: bool,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
}
impl CompositionLoop {
    pub fn new(sid: u32, pid: u32, knt:u8, msr: i32, msg: Vec<Vec<i16>>, whole_tick: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpCompositionLoop,},
            priority: PRI_CMPS_LOOP,
            cmps_dt: msg,
            _keynote: knt,
            play_counter: 0,
            next_tick_in_cmps: 0,

            chord_name: "".to_string(),
            root: NO_ROOT,
            translation_tbl: NO_TABLE,
            already_end: false,

            // for super's member
            whole_tick,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,   //   次に呼ばれる小節番号が保持される
            next_tick: 0,
        }))
    }
    pub fn get_chord(&self) -> (i16, i16) {(self.root, self.translation_tbl)}
    pub fn get_chord_name(&self) -> String {self.chord_name.clone()}
    pub fn get_chord_map(&self, msr: i32, tick_for_onemsr: i32, tick_for_onebeat: i32) -> Vec<bool> { // for Damper
        let first_tick = (msr - self.first_msr_num)*tick_for_onemsr;
        let end_tick = (msr - self.first_msr_num + 1)*tick_for_onemsr;
        let beat_num = tick_for_onemsr/tick_for_onebeat;
        let mut trace: usize = 0;
        let cmps = self.cmps_dt.to_vec();
        let mut chord_map: Vec<bool> = vec![false; beat_num as usize];
        let max_ev: usize = cmps.len();
        loop {
            if max_ev <= trace {break}
            let tick = cmps[trace][TICK] as i32;
            if first_tick <= tick && tick < end_tick && cmps[trace][CD_TABLE] != 0 {
                // Chord Table が "thru" で無ければ
                chord_map[((tick%tick_for_onemsr)/tick_for_onebeat) as usize] = true;
            }
            else if tick > end_tick {break;}
            trace += 1;
        }
        chord_map
    }
    fn _reset_note_translation(&mut self) {/*<<DoItLater>>*/}
    fn prepare_note_translation(&mut self, cd: Vec<i16>) {
        if cd[TYPE] == TYPE_CHORD {
            self.root = cd[CD_ROOT];
            self.translation_tbl = cd[CD_TABLE];

            let tbl_num: usize = self.translation_tbl as usize;
            let tbl_name = crate::cmd::txt2seq_cmps::get_table_name(tbl_num);
            let cname = tbl_name.to_string();
            if cname.chars().nth(0).unwrap_or(' ') == '_' {
                let root_index = ((self.root-1)/3) as usize;
                let alteration = (self.root+1)%3;
                let mut root = crate::cmd::txt2seq_cmps::get_root_name(root_index).to_string();
                if alteration == 1 {root += "#";}
                else if alteration == 2 {root += "b";}
                self.chord_name = root.to_string() + &cname[1..];
            }
            else {
                self.chord_name = cname;
            }
            println!("Chord Data: {}, {}, {}",self.chord_name, cd[CD_ROOT], cd[CD_TABLE]);
        }
    }
    fn generate_event(&mut self, _crnt_: &CrntMsrTick, _estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut trace: usize = self.play_counter;
        let mut next_tick: i32;
        let cmps = self.cmps_dt.to_vec();
        loop {
            if cmps.len() <= trace {
                next_tick = END_OF_DATA;   // means sequence finished
                break
            }
            next_tick = cmps[trace][TICK] as i32;
            if next_tick <= elapsed_tick {
                self.prepare_note_translation(cmps[trace].clone());
            }
            else {break;}
            trace += 1;
        }
        self.play_counter = trace;
        next_tick
    }
}
impl Elapse for CompositionLoop {
    fn id(&self) -> ElapseId {self.id}     // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {}    // User による start/play 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {} // User による stop 時にコールされる
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {self.destroy()}   // 自クラスが役割を終えた時に True を返す
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        if self.destroy {return;}

        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick >= self.whole_tick { // =をつけないと、loop終了直後の小節頭で無限ループになる
            self.next_msr = FULL;
            self.destroy = true;
            return
        }

        if !self.already_end && elapsed_tick >= self.next_tick_in_cmps {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            if next_tick == END_OF_DATA {
                self.already_end = true;
                self.next_tick_in_cmps = self.whole_tick;
            }
            else {
                self.next_tick_in_cmps = next_tick;
            }
            let (next_msr, next_tick) = self.gen_msr_tick(crnt_, self.next_tick_in_cmps);
            self.next_msr = next_msr;
            self.next_tick = next_tick;
        }
        //assert!(self.next_msr > crnt_.msr);
    }
}
impl Loop for CompositionLoop {
    fn destroy(&self) -> bool {self.destroy}
    fn set_destroy(&mut self) {
        self.next_tick = 0;
        self.next_msr = FULL;
        self.destroy = true;
    }
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}

//*******************************************************************
//          Damper Loop Struct
//*******************************************************************
pub struct DamperLoop {
    id: ElapseId,
    priority: u32,

    evt: Vec<Vec<i16>>,
    play_counter: usize,
    next_tick_in_phrase: i32,
    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
}
impl DamperLoop {
    pub fn new(sid: u32, pid: u32, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpDamperLoop,},
            priority: PRI_CMPS_LOOP,
            evt: Vec::new(),
            play_counter: 0,
            next_tick_in_phrase: 0,
            // for super's member
            whole_tick: 0,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,   //   次に呼ばれる小節番号が保持される
            next_tick: 0,
        }))
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let evt = self.evt.to_vec();
        let max_ev = self.evt.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA;   // means sequence finished
                break;
            }
            next_tick = evt[trace][TICK] as i32;
            if next_tick <= elapsed_tick {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                if evt[trace][TYPE] == TYPE_DAMPER {
                    let dmpr: Rc<RefCell<dyn Elapse>> = Damper::new(
                        trace as u32,   //  read pointer
                        self.id.sid,    //  loop.sid -> note.pid
                        estk,
                        &evt[trace],
                        msr,
                        tick);
                    estk.add_elapse(Rc::clone(&dmpr));
                }
            }
            else {break;}
            trace += 1;
        }

        self.play_counter = trace;
        next_tick
    }
    fn make_events_in_msr(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        // 再生 msr/tick に達したらコールされる
        let (tick_for_onemsr, tick_for_onebeat) = estk.tg().get_beat_tick();
        let beat_num: usize = (tick_for_onemsr/tick_for_onebeat) as usize;
        let mut chord_map = vec![false; beat_num];
        for i in 0..MAX_USER_PART {
            if let Some(phr) = estk.get_phr(i) {
                if phr.borrow().get_noped() {
                    chord_map = vec![false; beat_num];
                    break;
                }
            }
            else {continue;}
            if let Some(cmps) = estk.get_cmps(i) {
                let ba = cmps.borrow().get_chord_map(crnt_.msr, tick_for_onemsr, tick_for_onebeat);
                for (i, x) in chord_map.iter_mut().enumerate() {*x |= ba[i];}
            }
        }
        //println!("@@@@ Damper Map:{:?}",chord_map);
        let mut keep: usize = beat_num;
        let mut dmpr_evt: Vec<Vec<i16>> = Vec::new();
        const PDL_MARGIN_TICK: i32 = 60;
        for (j, k) in chord_map.iter().enumerate() {
            if *k {
                if keep != beat_num {
                    dmpr_evt.push(vec![
                        TYPE_DAMPER, 
                        ((keep as i32)*tick_for_onebeat+PDL_MARGIN_TICK) as i16,
                        (((j-keep) as i32)*tick_for_onebeat-PDL_MARGIN_TICK) as i16,
                        127]);
                }
                keep = j;
            }
        }
        if keep != beat_num {
            dmpr_evt.push(vec![
                TYPE_DAMPER, 
                ((keep as i32)*tick_for_onebeat+PDL_MARGIN_TICK) as i16,
                (((beat_num-keep) as i32)*tick_for_onebeat-PDL_MARGIN_TICK) as i16,
                127]);
        }
        //println!("@@@@ Damper Event:{:?}",dmpr_evt);
        self.evt = dmpr_evt;
        self.whole_tick = tick_for_onemsr;
        self.next_tick_in_phrase = 0;
    }
}
impl Elapse for DamperLoop {
    fn id(&self) -> ElapseId {self.id}     // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {self.first_msr_num = 0;}    // User による start/play 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {} // User による stop 時にコールされる
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {self.destroy()}   // 自クラスが役割を終えた時に True を返す
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        if self.destroy {return;}

        if self.next_tick_in_phrase == 0 {
            self.make_events_in_msr(crnt_, estk);
        }

        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = FULL;
            self.destroy = true;
            return
        }

        if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            self.next_tick_in_phrase = next_tick;
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            }
            else {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                self.next_msr = msr;
                self.next_tick = tick;
            }
        }
    }
}
impl Loop for DamperLoop {
    fn destroy(&self) -> bool {self.destroy}
    fn set_destroy(&mut self) {
        self.next_tick = 0;
        self.next_msr = FULL;
        self.destroy = true;
    }
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}