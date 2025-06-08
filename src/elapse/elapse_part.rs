//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::elapse_loop_phr::*;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use super::unfold_cmp::*;
use crate::elapse::elapse_flow::Flow;
use crate::lpnlib::*;

#[derive(Debug, Copy, Clone)]
pub struct PartBasicPrm {
    pub part_num: u32,
    pub keynote: u8,
    pub sync_flag: bool,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum LoopPhase {
    BeforeBeginPhr,
    DuringBeginPhr,
    AfterBeginCnct,
    OneBarBeforeEndCnct,
    BeforeEndPtr,
}
//*******************************************************************
//          Phrase Loop Wrapper Struct
//*******************************************************************
#[derive(Clone)]
struct PhrLoopWrapper {
    pub begin_phr: i32, // measure number / first measure number
    pub end_cnct: i32,  // measure number
    pub whole_tick: i32,
    pub max_loop_msr: i32, // from whole_tick
    pub phrase: Rc<RefCell<PhraseLoop>>,
}
impl PhrLoopWrapper {
    pub fn new(
        //crnt_: &CrntMsrTick,
        tick_for_onemsr: i32,
        crnt_msr: i32,
        pbp: PartBasicPrm,
        loop_id: u32,
        turnnote: i16,
        phr_stock: PhrData,
    ) -> Self {
        let mut repeat_tick = phr_stock.whole_tick as i32;
        let max_loop_msr = if repeat_tick == 0 {
            0
        } else {
            repeat_tick -= 2 * tick_for_onemsr; // 2 小節分
            if repeat_tick > 0 {
                let plus_one = if repeat_tick % tick_for_onemsr == 0 {
                    0
                } else {
                    1
                };
                (repeat_tick / tick_for_onemsr) + plus_one
            } else {
                0
            }
        };
        #[cfg(feature = "verbose")]
        println!(
            "**** PhrLoopWrapper::new: loop_id: {}, repeat_tick: {}, max_loop_msr: {}\n**** Phrase: {:?}",
            loop_id, repeat_tick, max_loop_msr, phr_stock
        );
        let phrase = PhraseLoop::new(
            loop_id,
            pbp.part_num,
            PhraseLoopParam::new(
                pbp.keynote,
                crnt_msr,
                phr_stock.evts.to_vec(),
                phr_stock.ana.to_vec(),
                phr_stock.whole_tick as i32,
                turnnote,
            ),
        );
        Self {
            begin_phr: crnt_msr,
            end_cnct: crnt_msr + max_loop_msr + 1,
            whole_tick: phr_stock.whole_tick as i32,
            max_loop_msr,
            phrase: Rc::clone(&phrase),
        }
    }
    /// 現在の PhraseLoop の状態を返す
    pub fn crnt_phase(&self, crnt_: &CrntMsrTick) -> LoopPhase {
        if crnt_.msr < self.begin_phr {
            // Phrase Loop の開始前
            LoopPhase::BeforeBeginPhr
        } else if crnt_.msr == self.begin_phr {
            // Phrase Loop の開始時
            LoopPhase::DuringBeginPhr
        } else if crnt_.msr < self.end_cnct - 1 {
            // Phrase Loop の begin_cnct 後の小節
            LoopPhase::AfterBeginCnct
        } else if crnt_.msr == self.end_cnct - 1 {
            // Phrase Loop の end_cnct 前の小節
            LoopPhase::OneBarBeforeEndCnct
        } else if crnt_.msr == self.end_cnct {
            // Phrase Loop の end_cnct 後の小節
            LoopPhase::BeforeEndPtr
        } else {
            // その他の状態
            LoopPhase::BeforeBeginPhr // 仮の値
        }
    }
    // PhraseLoop に残りのイベントがあるか調べる
}

//*******************************************************************
//          Phrase Loop Manager Struct
//*******************************************************************
struct PhrLoopManager {
    loop_id: u32,            // loop sid
    phr_stock: Vec<PhrData>, // 0: Normal
    phr_idx: usize,          // 0: Normal, 現在再生されている phr_stock の index
    phr_instance_a: Option<PhrLoopWrapper>,
    phr_instance_b: Option<PhrLoopWrapper>,
    vari_reserve: Option<usize>, // 1-9: rsv, None: Normal
    a_is_gened_last: bool,       // true: instance_a, false: instance_b
    begin_phr_ev: bool,
    chasing_play: bool, // 追いかけ再生フラグ
    del_a_ev: bool,     // instance_a を削除するイベント
    del_b_ev: bool,     // instance_b を削除するイベント
    turnnote: i16,
}
impl PhrLoopManager {
    pub fn new() -> Self {
        Self {
            loop_id: 0,
            phr_stock: vec![PhrData::empty()],
            phr_idx: 0,
            phr_instance_a: None,
            phr_instance_b: None,
            vari_reserve: None,
            a_is_gened_last: false,
            begin_phr_ev: false,
            chasing_play: false, // 追いかけ再生フラグ
            del_a_ev: false,
            del_b_ev: false,
            turnnote: DEFAULT_TURNNOTE,
        }
    }
    pub fn msrtop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        self.delete_ev();
        // Phrase Loop の状態を確認し、新 Phrase の再生開始処理などを行う
        if let Some(idx) = self.exist_msr_phr(crnt_) {
            // Measure 指定がある場合
            self.phr_idx = idx;
            self.gen_phr_alternately(crnt_, estk, pbp, 0);
        } else if let Some(vr) = self.vari_reserve {
            // Variation 指定がある場合
            if let Some(idx) = self.exist_vari_phr(vr) {
                self.phr_idx = idx;
                self.gen_phr_alternately(crnt_, estk, pbp, 0);
            }
            self.vari_reserve = None; // 予約をリセット
        } else if self.begin_phr_ev {
            // この小節が begin_phr になるとき
            self.gen_phr_alternately(crnt_, estk, pbp, 0); // Alternate
            self.begin_phr_ev = false;
        } else if self.if_end_prpr(crnt_) {
            // この小節が end_prpr になるとき（追いかけより優先）
            self.phr_idx = 0; // 0: Normal
            self.gen_phr_alternately(crnt_, estk, pbp, 0); // Alternate
            self.begin_phr_ev = false;
            self.chasing_play = false; // 追いかけ再生フラグをリセット
        } else if self.chasing_play {
            // 追いかけ再生フラグが立っているとき
            self.chasing_play(crnt_, estk, pbp);
            self.chasing_play = false; // 追いかけ再生フラグをリセット
            self.delete_ev();
        } else {
            // 何もしない
        }
    }
    pub fn start(&mut self) {
        if self.phr_stock.len() >= self.phr_idx && self.phr_stock[self.phr_idx].whole_tick != 0 {
            // instance_a を使用
            self.begin_phr_ev = true;
        }
    }
    //    pub fn sync(&mut self) {
    //        // 次の小節を begin_cnct として、再生し直す
    //        self.begin_phr_ev = false;
    //    }
    pub fn rcv_phrase(
        &mut self,
        msg: PhrData,
        crnt_: &CrntMsrTick,
        estk_: &mut ElapseStack,
        pbp: PartBasicPrm,
        during_play: bool,
    ) {
        if msg.evts.is_empty() && msg.whole_tick == 0 {
            // phrase = [] の時の処理
            self.delete_phrase(msg);
        } else {
            // Phrase 入力イベントがあった場合
            self.append_phrase(msg, crnt_, estk_, pbp, during_play);
        }
    }
    pub fn get_phr(&self) -> Option<&Rc<RefCell<PhraseLoop>>> {
        if self.a_is_gened_last {
            if let Some(inst) = &self.phr_instance_a {
                return Some(&inst.phrase);
            }
        } else if let Some(inst) = &self.phr_instance_b {
            return Some(&inst.phrase);
        }
        None
    }
    pub fn gen_msrcnt(&self, crnt_msr: i32) -> Option<(i32, i32)> {
        if let Some(phrloop) = self.crnt_phr() {
            let denomirator = phrloop.max_loop_msr;
            let numerator = crnt_msr - phrloop.phrase.borrow().first_msr_num() + 1; // 1origin
            //format!("{}/{}", numerator, denomirator)
            Some((numerator, denomirator))
        } else {
            None
        }
    }
    pub fn del_phrase(&mut self) {
        self.del_a();
        self.del_b();
        self.phr_stock = vec![PhrData::empty()];
        self.phr_idx = 0;
        self.vari_reserve = None;
        self.a_is_gened_last = true; // instance_a を使用
        self.begin_phr_ev = false;
    }
    pub fn set_turnnote(&mut self, tn: i16) {
        self.turnnote = tn;
    }
    pub fn reserve_vari(&mut self, vari_num: usize) {
        if vari_num != 0 {
            self.vari_reserve = Some(vari_num); // 1-9
        }
    }
    pub fn whole_tick(&self) -> i32 {
        if let Some(phr) = self.crnt_phr() {
            phr.whole_tick
        } else {
            0
        }
    }
    //---------------------------------------------------------------
    fn crnt_phr(&self) -> Option<PhrLoopWrapper> {
        if self.a_is_gened_last {
            if let Some(inst) = &self.phr_instance_a {
                return Some(inst.clone());
            }
        } else if let Some(inst) = &self.phr_instance_b {
            return Some(inst.clone());
        }
        None
    }
    fn append_phrase(
        &mut self,
        msg: PhrData,
        crnt_: &CrntMsrTick,
        estk_: &mut ElapseStack,
        pbp: PartBasicPrm,
        during_play: bool,
    ) {
        // whole_tick: 前後に１小節分追加
        let msg = self.add_float_part(msg, crnt_.tick_for_onemsr as i16);

        match msg.vari {
            PhraseAs::Normal => {
                // Normal Phrase
                self.phr_stock[0] = msg;
                self.phr_idx = 0;
                let phase = self.get_crnt_phr_phase(crnt_);
                #[cfg(feature = "verbose")]
                println!("PhrLoopManager::append_phrase: phase: {:?}", phase);
                if during_play {
                    match phase {
                        LoopPhase::BeforeBeginPhr => {
                            // Phrase Loop の開始前
                            self.begin_phr_ev = true;
                        }
                        LoopPhase::DuringBeginPhr => {
                            // Phrase Loop の開始時
                            self.begin_phr_ev = false;
                            self.gen_phr_alternately(crnt_, estk_, pbp, 0); // Replace
                        }
                        LoopPhase::AfterBeginCnct => {
                            // Phrase Loop の begin_cnct 後の小節
                            self.begin_phr_ev = false;
                            if self.whole_tick() <= self.phr_stock[0].whole_tick as i32 {
                                // 新しい Phrase の方が長い場合
                                self.chasing_play = true;
                            }
                        }
                        LoopPhase::OneBarBeforeEndCnct => {
                            // Phrase Loop の end_cnct 前の小節
                            self.begin_phr_ev = false;
                            self.gen_phr_alternately(crnt_, estk_, pbp, 0); // Alternate
                        }
                        //LoopPhase::BeforeEndPtr => {
                        _ => {
                            // Phrase Loop の end_cnct 以降の小節
                            self.begin_phr_ev = false;
                            self.gen_phr_alternately(crnt_, estk_, pbp, 0); // Alternate
                        }
                    }
                }
            }
            PhraseAs::Variation(_v) => {
                // Variation Phrase
                if let Some(idx) = self.exists_same_vari(msg.vari.clone()) {
                    self.phr_stock[idx] = msg; // 上書き
                } else {
                    self.phr_stock.push(msg); // 新規追加
                }
            }
            PhraseAs::Measure(_m) => {
                // Measure 指定 Phrase
                self.phr_stock.push(msg);
            }
        }
    }
    fn delete_ev(&mut self) {
        if self.del_a_ev {
            // instance_a を削除するイベント
            self.del_a();
            self.del_a_ev = false;
        } else if self.del_b_ev {
            // instance_b を削除するイベント
            self.del_b();
            self.del_b_ev = false;
        }
    }
    fn delete_phrase(&mut self, msg: PhrData) {
        // phrase = [] の時の処理
        if let Some(idx) = self.exists_same_vari(msg.vari) {
            if idx == 0 {
                // 0 の場合は、空の Phrase を入れ、phr_stock の要素数を0にしない
                self.phr_stock = vec![PhrData::empty()];
            } else {
                self.phr_stock.remove(idx);
            }
        }
    }
    fn exists_same_vari(&self, vari: PhraseAs) -> Option<usize> {
        self.phr_stock.iter().enumerate().find_map(
            |(i, phr)| {
                if phr.vari == vari { Some(i) } else { None }
            },
        )
    }
    fn exist_msr_phr(&self, crnt_: &CrntMsrTick) -> Option<usize> {
        for (i, phr) in self.phr_stock.iter().enumerate() {
            if phr.vari == PhraseAs::Measure((crnt_.msr as usize) + 1) {
                return Some(i);
            }
        }
        None
    }
    fn exist_vari_phr(&self, vari_num: usize) -> Option<usize> {
        for (i, phr) in self.phr_stock.iter().enumerate() {
            if phr.vari == PhraseAs::Variation(vari_num) {
                return Some(i);
            }
        }
        None
    }
    /// Phrase Loop が end prepare 小節かどうか
    fn if_end_prpr(&self, crnt_: &CrntMsrTick) -> bool {
        if self.get_crnt_phr_phase(crnt_) == LoopPhase::OneBarBeforeEndCnct {
            // Phrase Loop の end_cnct 前の小節
            return true;
        }
        false
    }
    /// 変更前後の Phrase Loop の条件を確認し、追いかけ再生を行う
    fn chasing_play(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        // Phrase Loop の begin_cnct から追いかけ再生
        if let Some(phr_now) = self.crnt_phr() {
            let elapsed_msr = crnt_.msr - phr_now.begin_phr;
            self.gen_phr_alternately(crnt_, estk, pbp, phr_now.begin_phr);
            // 新しい Phrase を早送りする
            if let Some(phr_nxt) = self.crnt_phr() {
                phr_nxt.phrase.borrow_mut().set_forward(crnt_, elapsed_msr);
            }
        }
    }
    /// 前後に１小節分の余白を追加
    fn add_float_part(&self, mut msg: PhrData, tick_for_onemsr: i16) -> PhrData {
        if msg.whole_tick != 0 {
            if msg.auftakt == 0 {
                msg.whole_tick += 2 * tick_for_onemsr; // 2 小節分
                msg.evts.iter_mut().for_each(|evt| {
                    evt.set_tick(evt.tick() + tick_for_onemsr); // 1 小節分
                });
                msg.ana.iter_mut().for_each(|ana| {
                    ana.set_tick(ana.tick() + tick_for_onemsr); // 1 小節分
                });
            } else {
                // auftakt がある場合は、auftakt の分を引く
                let rest = msg.whole_tick % tick_for_onemsr;
                msg.whole_tick = msg.whole_tick - rest + 2 * tick_for_onemsr;
            }
        }
        msg
    }
    /// 現在の PhraseLoop の状態を返す
    fn get_crnt_phr_phase(&self, crnt_: &CrntMsrTick) -> LoopPhase {
        if let Some(phr) = self.crnt_phr() {
            return phr.crnt_phase(crnt_);
        }
        LoopPhase::BeforeBeginPhr
    }
    /// 現在動作中の instance とは違う PhrLoopWrapper を生成する(交互)
    fn gen_phr_alternately(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
        replace_msr: i32, // 0以外なら置き換える小節番号
    ) {
        let phr: PhrData = self.phr_stock[self.phr_idx].clone();
        let pinst = PhrLoopWrapper::new(
            crnt_.tick_for_onemsr,
            if replace_msr != 0 {
                replace_msr
            } else {
                crnt_.msr
            },
            pbp,
            self.loop_id + 1, // loop_id をインクリメント
            self.turnnote,
            phr,
        );
        if self.a_is_gened_last {
            // instance_b を使用
            self.a_is_gened_last = false;
            self.phr_instance_b = Some(pinst.clone());
            estk.add_elapse(pinst.phrase);
            self.del_b_ev = false;
            self.del_a_ev = true; // instance_a を削除するイベント
        } else {
            // instance_a を使用
            self.a_is_gened_last = true;
            self.phr_instance_a = Some(pinst.clone());
            estk.add_elapse(pinst.phrase);
            self.del_a_ev = false;
            self.del_b_ev = true; // instance_b を削除するイベント
        }
        self.loop_id += 1; // loop_id をインクリメント
    }
    fn del_a(&mut self) {
        if self.phr_instance_a.is_some() {
            self.phr_instance_a
                .as_ref()
                .unwrap()
                .phrase
                .borrow_mut()
                .set_destroy();
        }
        self.phr_instance_a = None;
    }
    fn del_b(&mut self) {
        if self.phr_instance_b.is_some() {
            self.phr_instance_b
                .as_ref()
                .unwrap()
                .phrase
                .borrow_mut()
                .set_destroy();
        }
        self.phr_instance_b = None;
    }
}

