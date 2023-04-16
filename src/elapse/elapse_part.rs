//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::elapse::elapse_loop::Loop;
use crate::lpnlib::*;
use super::elapse::*;
use super::elapse_loop::{PhraseLoop, CompositionLoop, DamperLoop};
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;

#[derive(Debug,Copy,Clone)]
struct PartBasicPrm {
    part_num: u32,
    keynote: u8,
    sync_flag: bool,
}

//*******************************************************************
//          Phrase Loop Manager Struct
//*******************************************************************
struct PhrLoopManager {
    first_msr_num: i32,
    max_loop_msr: i32,
    whole_tick: i32,
    loop_cntr: u32,         // loop sid
    new_data_stock: Option<Vec<Vec<i16>>>,
    new_ana_stock: Option<Vec<Vec<i16>>>,
    whole_tick_stock: i16,
    loop_phrase: Option<Rc<RefCell<PhraseLoop>>>,
    state_reserve: bool,
}
impl PhrLoopManager {
    pub fn new() -> Self {
        Self {
            first_msr_num: 0,
            max_loop_msr: 0,
            whole_tick: 0,
            loop_cntr: 0,
            new_data_stock: None,
            new_ana_stock: None,
            whole_tick_stock: 0,
            loop_phrase: None,
            state_reserve: false,
        }
    }
    pub fn start(&mut self) {
        self.first_msr_num = 0;
    }
    pub fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk, pbp);
            }
            else if self.max_loop_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk, pbp);
            }
            else if self.max_loop_msr != 0 &&
              (crnt_.msr - self.first_msr_num)%(self.max_loop_msr) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk, pbp);
            }
            else if self.max_loop_msr != 0 && pbp.sync_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                if let Some(phr) = self.loop_phrase.as_mut() {
                    phr.borrow_mut().set_destroy();
                }
                self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk, pbp);
            }
            else {
                // 現在の Loop Obj が終了していない時
                // state_reserve は持ち越す
            }
        }
        else if self.max_loop_msr != 0 &&
          (crnt_.msr - self.first_msr_num)%(self.max_loop_msr) == 0 {
            // 同じ Loop.Obj を生成する
            self.new_loop(crnt_.msr, crnt_.tick_for_onemsr, estk, pbp);
        }
        else {
            // 通常の Loop 中
        }
    }
    pub fn rcv_msg(&mut self, msg: Vec<Vec<i16>>, whole_tick: i16) {
        //println!("Phrase Msg: {:?}", msg);
        if msg.len() == 0 && whole_tick == 0 {self.new_data_stock = None;}
        else {self.new_data_stock = Some(msg);}
        self.state_reserve = true;
        self.whole_tick_stock = whole_tick;
    }
    pub fn rcv_ana(&mut self, msg: Vec<Vec<i16>>) {
        //println!("Analysed Msg: {:?}", msg);
        if msg.len() == 0 {self.new_ana_stock = None;}
        else {self.new_ana_stock = Some(msg);}
        self.state_reserve = true;
    }
    pub fn get_phr(&self) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.loop_phrase.clone()    // 重いclone()?
    }
    pub fn gen_msrcnt(&self, crnt_msr: i32) -> String {
        if let Some(phr) = &self.loop_phrase {
            let denomirator = self.max_loop_msr;
            let numerator = crnt_msr - phr.borrow().first_msr_num() + 1;    // 1origin
            format!("{}/{}", numerator, denomirator)
        }
        else {
            String::from("---")
        }
    }
    fn new_loop(&mut self, msr: i32, tick_for_onemsr: i32, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        self.first_msr_num = msr;
        if let Some(phr) = &self.new_data_stock {
        if let Some(ana) = &self.new_ana_stock {
            // 新しいデータが来ていれば、新たに Loop Obj.を生成
            self.whole_tick = self.whole_tick_stock as i32;
            println!("New Phrase Loop! --whole tick: {}", self.whole_tick_stock);

            // その時の beat 情報で、whole_tick を loop_measure に換算
            let plus_one = if self.whole_tick%tick_for_onemsr == 0 {0} else {1};
            self.max_loop_msr = self.whole_tick/tick_for_onemsr + plus_one;

            //self.update_loop_for_gui(); // for 8indicator
            if self.whole_tick == 0 {
                self.state_reserve = true; // 次小節冒頭で呼ばれるように
                self.loop_phrase = None;
                return;
            }

            self.loop_cntr += 1;
            let lp = PhraseLoop::new(self.loop_cntr, pbp.part_num, 
                pbp.keynote, msr, phr.to_vec(), ana.to_vec(), self.whole_tick);
            self.loop_phrase = Some(Rc::clone(&lp));
            estk.add_elapse(lp);
        }}
        else {
            self.whole_tick = 0;
        }
    }
}

