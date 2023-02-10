//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib;
use super::elapse::{PRI_LOOP, LOOP_ID_OFS, Elapse};
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;

//---------------------------------------------------------
pub trait Loop: Elapse {
    fn new(num: u32, knt:u8, msr: i32) -> Rc<RefCell<Self>>;
    fn calc_serial_tick(&self, crnt_: &CrntMsrTick) -> i32;
    fn gen_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32);
}

//---------------------------------------------------------
pub struct PhraseLoop {
    id: u32,
    priority: u32,

    phrase_dt: Option<Vec<Vec<u16>>>,
    //analys_dt:
    keynote: u8,
    part_num: u32,    // 親パートの番号
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
    fn id(&self) -> u32 {self.id}         // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (0,0)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる

    }
    fn stop(&mut self) {        // User による stop 時にコールされる

    }
    fn fine(&mut self) {        // User による fine があった次の小節先頭でコールされる

    }
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
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        self.destroy
    }
}
impl Loop for PhraseLoop {
    fn new(num: u32, knt: u8, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: LOOP_ID_OFS+num,
            priority: PRI_LOOP,
            phrase_dt: None,
            keynote: knt,
            part_num: num,    // 親パートの番号
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
    fn calc_serial_tick(&self, crnt_: &CrntMsrTick) -> i32 {
        (crnt_.msr - self.first_msr_num)*crnt_.tick_for_onemsr + crnt_.tick
    }
    fn gen_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32) {
        let tick = srtick%crnt_.tick_for_onemsr;
        let msr = self.first_msr_num + srtick/crnt_.tick_for_onemsr;
        (msr, tick)
    }
}
impl PhraseLoop {
    fn note_event(&self, ev: Vec<u16>, next_tick: i32, msr: i32, tick: i32) {
        // phr: ['note', tick, duration, note, velocity]
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, elapsed_tick: i32) -> i32 {
        let mut max_ev = 0;
        let mut trace: usize = self.play_counter;
        let mut next_tick: i32 = lpnlib::END_OF_DATA;
        loop {
            if let Some(phr) = &self.phrase_dt {
                max_ev = phr.len();
                next_tick = phr[trace][lpnlib::TICK] as i32;
                if max_ev <= trace {
                    next_tick = lpnlib::END_OF_DATA;   // means sequence finished
                    break;
                }
                if next_tick <= elapsed_tick {
                    let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                    if phr[trace][lpnlib::TYPE] == lpnlib::TYPE_DAMPER {
                        // phr: ['damper', duration, tick, value]
                        //estk.add_obj(elpn.Damper(self.est, self.md, phr, msr, tick))
                    }
                    else if phr[trace][lpnlib::TYPE] == lpnlib::TYPE_NOTE {
                        self.note_event(phr[trace].clone(), next_tick, msr, tick);
                    }
                }
                else {break;}
                trace += 1;
            }
            else {break;}
        }

        self.play_counter = trace;
        return next_tick;
    }
}


//---------------------------------------------------------
pub struct CompositionLoop {
    id: u32,
    priority: u32,
}
impl Elapse for CompositionLoop {
    fn id(&self) -> u32 {self.id}         // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (0,0)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる

    }
    fn stop(&mut self) {        // User による stop 時にコールされる

    }
    fn fine(&mut self) {        // User による fine があった次の小節先頭でコールされる

    }
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる

    }
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}

impl Loop for CompositionLoop {
    fn new(num: u32, knt:u8, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: LOOP_ID_OFS+num,
            priority: PRI_LOOP,
        }))
    }
    fn calc_serial_tick(&self, crnt_: &CrntMsrTick) -> i32 {0}
    fn gen_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32) {(0,0)}
}