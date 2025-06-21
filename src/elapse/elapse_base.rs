//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;

// Timing Priority(pri) 数値が小さいほど優先度が高い（同じtickなら先に再生される）
pub const _PRI_NONE: u32 = 1000;
pub const PRI_PART: u32 = 100;
pub const PRI_FLOW: u32 = 200;
pub const PRI_PHR_LOOP: u32 = 300;
pub const PRI_DYNPTN: u32 = 350;
pub const PRI_NOTE: u32 = 400;
pub const PRI_DMPR: u32 = 500;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ElapseType {
    _TpNone,
    TpPart,
    TpDamperPart,
    TpPhraseLoop,
    TpClusterPattern,
    TpBrokenPattern,
    TpNote,
    TpFlow,
    _TpDamper,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ElapseMsg {
    _MsgNone,
}

//*******************************************************************
//          Elapse Struct
//*******************************************************************
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ElapseId {
    pub pid: u32, // parent
    pub sid: u32, // self
    pub elps_type: ElapseType,
}

pub trait Elapse {
    /// id を得る
    #[allow(dead_code)]
    fn id(&self) -> ElapseId;
    /// priority を得る
    fn prio(&self) -> u32;
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32);
    /// User による start/play 時にコールされる msr:開始小節番号
    fn start(&mut self, msr: i32);
    /// User による stop 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack);
    /// 再生データを消去
    fn clear(&mut self, estk: &mut ElapseStack);
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack);
    /// 特定 elapse に message を送る
    #[allow(dead_code)]
    fn rcv_sp(&mut self, msg: ElapseMsg, msg_data: u8);
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool;
}

//*******************************************************************
//          Loop Struct
//*******************************************************************
pub trait Loop: Elapse {
    fn destroy(&self) -> bool;
    fn set_destroy(&mut self);
    fn first_msr_num(&self) -> i32;
    fn calc_serial_tick(&self, crnt_: &CrntMsrTick) -> i32 {
        (crnt_.msr - self.first_msr_num()) * crnt_.tick_for_onemsr + crnt_.tick
    }
    fn gen_msr_tick(&self, crnt_: &CrntMsrTick, srtick: i32) -> (i32, i32) {
        let tick = srtick % crnt_.tick_for_onemsr;
        let msr = self.first_msr_num() + srtick / crnt_.tick_for_onemsr;
        (msr, tick)
    }
    /// Loopの途中から再生するための小節数を設定
    fn set_forward(&mut self, crnt_: &CrntMsrTick, elapsed_msr: i32);
}
