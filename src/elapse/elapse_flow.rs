//  Created by Hasebe Masahiko on 2023/05/18.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib::*;
use super::elapse::*;
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;

//*******************************************************************
//          Flow Struct
//*******************************************************************
//  動作イメージ
//  ・elapse object であると同時に、Part にも集約される
//  ・Part から、制御命令（生成・消滅）、MIDI Inメッセージを受け取る
//      Part は、ElapseStack から、MIDI In メッセージを受け取る
//  ・next_tick は常に 0、next_msr は MIDI In メッセージの有無に応じて変化させる 
//
pub struct Flow {
    id: ElapseId,
    priority: u32,

    // for super's member
    //whole_tick: i32,
    destroy: bool,
    _first_msr_num: i32,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
//    destroy: bool,
}

impl Flow {
    pub fn new(sid: u32, pid: u32, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpFlow,},
            priority: PRI_FLOW,

            // for super's member
            //whole_tick: 0,
            destroy: false,
            _first_msr_num: msr,
            next_msr: FULL, // not called process()
            next_tick: 0,
        }))
    }
    /// Flow オブジェクトを消滅させ、MIDI IN による発音を終了
    pub fn deactivate(&mut self) {
        // 発音中の音をキャンセル
        self.destroy = true;
    }
    pub fn rcv_midi(&mut self, note_on:bool, locate:u8, vel:u8) {
        println!("{}:{}-{}", if note_on{"On"}else{"Of"},locate, vel);
    }
}

impl Elapse for Flow {
    /// id を得る
    fn id(&self) -> ElapseId {self.id}
    /// priority を得る
    fn prio(&self) -> u32 {self.priority}
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32) {(self.next_msr, self.next_tick)}
    /// User による start/play 時にコールされる
    fn start(&mut self) {}
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {}
    /// User による fine があった次の小節先頭でコールされる
    fn fine(&mut self, _estk: &mut ElapseStack) {}
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, _crnt: &CrntMsrTick, _estk: &mut ElapseStack) {}
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {self.destroy}
}