//*******************************************************************
//          Part Struct
//*******************************************************************
pub struct Part {
    id: ElapseId,
    priority: u32,

    during_play: bool,
    keynote: u8,
    next_msr: i32,
    next_tick: i32,
    pm: PhrLoopManager,
    cm: CmpsLoopMediator,
    flow: Option<Rc<RefCell<Flow>>>,
    sync_next_msr_flag: bool,
    start_flag: bool,
}
//*******************************************************************
impl Part {
    pub fn new(num: u32, flow: Option<Rc<RefCell<Flow>>>) -> Rc<RefCell<Part>> {
        let new_id = ElapseId {
            pid: 0,
            sid: num,
            elps_type: ElapseType::TpPart,
        };
        Rc::new(RefCell::new(Self {
            id: new_id,
            priority: PRI_PART,
            during_play: false,
            keynote: 0,
            next_msr: 0,
            next_tick: 0,
            pm: PhrLoopManager::new(),
            cm: CmpsLoopMediator::new(),
            flow,
            sync_next_msr_flag: false,
            start_flag: false,
        }))
    }
    pub fn change_key(&mut self, knt: u8) {
        self.keynote = knt; // 0-11
        if let Some(fl) = &self.flow {
            fl.borrow_mut().set_keynote(knt);
        }
        //self.pm.state_reserve = true;
    }
    pub fn rcv_phr_msg(&mut self, msg: PhrData, crnt_: &CrntMsrTick, estk_: &mut ElapseStack) {
        let pbp = PartBasicPrm {
            part_num: self.id.sid,
            keynote: self.keynote,
            sync_flag: self.sync_next_msr_flag,
        };
        self.pm.rcv_phrase(msg, crnt_, estk_, pbp, self.during_play);
    }
    pub fn del_phr(&mut self) {
        self.pm.del_phrase();
    }
    pub fn rcv_cmps_msg(&mut self, msg: ChordData, (msr, tick): (i32, i32)) {
        self.cm.rcv_cmp(msg, msr, tick);
    }
    /// CmpsLoopMediator を取得する
    pub fn get_cmps_med(&mut self) -> &mut CmpsLoopMediator {
        &mut self.cm
    }
    pub fn get_phr(&self) -> Option<&Rc<RefCell<PhraseLoop>>> {
        self.pm.get_phr()
    }
    pub fn get_flow(&self) -> Option<Rc<RefCell<Flow>>> {
        self.flow.clone()
    }
    pub fn set_turnnote(&mut self, tn: i16) {
        self.pm.set_turnnote(tn);
    }
    /// sync command 発行時にコールされる
    pub fn set_sync(&mut self) {
        //self.pm.state_reserve = true;
        self.cm.state_reserve = true;
        self.sync_next_msr_flag = true;
    }
    pub fn gen_part_indicator(&mut self, crnt_: &CrntMsrTick) -> PartUi {
        let mut exist = true;
        let mut flow = false;
        let mut chord_name = "".to_string();
        let mut msr_in_loop = 0;
        let mut all_msrs = 0;
        if !self.during_play {
            exist = false;
        } else if self.pm.whole_tick() != 0 {
            if let Some(a) = self.pm.gen_msrcnt(crnt_.msr) {
                (msr_in_loop, all_msrs) = a;
            } else {
                exist = false;
            }
            chord_name = self.get_cmps_med().get_chord_name(crnt_);
        } else if self.flow.is_some() {
            chord_name = self.get_cmps_med().get_chord_name(crnt_).to_string();
            flow = true;
        } else {
            exist = false;
        }
        PartUi {
            exist,
            msr_in_loop,
            all_msrs,
            flow,
            chord_name,
        }
    }
    pub fn rcv_midi_in(
        &mut self,
        estk_: &mut ElapseStack,
        crnt_: &CrntMsrTick,
        status: u8,
        locate: u8,
        vel: u8,
    ) {
        if let Some(fl) = &self.flow {
            fl.borrow_mut().rcv_midi(estk_, crnt_, status, locate, vel);
        }
    }
    /// Phrase Variation があるか確認し、あれば予約する
    fn check_variation(&mut self, crnt_: &CrntMsrTick) {
        let vari_num = self.get_cmps_med().get_vari_num(crnt_) as usize;
        self.pm.reserve_vari(vari_num);
    }
}
//*******************************************************************
impl Elapse for Part {
    /// id を得る
    fn id(&self) -> ElapseId {
        self.id
    }
    /// priority を得る
    fn prio(&self) -> u32 {
        self.priority
    }
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32) {
        (self.next_msr, self.next_tick)
    }
    /// User による start/play 時にコールされる msr:開始小節番号
    fn start(&mut self, msr: i32) {
        self.during_play = true;
        self.start_flag = true;
        self.next_msr = msr;
        self.next_tick = 0;
        self.cm.start();
        self.pm.start();
    }
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {
        self.during_play = false;
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.pm = PhrLoopManager::new();
        self.cm = CmpsLoopMediator::new();
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        let pbp = PartBasicPrm {
            part_num: self.id.sid,
            keynote: self.keynote,
            sync_flag: self.sync_next_msr_flag,
        };
        if self.start_flag {
            // Start 直後
            self.cm.msrtop(crnt_, estk, pbp);
            self.check_variation(crnt_);
            self.pm.msrtop(crnt_, estk, pbp);
            self.start_flag = false;
            // 小節最後の tick をセット
            self.next_msr += 1;
            self.next_tick = 0;
        } else {
            // 小節先頭
            self.cm.msrtop(crnt_, estk, pbp);
            self.check_variation(crnt_);
            self.pm.msrtop(crnt_, estk, pbp);
            self.sync_next_msr_flag = false;
            // 次の小節の頭をセット
            self.next_msr += 1;
            self.next_tick = 0;
        }
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 静的に存在するので、destroy はしない
    fn destroy_me(&self) -> bool {
        false
    }
}
