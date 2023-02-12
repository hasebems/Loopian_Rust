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

//---------------------------------------------------------
pub trait Loop: Elapse {
    fn new(sid: u32, pid: u32, knt:u8, msr: i32) -> Rc<RefCell<Self>>;
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

//---------------------------------------------------------
pub struct PhraseLoop {
    id: ElapseId,
    priority: u32,

    phrase_dt: Option<Vec<Vec<u16>>>,
    //analys_dt:
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
    fn stop(&mut self) {self.set_destroy();} // User による stop 時にコールされる
    fn fine(&mut self) {self.set_destroy();} // User による fine があった次の小節先頭でコールされる
    fn rcv_sp(&mut self, msg: ElapseMsg, msg_data: u8, estk: &mut ElapseStack) {}
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
    fn new(sid: u32, pid: u32, knt: u8, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpPhraseLoop,},
            priority: PRI_LOOP,
            phrase_dt: None,
            keynote: knt,
            play_counter: 0,
            next_tick_in_phrase: 0,
            last_note: lpnlib::NO_NOTE,
            // for super's member
            whole_tick: 0,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,
            next_tick: 0,
        }))
    }
    fn destroy(&self) -> bool {self.destroy}
    fn set_destroy(&mut self) {self.destroy = true;}
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}
impl PhraseLoop {
    fn note_event(&self, estk: &mut ElapseStack, trace: usize, ev: Vec<u16>, next_tick: i32, msr: i32, tick: i32) {
        // phr: ['note', tick, duration, note, velocity]
        // <<DoItLater>>
        //if let Some(linked_part) = estk.get_part(self.id.pid) {
        //    if let Some(linked_comp) = linked_part.borrow().get_comp() {
        //        let (root, trans_tbl) = linked_comp.borrow().get_translation();
        //    }
        //}
        let nt: Rc<RefCell<dyn Elapse>> = Note::new(trace as u32, self.id.sid, estk, &ev, msr, tick);
        estk.add_elapse(Rc::clone(&nt));
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut next_tick: i32 = lpnlib::END_OF_DATA;
        let mut trace: usize = self.play_counter;
        if let Some(phr) = &self.phrase_dt {
            let max_ev = phr.len();
            loop {
                next_tick = phr[trace][lpnlib::TICK] as i32;
                if max_ev <= trace {
                    next_tick = lpnlib::END_OF_DATA;   // means sequence finished
                    break;
                }
                if next_tick <= elapsed_tick {
                    let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                    if phr[trace][lpnlib::TYPE] == lpnlib::TYPE_DAMPER {
                        //<<DoItLater>>
                        // phr: ['damper', duration, tick, value]
                        //estk.add_obj(elpn.Damper(self.est, self.md, phr, msr, tick))
                    }
                    else if phr[trace][lpnlib::TYPE] == lpnlib::TYPE_NOTE {
                        self.note_event(estk, trace, phr[trace].clone(), next_tick, msr, tick);
                    }
                }
                else {break;}
                trace += 1;
            }
        }
        self.play_counter = trace;
        return next_tick;
    }
}


//---------------------------------------------------------
pub struct CompositionLoop {
    id: ElapseId,
    priority: u32,

    comp_dt: Option<Vec<Vec<u16>>>,
    //analys_dt:
    keynote: u8,
    play_counter: usize,
    next_tick_in_comp: i32,
    // for Composition
    chord_name: String,
    root: u8,
    translation_tbl: Vec<Vec<i32>>,

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
    fn stop(&mut self) {self.set_destroy();} // User による stop 時にコールされる
    fn fine(&mut self) {self.set_destroy();} // User による fine があった次の小節先頭でコールされる
    fn rcv_sp(&mut self, msg: ElapseMsg, msg_data: u8, estk: &mut ElapseStack) {}
    fn destroy_me(&self) -> bool {self.destroy()}   // 自クラスが役割を終えた時に True を返す
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = lpnlib::FULL;
            self.destroy = true;
            return
        }

        if elapsed_tick >= self.next_tick_in_comp {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            if next_tick == lpnlib::END_OF_DATA {
                // Composition Loop はイベントが終わっても、コード情報が終了するまで Loop が存在するようにしておく
                while self.whole_tick - self.next_tick_in_comp >= crnt_.tick_for_onemsr {
                    self.next_tick_in_comp += crnt_.tick_for_onemsr;
                    self.next_msr += 1;
                }
                self.next_tick_in_comp = self.whole_tick;
                self.next_tick = crnt_.tick_for_onemsr;
            }
            else {
                self.next_tick_in_comp = next_tick;
                let (next_msr, next_tick) = self.gen_msr_tick(crnt_, self.next_tick_in_comp);
                self.next_msr = next_msr;
                self.next_tick = next_tick;
            }
        }
    }
}
impl Loop for CompositionLoop {
    fn new(sid: u32, pid: u32, knt:u8, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpCompositionLoop,},
            priority: PRI_LOOP,
            comp_dt: None,
            //analys_dt:
            keynote: knt,
            play_counter: 0,
            next_tick_in_comp: 0,

            chord_name: "".to_string(),
            root: 0,
            translation_tbl: Vec::new(),
            // for super's member
            whole_tick: 0,
            destroy: false,
            first_msr_num: msr,
            next_msr: 0,   //   次に呼ばれる小節番号が保持される
            next_tick: 0,        
        }))
    }
    fn destroy(&self) -> bool {self.destroy}
    fn set_destroy(&mut self) {self.destroy = true;}
    fn first_msr_num(&self) -> i32 {self.first_msr_num}
}
impl CompositionLoop {
    pub fn get_translation(&self) -> (u8, Vec<u32>) {(0, vec![0,1,2,3,4,5,6,7,8,9,10,11])}
    fn reset_note_translation(&mut self) {/*<<DoItLater>>*/}
    fn prepare_note_translation(&mut self, cd: Vec<u16>) {/*<<DoItLater>>*/}
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut trace: usize = self.play_counter;
        let mut next_tick: i32 = 0;
        loop {
            if let Some(comp) = &self.comp_dt {
                let max_ev: usize = comp.len();
                if max_ev <= trace {
                    next_tick = lpnlib::END_OF_DATA;   // means sequence finished
                    break
                }
                next_tick = comp[trace][lpnlib::TICK] as i32;
                if next_tick <= elapsed_tick {
                    self.prepare_note_translation(comp[trace].clone());
                }
                else {break;}
                trace += 1;
            }
            else {
                // データを持っていない
                self.reset_note_translation();
                return lpnlib::END_OF_DATA;
            }
        }
        self.play_counter = trace;
        next_tick
    }
}