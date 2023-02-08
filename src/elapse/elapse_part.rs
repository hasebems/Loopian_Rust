//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib;
use super::elapse::{PRI_PART, PART_ID_OFS, Elapse};
use super::elapse_loop::{Loop, PhraseLoop, CompositionLoop};
use super::tickgen::CrntMsrTick;

pub struct Part {
    id: u32,
    priority: u32,

    keynote: u8,
    base_note: u8,
    first_measure_num: i32,
    next_msr: i32,
    next_tick: u32,
    max_loop_msr: u32,
    whole_tick: u32,
    loop_elps: Option<Rc<RefCell<dyn Elapse>>>,

    state_reserve: bool,
    sync_next_msr_flag: bool,
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
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr);
            }
            else if self.max_loop_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr);
            }
            else if self.max_loop_msr != 0 &&
              (crnt_.msr - self.first_measure_num)%(self.max_loop_msr as i32) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr);
            }
            else if self.max_loop_msr != 0 && self.sync_next_msr_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                self.sync_next_msr_flag = false;
                //self.est.del_obj(self.loop_elps);
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr);
            }
            else {
                // 現在の Loop Obj が終了していない時
                // state_reserve は持ち越す
            }
        }
        else if self.max_loop_msr != 0 &&
          (crnt_.msr - self.first_measure_num)%(self.max_loop_msr as i32) == 0 {
                // 同じ Loop.Obj を生成する
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr);
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
        // left なら 1, でなければ 0
        let left_part = 1-(num%(lpnlib::FIRST_PHRASE_PART as u32))/(lpnlib::MAX_LEFT_PART as u32);
        Rc::new(RefCell::new(Self {
            id: PART_ID_OFS+num,
            priority: PRI_PART,
            keynote: 0,
            base_note: lpnlib::DEFAULT_NOTE_NUMBER - 12*(left_part as u8),
            first_measure_num: 0,
            next_msr: 0,
            next_tick: 0,
            max_loop_msr: 0,
            whole_tick: 0,     // max_loop_msr と同時生成
            loop_elps: None,
            state_reserve: false,
            sync_next_msr_flag: false,
        }))
    }
    fn new_loop(&mut self, msr: i32, tick_for_onemsr: u32) {
        // 新たに Loop Obj.を生成
        self.first_measure_num = msr;    // 計測開始の更新
        //self.whole_tick, elm, ana = self.seqdt_part.get_final(msr)

        // その時の beat 情報で、whole_tick を loop_measure に換算
        let plus_one = if self.whole_tick%tick_for_onemsr == 0 {0} else {1};
        self.max_loop_msr = self.whole_tick/tick_for_onemsr + plus_one;

        //self.update_loop_for_gui(); // for 8indicator
        if self.whole_tick == 0 {
            self.state_reserve = true; // 次小節冒頭で呼ばれるように
            self.loop_elps = None;
            return;
        }

        let part_num = self.id - PART_ID_OFS;
        if part_num >= lpnlib::FIRST_PHRASE_PART as u32 {
            let lp: Rc<RefCell<PhraseLoop>> = Loop::new(part_num);
            self.loop_elps = Some(lp);
            //    self.est, self.md, msr, elm, ana,  \
            //    self.keynote, self.whole_tick, part_num);
            //self.est.add_obj(self.loop_elps);
        }
        else {
            let lp: Rc<RefCell<CompositionLoop>> = Loop::new(part_num);
            self.loop_elps = Some(lp);
            //    self.est, self.md, msr, elm, ana, \
            //    self.keynote, self.whole_tick, part_num);
            //self.est.add_obj_in_front(self.loop_elps);
        }
    }
}