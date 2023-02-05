//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;
use super::elapse::Elapse;

pub const PART_ID_OFS: u32 = 0x10000;
pub struct Part {
    id: u32,
    priority: u32,
}

impl Elapse for Part {
    fn id(&self) -> u32 {self.id}           // id を得る
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
    fn process(&mut self, _msr: i32, _tick: u32) {    // 再生 msr/tick に達したらコールされる

    }
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}

impl Part {
    pub fn new(num: u32) -> Rc<RefCell<dyn Elapse>> {
        Rc::new(RefCell::new(Self {
            id: self::PART_ID_OFS+num,
            priority: 0,
        }))
    }
}