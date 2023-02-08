//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use super::elapse::{PRI_LOOP, LOOP_ID_OFS, Elapse};
use super::tickgen::CrntMsrTick;

//---------------------------------------------------------
pub trait Loop: Elapse {
    fn new(num: u32) -> Rc<RefCell<Self>>;
    fn calc_serial_tick(&self, msr: i32, tick: u32) -> u32;
}

//---------------------------------------------------------
pub struct PhraseLoop {
    id: u32,
    priority: u32,
}
impl Elapse for PhraseLoop {
    fn id(&self) -> u32 {self.id}         // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, u32) {    // 次に呼ばれる小節番号、Tick数を返す
        (0,0)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる

    }
    fn stop(&mut self) {        // User による stop 時にコールされる

    }
    fn fine(&mut self) {        // User による fine があった次の小節先頭でコールされる

    }
    fn process(&mut self, crnt_: &CrntMsrTick) {    // 再生 msr/tick に達したらコールされる

    }
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}
impl Loop for PhraseLoop {
    fn new(num: u32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: LOOP_ID_OFS+num,
            priority: PRI_LOOP,
        }))
    }
    fn calc_serial_tick(&self, msr: i32, tick: u32) -> u32 {return 0;}
}

//---------------------------------------------------------
pub struct CompositionLoop {
    id: u32,
    priority: u32,
}
impl Elapse for CompositionLoop {
    fn id(&self) -> u32 {self.id}         // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, u32) {    // 次に呼ばれる小節番号、Tick数を返す
        (0,0)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる

    }
    fn stop(&mut self) {        // User による stop 時にコールされる

    }
    fn fine(&mut self) {        // User による fine があった次の小節先頭でコールされる

    }
    fn process(&mut self, crnt_: &CrntMsrTick) {    // 再生 msr/tick に達したらコールされる

    }
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}

impl Loop for CompositionLoop {
    fn new(num: u32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: LOOP_ID_OFS+num,
            priority: PRI_LOOP,
        }))
    }
    fn calc_serial_tick(&self, msr: i32, tick: u32) -> u32 {return 0;}
}