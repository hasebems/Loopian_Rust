//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;
use super::elapse::{PRI_PART, PART_ID_OFS, Elapse};
use super::elapse_loop::Loop;
use super::tickgen::CrntMsrTick;

pub struct Part {
    id: u32,
    priority: u32,

    first_measure_num: i32,
    next_msr: i32,
    next_tick: u32,
    max_loop_msr: u32,
    //loop_elps: Loop,

    state_reserve: bool,
    sync_next_msr_flag: bool,

/*
    self.loop_obj = None
    left_part = 1-(num%nlib.FIRST_NORMAL_PART)//nlib.MAX_LEFT_PART # left なら 1, でなければ 0
    self.keynote = 0
    self.base_note = nlib.DEFAULT_NOTE_NUMBER - 12*left_part
    self.max_loop_msr = 0   // whole_tick と同時生成
    self.whole_tick = 0     // max_loop_msr と同時生成
    self.sync_next_msr_flag = False
    self.state_reserve = False
    self.seqdt_part = None
    self.cb_handler = None
    self.handler_owner = None
*/
}

impl Elapse for Part {
    fn id(&self) -> u32 {self.id}           // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, u32) {    // 次に呼ばれる小節番号、Tick数を返す
        (0,0)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる
        self.first_measure_num = 0;
        self.next_msr = 0;
        self.next_tick = 0;
        self.state_reserve = true;
    }
    fn stop(&mut self) {        // User による stop 時にコールされる

    }
    fn fine(&mut self) {        // User による fine があった次の小節先頭でコールされる

    }
    fn process(&mut self, crnt_: &CrntMsrTick) {    // 再生 msr/tick に達したらコールされる
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = false;
                self.new_loop(crnt_.msr);
            }
            else if self.max_loop_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_.msr);
            }
            else if self.max_loop_msr != 0 &&
              (crnt_.msr - self.first_measure_num)%(self.max_loop_msr as i32) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_.msr);
            }
            else if self.max_loop_msr != 0 && self.sync_next_msr_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                self.sync_next_msr_flag = false;
                //self.est.del_obj(self.loop_elps);
                self.new_loop(crnt_.msr);
            }
            else {
                // 現在の Loop Obj が終了していない時
                // state_reserve は持ち越す
            }
        }
        else if self.max_loop_msr != 0 &&
          (crnt_.msr - self.first_measure_num)%(self.max_loop_msr as i32) == 0 {
                // 同じ Loop.Obj を生成する
                self.new_loop(crnt_.msr);
        }
        // 毎小節の頭で process() がコール
        self.next_msr = crnt_.msr + 1
    }
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}

impl Part {
    pub fn new(num: u32) -> Rc<RefCell<dyn Elapse>> {
        Rc::new(RefCell::new(Self {
            id: self::PART_ID_OFS+num,
            priority: PRI_PART,
            first_measure_num: 0,
            next_msr: 0,
            next_tick: 0,
            max_loop_msr: 0,
            //loop_elps: Loop::new(0),
            state_reserve: false,
            sync_next_msr_flag: false,
        }))
    }
    fn new_loop(&self, msr: i32) {

    }
}