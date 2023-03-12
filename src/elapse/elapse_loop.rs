//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib;
use super::elapse::*;
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;
use super::elapse_note::Note;
use crate::cmd::txt2seq_cmps;

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
    last_note: u8,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
}
impl Elapse for PhraseLoop {
    fn id(&self) -> ElapseId {self.id}     // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {}      // User による start/play 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {} // User による stop 時にコールされる
    fn fine(&mut self, _estk: &mut ElapseStack) {} // User による fine があった次の小節先頭でコールされる
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {self.destroy()}   // 自クラスが役割を終えた時に True を返す
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = lpnlib::FULL;
            self.destroy = true;
            return
        }

        if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            self.next_tick_in_phrase = next_tick;
            if next_tick == lpnlib::END_OF_DATA {
                self.next_msr = lpnlib::FULL;
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
    fn set_destroy(&mut self) {self.destroy = true;}
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}
impl PhraseLoop {
    pub fn new(sid: u32, pid: u32, keynote: u8, msr: i32, msg: Vec<Vec<i16>>, ana: Vec<Vec<i16>>, whole_tick: i32) 
      -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpPhraseLoop,},
            priority: PRI_PHR_LOOP,
            phrase_dt: msg,
            analys_dt: ana,
            keynote,
            play_counter: 0,
            next_tick_in_phrase: 0,
            last_note: lpnlib::NO_NOTE,
            // for super's member
            whole_tick,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,
            next_tick: 0,
        }))
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let phr = self.phrase_dt.to_vec();
        let max_ev = self.phrase_dt.len();
        loop {
            if max_ev <= trace {
                next_tick = lpnlib::END_OF_DATA;   // means sequence finished
                break;
            }
            next_tick = phr[trace][lpnlib::TICK] as i32;
            if next_tick <= elapsed_tick {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                if phr[trace][lpnlib::TYPE] == lpnlib::TYPE_DAMPER {
                    //<<DoItLater>>
                    // phr: ['damper', duration, tick, value]
                    //estk.add_obj(elpn.Damper(self.est, self.md, phr, msr, tick))
                }
                else if self.phrase_dt[trace][lpnlib::TYPE] == lpnlib::TYPE_NOTE {
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
        let (root, ctbl) = estk.get_chord_info(self.id.pid as usize);
        let mut deb_txt: String = "no chord".to_string();

        if root != lpnlib::NO_ROOT || ctbl != lpnlib::NO_TABLE  {
            let option = self.identify_trans_option(next_tick, ev[lpnlib::NOTE]);
            let trans_note: u8;
            let root_nt = Self::ROOT2NTNUM[root as usize];
            if option == lpnlib::ARP_PARA {
                let mut tgt_nt = ev[lpnlib::NOTE]+root;
                if root_nt > 5 {tgt_nt -= 12;}
                trans_note = self.translate_note_com(root_nt, ctbl, tgt_nt);
                deb_txt = "para:".to_string();
            }
            else if option == lpnlib::ARP_COM {
                trans_note = self.translate_note_com(root_nt, ctbl, ev[lpnlib::NOTE]);
                deb_txt = "com:".to_string();
            }
            else { // Arpeggio
                trans_note = self.translate_note_arp(root_nt, ctbl, option);
                deb_txt = "arp:".to_string();
            }
            self.last_note = trans_note;
            crnt_ev[lpnlib::NOTE] = trans_note as i16;
            deb_txt += &(root_nt.to_string() + "-" + &ctbl.to_string());
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
    fn identify_trans_option(&self, next_tick: i32, note: i16) -> i16 {
        for anaone in self.analys_dt.iter() {
            if anaone[lpnlib::TICK] == next_tick as i16 && 
               anaone[lpnlib::NOTE] == note {
                return anaone[lpnlib::ARP_DIFF];
            }
        }
        lpnlib::ARP_COM
    }
    fn translate_note_com(&self, root: i16, ctbl: i16, tgt_nt: i16) -> u8 {
        let mut proper_nt = tgt_nt;
        let tbl = txt2seq_cmps::get_table(ctbl as usize);
        let real_root = root + lpnlib::DEFAULT_NOTE_NUMBER as i16;
        let mut former_nt: i16 = 0;
        let mut found = false;
        let oct_adjust = 
            if tgt_nt - real_root >= 0 {(tgt_nt - (real_root+tbl[0]))/12}
            else {((tgt_nt-11) - (real_root+tbl[0]))/12};
        for ntx in tbl.iter() {
            proper_nt = *ntx + real_root + oct_adjust*12;
            if proper_nt == tgt_nt {
                found = true;
                break;
            }
            else if proper_nt > tgt_nt {
                if tgt_nt - former_nt <= proper_nt - tgt_nt {
                    //等距離なら下を取る
                    proper_nt = former_nt;
                }
                found = true;
                break
            }
            former_nt = proper_nt;
        }
        if !found {
            proper_nt = tbl[0] + real_root + (oct_adjust+1)*12;
            if tgt_nt - former_nt <= proper_nt - tgt_nt {
                // 等距離なら下を取る
                proper_nt = former_nt
            }
        }
        proper_nt as u8
    }
    fn translate_note_arp(&self, root: i16, ctbl: i16, nt_diff: i16) -> u8 {
        let arp_nt = self.last_note as i16 + nt_diff;
        let mut nty = lpnlib::DEFAULT_NOTE_NUMBER as i16;
        let tbl = txt2seq_cmps::get_table(ctbl as usize);
        if nt_diff == 0 {
            arp_nt as u8
        }
        else if nt_diff > 0 {
            let mut ntx = self.last_note as i16 + 1;
            ntx = PhraseLoop::search_scale_nt_just_above(root, tbl, ntx);
            if ntx >= arp_nt {
                return ntx as u8;
            }
            while nty < 128 {
                nty = ntx + 1;
                nty = PhraseLoop::search_scale_nt_just_above(root, tbl, nty);
                if nty >= arp_nt {
                    if nty - arp_nt > arp_nt - ntx {
                        nty = ntx;
                    }
                    break;
                }
                ntx = nty;
            }
            nty as u8
        }
        else {
            let mut ntx = self.last_note as i16 - 1;
            ntx = PhraseLoop::search_scale_nt_just_below(root, tbl, ntx);
            if ntx <= arp_nt {
                return ntx as u8;
            }
            while nty >= 0 {
                nty = ntx - 1;
                nty = PhraseLoop::search_scale_nt_just_below(root, tbl, nty);
                if nty <= arp_nt {
                    if arp_nt - nty > ntx - arp_nt {
                        nty = ntx;
                    }
                    break;
                }
                ntx = nty;
            }
            nty as u8
        }
    }
    fn search_scale_nt_just_above(root: i16, tbl:&[i16], nt: i16) -> i16 {
        // nt の音程より上にある(nt含む)、一番近い root/tbl の音程を探す
        let mut scale_nt: i16 = 0;
        let mut octave: i16 = -1;
        while nt > scale_nt {// Octave 判定
            octave += 1;
            scale_nt = root + octave*12;
        }
        scale_nt = 0;
        octave -= 1;
        let mut cnt: i16 = -1;
        while nt > scale_nt { //Table index 判定
            cnt += 1;
            if cnt >= tbl.len() as i16 {
                octave += 1;
                cnt = 0;
            }
            scale_nt = root + tbl[cnt as usize] + octave*12;
        }
        scale_nt
    }
    fn search_scale_nt_just_below(root: i16, tbl:&[i16], nt: i16) -> i16 {
        // nt の音程から下にある(nt含む)、一番近い root/tbl の音程を探す
        let mut scale_nt: i16 = 0;
        let mut octave: i16 = -1;
        while nt > scale_nt {// Octave 判定
            octave += 1;
            scale_nt = root + octave*12;
        }
        scale_nt = lpnlib::MAX_NOTE_NUMBER as i16;
        octave = -1;
        let mut cnt = tbl.len() as i16;
        while nt < scale_nt { // Table index 判定
            cnt -= 1;
            if cnt < 0 {
                octave -= 1;
                cnt = tbl.len() as i16 -1;
            }
            scale_nt = root + tbl[cnt as usize] + octave*12;
        }
        scale_nt
    }
}

//*******************************************************************
//          Composition Loop Struct
//*******************************************************************
pub struct CompositionLoop {
    id: ElapseId,
    priority: u32,

    cmps_dt: Vec<Vec<i16>>,
    //analys_dt:
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
impl Elapse for CompositionLoop {
    fn id(&self) -> ElapseId {self.id}     // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {}    // User による start/play 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {} // User による stop 時にコールされる
    fn fine(&mut self, _estk: &mut ElapseStack) {} // User による fine があった次の小節先頭でコールされる
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {self.destroy()}   // 自クラスが役割を終えた時に True を返す
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick >= self.whole_tick { // =をつけないと、loop終了直後の小節頭で無限ループになる
            self.next_msr = lpnlib::FULL;
            self.destroy = true;
            return
        }

        if !self.already_end && elapsed_tick >= self.next_tick_in_cmps {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            if next_tick == lpnlib::END_OF_DATA {
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
    fn set_destroy(&mut self) {self.destroy = true;}
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}
impl CompositionLoop {
    pub fn new(sid: u32, pid: u32, knt:u8, msr: i32, msg: Vec<Vec<i16>>, whole_tick: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpCompositionLoop,},
            priority: PRI_CMPS_LOOP,
            cmps_dt: msg,
            //analys_dt:
            _keynote: knt,
            play_counter: 0,
            next_tick_in_cmps: 0,

            chord_name: "".to_string(),
            root: 0,
            translation_tbl: 0,
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
    fn _reset_note_translation(&mut self) {/*<<DoItLater>>*/}
    fn prepare_note_translation(&mut self, cd: Vec<i16>) {
        if cd[lpnlib::TYPE] == lpnlib::TYPE_CHORD {
            self.root = cd[lpnlib::CD_ROOT];
            self.translation_tbl = cd[lpnlib::CD_TABLE];

            let tbl_num: usize = self.translation_tbl as usize;
            let tbl_name = crate::cmd::txt2seq_cmps::get_table_name(tbl_num);
            let cname = tbl_name.to_string();
            if cname.chars().nth(0).unwrap_or(' ') == '_' {
                let root_index = ((self.root-1)/3) as usize;
                let root = crate::cmd::txt2seq_cmps::get_root_name(root_index);
                self.chord_name = root.to_string() + &cname[1..];
            }
            else {
                self.chord_name = cname;
            }
            println!("Chord Data: {}, {}, {}",self.chord_name, cd[lpnlib::CD_ROOT], cd[lpnlib::CD_TABLE]);
        }
    }
    fn generate_event(&mut self, _crnt_: &CrntMsrTick, _estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut trace: usize = self.play_counter;
        let mut next_tick: i32;
        let cmps = self.cmps_dt.to_vec();
        loop {
            let max_ev: usize = cmps.len();
            if max_ev <= trace {
                next_tick = lpnlib::END_OF_DATA;   // means sequence finished
                break
            }
            next_tick = cmps[trace][lpnlib::TICK] as i32;
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