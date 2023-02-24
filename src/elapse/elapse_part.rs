//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib;
use super::elapse::*;
use super::elapse_loop::{PhraseLoop, CompositionLoop};
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;

pub struct Part {
    id: ElapseId,
    priority: u32,

    keynote: u8,
    _base_note: u8,
    first_measure_num: i32,
    next_msr: i32,
    next_tick: i32,
    max_loop_msr: i32,
    whole_tick: i32,
    loop_cmps: Option<Rc<RefCell<CompositionLoop>>>,
    loop_phrase: Option<Rc<RefCell<PhraseLoop>>>,
    loop_cntr: u32,
    new_data_stock: Option<Vec<Vec<u16>>>,
    whole_tick_stock: u16,

    state_reserve: bool,
    sync_next_msr_flag: bool,
}

impl Elapse for Part {
    fn id(&self) -> ElapseId {self.id}           // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる
        self.first_measure_num = 0;
        self.next_msr = 0;
        self.next_tick = 0;
        self.state_reserve = true;
    }
    fn stop(&mut self, _estk: &mut ElapseStack) {}        // User による stop 時にコールされる
    fn fine(&mut self, _estk: &mut ElapseStack) {}        // User による fine があった次の小節先頭でコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk);
            }
            else if self.max_loop_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk);
            }
            else if self.max_loop_msr != 0 &&
              (crnt_.msr - self.first_measure_num)%(self.max_loop_msr) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk);
            }
            else if self.max_loop_msr != 0 && self.sync_next_msr_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                self.sync_next_msr_flag = false;
                if let Some(lp) = &self.loop_phrase {
                    estk.del_elapse(lp.borrow().id());
                    self.loop_phrase = None;
                }
                if let Some(lp) = &self.loop_cmps {
                    estk.del_elapse(lp.borrow().id());
                    self.loop_cmps = None;
                }
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk);
            }
            else {
                // 現在の Loop Obj が終了していない時
                // state_reserve は持ち越す
            }
        }
        else if self.max_loop_msr != 0 &&
          (crnt_.msr - self.first_measure_num)%(self.max_loop_msr) == 0 {
            // 同じ Loop.Obj を生成する
            self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk);
        }
        // 毎小節の頭で process() がコール
        self.next_msr = crnt_.msr + 1
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}

impl Part {
    pub fn new(num: u32) -> Rc<RefCell<Part>> {
        // left なら 1, でなければ 0
        let left_part = 1-(num%(lpnlib::FIRST_PHRASE_PART as u32))/(lpnlib::MAX_LEFT_PART as u32);
        let new_id = ElapseId {pid:0, sid:num, elps_type: ElapseType::TpPart,};
        Rc::new(RefCell::new(Self {
            id: new_id,
            priority: PRI_PART,
            keynote: 0,
            _base_note: lpnlib::DEFAULT_NOTE_NUMBER - 12*(left_part as u8),
            first_measure_num: 0,
            next_msr: 0,
            next_tick: 0,
            max_loop_msr: 0,
            whole_tick: 0,     // max_loop_msr と同時生成
            loop_cmps: None,
            loop_phrase: None,
            loop_cntr: 1,
            new_data_stock: None,
            whole_tick_stock: 0,
            state_reserve: false,
            sync_next_msr_flag: false,
        }))
    }
    pub fn _change_key(&mut self, knt: u8) {
        self.keynote = knt;          // 0-11
        self.state_reserve = true;
    }
    pub fn _get_cmps(&self) -> Option<Rc<RefCell<CompositionLoop>>> {
        match &self.loop_cmps {
            Some(lc) => Some(Rc::clone(&lc)),
            None => None,
        }
    }
    pub fn rcv_msg(&mut self, msg: Vec<Vec<u16>>, whole_tick: u16) {
        //println!("Msg: {:?}", msg);
        self.new_data_stock = Some(msg);
        self.state_reserve = true;
        self.whole_tick_stock = whole_tick;
    }
    fn new_loop(&mut self, msr: i32, tick_for_onemsr: i32, estk: &mut ElapseStack) {
        // 新たに Loop Obj.を生成
        if let Some(phr) = &self.new_data_stock {
            println!("New Loop!");
            self.first_measure_num = msr;    // 計測開始の更新

            //<<DoItLater>>
            // 新しい data から、ana データを取得
            //elm, ana = self.seqdt_part.get_final(msr)
            self.whole_tick = self.whole_tick_stock as i32;

            // その時の beat 情報で、whole_tick を loop_measure に換算
            let plus_one = if self.whole_tick%tick_for_onemsr == 0 {0} else {1};
            self.max_loop_msr = self.whole_tick/tick_for_onemsr + plus_one;

            //self.update_loop_for_gui(); // for 8indicator
            if self.whole_tick == 0 {
                self.state_reserve = true; // 次小節冒頭で呼ばれるように
                self.loop_phrase = None;
                self.loop_cmps = None;
                return;
            }

            let part_num = self.id.sid;
            if part_num >= lpnlib::FIRST_PHRASE_PART as u32 {
                let lp = PhraseLoop::new(self.loop_cntr, part_num, 
                    self.keynote, msr, phr.to_vec(), self.whole_tick);
                self.loop_phrase = Some(Rc::clone(&lp));
                //<<DoItLater>> 引数の追加
                //    self.est, self.md, msr, elm, ana,  \
                //    self.keynote, self.whole_tick, part_num);
                estk.add_elapse(lp);
                self.loop_cntr += 1;
            }
            else {
                let lp = CompositionLoop::new(self.loop_cntr, part_num, 
                    self.keynote, msr, self.whole_tick);
                self.loop_cmps = Some(Rc::clone(&lp));
                //<<DoItLater>> 引数の追加
                //    self.est, self.md, msr, elm, ana, \
                //    self.keynote, self.whole_tick, part_num);
                estk.add_elapse(lp);
                self.loop_cntr += 1;
            }
        }
    }
}