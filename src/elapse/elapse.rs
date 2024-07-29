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
pub const PRI_CMPS_LOOP: u32 = 200;
pub const PRI_FLOW: u32 = 250;
pub const PRI_PHR_LOOP: u32 = 300;
pub const PRI_NOTE: u32 = 400;
pub const PRI_DMPR: u32 = 500;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ElapseType {
    _TpNone,
    TpPart,
    TpDamperPart,
    TpPhraseLoop,
    TpCompositionLoop,
    TpNote,
    TpFlow,
    _TpDamper,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ElapseMsg {
    _MsgNone,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ElapseId {
    pub pid: u32, // parent
    pub sid: u32, // self
    pub elps_type: ElapseType,
}

pub trait Elapse {
    /// id を得る
    fn id(&self) -> ElapseId;
    /// priority を得る
    fn prio(&self) -> u32;
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32);
    /// User による start/play 時にコールされる
    fn start(&mut self);
    /// User による stop 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack);
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack);
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, msg: ElapseMsg, msg_data: u8);
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool;
}