//*******************************************************************
//          Composition Loop Manager Struct
//*******************************************************************
struct CmpsLoopManager {
    first_msr_num: i32,
    max_loop_msr: i32,
    whole_tick: i32,
    loop_cntr: u32,         // loop sid
    new_data_stock: Option<Vec<Vec<i16>>>,
    whole_tick_stock: i16,
    loop_cmps: Option<Rc<RefCell<CompositionLoop>>>,
    state_reserve: bool,
}
impl CmpsLoopManager {
    pub fn new() -> Self {
        Self {
            first_msr_num: 0,
            max_loop_msr: 0,
            whole_tick: 0,
            loop_cntr: 0,
            new_data_stock: None,
            whole_tick_stock: 0,
            loop_cmps: None,
            state_reserve: false,
        }
    }
    pub fn start(&mut self) {
        self.first_msr_num = 0;
    }
    pub fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = false;
                self.new_loop(crnt_, estk, pbp);
            }
            else if self.max_loop_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_, estk, pbp);
            }
            else if self.max_loop_msr != 0 &&
              (crnt_.msr - self.first_msr_num)%(self.max_loop_msr) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_, estk, pbp);
            }
            else if self.max_loop_msr != 0 && pbp.sync_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                if let Some(phr) = self.loop_cmps.as_mut() {
                    phr.borrow_mut().set_destroy();
                }
                self.new_loop(crnt_, estk, pbp);
            }
            else {
                // 現在の Loop Obj が終了していない時
                // state_reserve は持ち越す
            }
        }
        else if self.max_loop_msr != 0 &&
          (crnt_.msr - self.first_msr_num)%(self.max_loop_msr) == 0 {
            // 同じ Loop.Obj を生成する
            self.new_loop(crnt_, estk, pbp);
        }
    }
    pub fn rcv_msg(&mut self, msg: Vec<Vec<i16>>, whole_tick: i16) {
        //println!("Composition Msg: {:?}", msg);
        if msg.len() == 0 && whole_tick == 0 {self.new_data_stock = None;}
        else {self.new_data_stock = Some(msg);}
        self.state_reserve = true;
        self.whole_tick_stock = whole_tick;
    }
    pub fn get_cmps(&self) -> Option<Rc<RefCell<CompositionLoop>>> {
        self.loop_cmps.clone()    // 重いclone()?
    }
    pub fn gen_chord_name(&self) -> String {
        if let Some(cmps) = &self.loop_cmps {
            cmps.borrow().get_chord_name()
        }
        else {String::from("")}
    }
    fn new_loop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        // 新たに Loop Obj.を生成
        if let Some(cmps) = &self.new_data_stock {
            println!("New Composition Loop!");
            self.first_msr_num = crnt_.msr;    // 計測開始の更新
            self.whole_tick = self.whole_tick_stock as i32;

            // その時の beat 情報で、whole_tick を loop_measure に換算
            let plus_one = if self.whole_tick%crnt_.tick_for_onemsr == 0 {0} else {1};
            self.max_loop_msr = self.whole_tick/crnt_.tick_for_onemsr + plus_one;

            if self.whole_tick == 0 {
                self.state_reserve = true; // 次小節冒頭で呼ばれるように
                self.loop_cmps = None;
                return;
            }

            self.loop_cntr += 1;
            let cmplp = CompositionLoop::new(self.loop_cntr, pbp.part_num, 
                pbp.keynote, crnt_.msr, cmps.to_vec(), self.whole_tick);
            cmplp.borrow_mut().process(crnt_,estk); // 起動後、初回のみ process を呼び、同タイミングの再生を行う
            self.loop_cmps = Some(Rc::clone(&cmplp));
            estk.add_elapse(cmplp);
        }
        else {
            // 新しい Composition が空のとき
            self.max_loop_msr = 0;
            self.whole_tick = 0;
            self.loop_cntr = 0;
            self.state_reserve = true;
            self.loop_cmps = None;
        }
    }
}
//*******************************************************************
//          Damper Loop Manager Struct
//*******************************************************************
struct DamperLoopManager {
    first_msr_num: i32,
    loop_dmpr: Option<Rc<RefCell<DamperLoop>>>,
    loop_cntr: u32,
}
impl DamperLoopManager {
    pub fn new() -> Self {
        Self {
            first_msr_num: 0,
            loop_dmpr: None,
            loop_cntr: 0,
        }
    }
    pub fn start(&mut self) {self.first_msr_num = 0;}
    pub fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        let dp = DamperLoop::new(self.loop_cntr, pbp.part_num, crnt_.msr);
        self.loop_dmpr = Some(Rc::clone(&dp));
        estk.add_elapse(dp);
        self.loop_cntr += 1;
    }
}
//*******************************************************************
//          Part Struct
//*******************************************************************
pub struct Part {
    id: ElapseId,
    priority: u32,

