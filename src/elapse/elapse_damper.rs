//  Created by Hasebe Masahiko on 2024/01/27
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse::*;
use super::elapse_loop::DamperLoop;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;

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

    first_msr_num: i32,
    loop_dmpr: Option<Rc<RefCell<DamperLoop>>>,
    loop_cntr: u32,
}
impl DamperPart {
    pub fn new(num: u32) -> Rc<RefCell<DamperPart>> {
        let new_id = ElapseId {
            pid: 0,
            sid: num,
            elps_type: ElapseType::TpPart,
        };
        Rc::new(RefCell::new(Self {
            id: new_id,
            priority: PRI_PART,
            during_play: false,
            next_msr: 0,
            next_tick: 0,
            start_flag: false,

            first_msr_num: 0,
            loop_dmpr: None,
            loop_cntr: 0,
        }))
    }
    fn new_msr(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        let dp = DamperLoop::new(self.loop_cntr, self.id.sid, crnt_.msr);
        self.loop_dmpr = Some(Rc::clone(&dp));
        estk.add_elapse(dp);
        self.loop_cntr += 1;
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
    fn start(&mut self) {
        // User による start/play 時にコールされる
        self.during_play = true;
        self.start_flag = true;
        self.next_msr = 0;
        self.next_tick = 0;
        if self.loop_dmpr.is_some() {
            self.first_msr_num = 0;
        }
    }
    fn stop(&mut self, _estk: &mut ElapseStack) {
        // User による stop 時にコールされる
        self.during_play = false;
    }
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        // 再生 msr/tick に達したらコールされる
        if self.start_flag {
            // Start 直後
            self.new_msr(crnt_, estk);
            self.start_flag = false;
        } else if self.next_tick == 0{
            // 小節先頭のみ
            self.new_msr(crnt_, estk);
        }

        // 次回 process を呼ぶタイミング
        if self.next_tick == 0 {
            // 小節最後の tick
            self.next_tick = crnt_.tick_for_onemsr - 1;
        } else {
            // 小節最初の tick
            self.next_msr = crnt_.msr + 1;
            self.next_tick = 0;
        }
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {
        // 自クラスが役割を終えた時に True を返す
        false
    }
}
