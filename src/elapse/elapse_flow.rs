//  Created by Hasebe Masahiko on 2023/05/18.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

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
//
//
pub struct Flow {
    id: ElapseId,
    priority: u32,

    destroy: bool,
}

impl Flow {
    pub fn new(sid: u32, pid: u32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpFlow,},
            priority: PRI_FLOW,
            destroy: false,
        }))
    }
    /// Flow オブジェクトを消滅させ、MIDI IN による発音を終了
    pub fn deactivate(&mut self) {
        // 発音中の音をキャンセル
        self.destroy = true
    }
}

impl Elapse for Flow {
    /// id を得る
    fn id(&self) -> ElapseId {self.id}
    /// priority を得る
    fn prio(&self) -> u32 {self.priority}
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32) {(0,0)}
    /// User による start/play 時にコールされる
    fn start(&mut self) {}
    /// User による stop 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack) {}
    /// User による fine があった次の小節先頭でコールされる
    fn fine(&mut self, estk: &mut ElapseStack) {}
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {}
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, msg: ElapseMsg, msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {self.destroy}
}