    keynote: u8,
    _base_note: u8,
    first_measure_num: i32,
    next_msr: i32,
    next_tick: i32,
    pm: PhrLoopManager,
    cm: CmpsLoopManager,
    dm: Option<DamperLoopManager>,
    sync_next_msr_flag: bool,
}
impl Part {
    pub fn new(num: u32) -> Rc<RefCell<Part>> {
        // left なら 1, でなければ 0
        let left_part = 1-(num/(MAX_LEFT_PART as u32));
        let new_id = ElapseId {pid:0, sid:num, elps_type: ElapseType::TpPart,};
        Rc::new(RefCell::new(Self {
            id: new_id,
            priority: PRI_PART,
            keynote: 0,
            _base_note: DEFAULT_NOTE_NUMBER - 12*(left_part as u8),
            first_measure_num: 0,
            next_msr: 0,
            next_tick: 0,
            pm: PhrLoopManager::new(),
            cm: CmpsLoopManager::new(),
            dm: if num as usize == DAMPER_PEDAL_PART {Some(DamperLoopManager::new())} else {None},
            sync_next_msr_flag: false,
        }))
    }
    pub fn change_key(&mut self, knt: u8) {
        self.keynote = knt;          // 0-11
        self.pm.state_reserve = true;
    }
    pub fn rcv_phr_msg(&mut self, msg: Vec<Vec<i16>>, whole_tick: i16) {
        self.pm.rcv_msg(msg, whole_tick);
    }
    pub fn rcv_cmps_msg(&mut self, msg: Vec<Vec<i16>>, whole_tick: i16) {
        self.cm.rcv_msg(msg, whole_tick);
    }
    pub fn rcv_ana_msg(&mut self, msg_ana: Vec<Vec<i16>>) {
        self.pm.rcv_ana(msg_ana);
    }
    pub fn get_phr(&self) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.pm.get_phr()
    }
    pub fn get_cmps(&self) -> Option<Rc<RefCell<CompositionLoop>>> {
        self.cm.get_cmps()
    }
    pub fn set_sync(&mut self) {
        self.pm.state_reserve = true;
        self.cm.state_reserve = true;
        self.sync_next_msr_flag = true;
    }
    pub fn gen_part_indicator(&self, crnt_: &CrntMsrTick) -> String {
        if self.pm.whole_tick != 0 {
            let msrcnt = self.pm.gen_msrcnt(crnt_.msr);
            let chord_name = self.cm.gen_chord_name();
            format!("{}{} {}",self.id.sid+4, msrcnt, chord_name)
        }
        else {format!("{}---",self.id.sid+4)}
    }
}
impl Elapse for Part {
    fn id(&self) -> ElapseId {self.id}      // id を得る
    fn prio(&self) -> u32 {self.priority}   // priority を得る
    fn next(&self) -> (i32, i32) {          // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {      // User による start/play 時にコールされる
        self.first_measure_num = 0;
        self.next_msr = 0;
        self.next_tick = 0;
        self.cm.start();
        self.pm.start();
        if let Some(dmpr) = self.dm.as_mut() {
            dmpr.start();
        }
    }
    fn stop(&mut self, _estk: &mut ElapseStack) {}        // User による stop 時にコールされる
    fn fine(&mut self, _estk: &mut ElapseStack) {}        // User による fine があった次の小節先頭でコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        let pbp = PartBasicPrm {
                                    part_num: self.id.sid,
                                    keynote: self.keynote,
                                    sync_flag: self.sync_next_msr_flag,
                                };
        self.cm.process(crnt_, estk, pbp);
        self.pm.process(crnt_, estk, pbp);
        if let Some(dmpr) = self.dm.as_mut() {
            dmpr.process(crnt_, estk, pbp);
        }

        self.sync_next_msr_flag = false;
        // 毎小節の頭で process() がコール
        self.next_msr = crnt_.msr + 1
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {   // 自クラスが役割を終えた時に True を返す
        false
    }
}