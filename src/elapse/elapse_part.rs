//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::cmp::Ordering;
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

//*******************************************************************
//          Phrase Loop Manager Struct
//*******************************************************************
struct PhrLoopManager {
    first_msr_num: i32,
    max_loop_msr: i32, // from whole_tick
    whole_tick: i32,
    loop_id: u32,            // loop sid
    phr_stock: Vec<PhrData>, // 0: Normal
    phr_idx: usize,          // 0: Normal
    loop_phrase: Option<Rc<RefCell<PhraseLoop>>>,
    vari_reserve: Option<usize>, // 1-9: rsv, None: Normal
    state_reserve: bool,
    turnnote: i16,
}
//*******************************************************************
impl PhrLoopManager {
    pub fn new() -> Self {
        Self {
            first_msr_num: 0,
            max_loop_msr: 0,
            whole_tick: 0,
            loop_id: 0,
            phr_stock: vec![PhrData::empty()],
            phr_idx: 0,
            loop_phrase: None,
            vari_reserve: None,
            state_reserve: false,
            turnnote: DEFAULT_TURNNOTE,
        }
    }
    pub fn start(&mut self) {
        self.clear_phr_prm();
        self.state_reserve = true;
    }
    pub fn msrtop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        let mut phr = &self.phr_stock[0]; // Normal Phrase

        let auftakt_cond_vari = || -> bool {
            // variation 再生時の弱起auftaktの条件
            self.max_loop_msr != 0 &&
            (crnt_.msr - self.first_msr_num)%(self.max_loop_msr) < self.max_loop_msr && // 残り１小節以上
            phr.whole_tick as i32 >= crnt_.tick_for_onemsr*2 // 新しい Phrase が２小節以上
        };
        let auftakt_cond = || -> bool {
            // 通常の弱起auftaktの条件
            self.max_loop_msr != 0
                && (crnt_.msr - self.first_msr_num) % (self.max_loop_msr) == self.max_loop_msr - 1
                && phr.whole_tick as i32 >= crnt_.tick_for_onemsr * 2 // 新しい Phrase が２小節以上
        };

        if let Some(idx) = self.exist_msr_phr(crnt_) {
            // Measure 指定があった場合
            self.phr_idx = idx;
            self.proc_replace_loop(crnt_, estk, pbp);
        } else if let Some(vr) = self.vari_reserve {
            // Variation 指定があった場合
            if let Some(idx) = self.exist_vari_phr(vr) {
                // 指定された Variation が存在した場合
                self.phr_idx = idx;
                phr = &self.phr_stock[idx];
                if phr.auftakt != 0 {
                    // variation : 今再生している Phrase が残り１小節以上
                    if auftakt_cond_vari() { // ◆◆ Auftakt ◆◆
                        let prm = (crnt_.msr, crnt_.tick_for_onemsr);
                        self.new_loop(prm, estk, pbp);
                    }
                } else {
                    let sr = self.state_reserve; // イベントがあれば保持
                    self.proc_replace_loop(crnt_, estk, pbp);
                    self.state_reserve = sr;
                }
            }
            self.vari_reserve = None; // 予約は解除
        } else if self.state_reserve {
            // User による Phrase 入力イベントがあった場合
            if phr.auftakt != 0 && auftakt_cond() { // ◆◆ Auftakt ◆◆
                self.state_reserve = false;
                let prm = (crnt_.msr, crnt_.tick_for_onemsr);
                self.vari_reserve = None; // 予約は解除
                self.new_loop(prm, estk, pbp);
            } else {
                self.phr_idx = 0;
                if crnt_.msr == 0 {
                    // 今回 start したとき
                    self.proc_new_loop_by_evt(crnt_, estk, pbp);
                } else if self.max_loop_msr == 0 {
                    // データのない状態から、今回初めて指定された時
                    self.proc_new_loop_by_evt(crnt_, estk, pbp);
                } else if self.check_last_msr(crnt_) {
                    // 前小節にて Loop Obj が終了した時
                    self.proc_new_loop_by_evt(crnt_, estk, pbp);
                } else if self.max_loop_msr != 0 && pbp.sync_flag {
                    // sync コマンドによる強制リセット
                    self.proc_replace_loop(crnt_, estk, pbp);
                } else {
                    // 現在の Loop Obj が終了していない時
                    // 現在の Phrase より新しい Phrase の whole_tick が大きい場合、
                    // 新しい Phrase を早送りして更新する
                    if self.phr_stock[0].whole_tick as i32 >= self.whole_tick {
                        self.proc_forward_by_evt(crnt_, estk, pbp);
                    }
                }
            } 
        } else if self.phr_stock.len() > self.phr_idx {
            // イベントがない場合
            phr = &self.phr_stock[self.phr_idx];
            if phr.whole_tick != 0 && auftakt_cond() && phr.do_loop { // ◆◆ Auftakt ◆◆
                let prm = (crnt_.msr, crnt_.tick_for_onemsr);
                self.vari_reserve = None; // 予約は解除
                self.new_loop(prm, estk, pbp);
            } else {
                self.msrtop_with_no_events(crnt_, estk, pbp);
            }
        } else {
            panic!("Phrase index is too large!"); // ここに来るのはおかしい
        }
    }
    pub fn rcv_phr(&mut self, msg: PhrData) {
        if msg.evts.is_empty() && msg.whole_tick == 0 {
            // phrase = [] の時の処理
            if let Some(idx) = self.exists_same_vari(msg.vari) {
                if idx == 0 {
                    // 0 の場合は、空の Phrase を入れ、phr_stock の要素数を0にしない
                    self.phr_stock = vec![PhrData::empty()];
                } else {
                    self.phr_stock.remove(idx);
                }
                match idx.cmp(&self.phr_idx) {
                    Ordering::Equal => {
                        // 今再生している Phrase が削除された
                        //self.del_loop_phrase(); // これを有効にすると即時消音
                        self.phr_idx = 0;
                    }
                    Ordering::Less => {
                        // 今再生している Phrase より前の Phrase が削除されたので、index をデクリメント
                        self.phr_idx -= 1;
                    }
                    _ => {}
                }
            }
        } else {
            match msg.vari {
                PhraseAs::Normal => {
                    self.phr_stock[0] = msg;
                    self.state_reserve = true;
                }
                PhraseAs::Variation(_v) => {
                    if let Some(num) = self.exists_same_vari(msg.clone().vari) {
                        self.phr_stock[num] = msg; // 上書き
                    } else {
                        self.phr_stock.push(msg); // 追加
                    }
                }
                PhraseAs::Measure(msr) => {
                    let mut msg_modified = msg.clone();
                    msg_modified.vari = PhraseAs::Measure(msr - 1); // 0origin
                    self.phr_stock.push(msg_modified);
                }
            }
        }
    }
    pub fn del_phr(&mut self) {
        self.del_loop_phrase();
        self.phr_stock = vec![PhrData::empty()];
        self.clear_phr_prm();
        self.state_reserve = true;
    }
    pub fn get_phr(&self) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.loop_phrase.clone() // 重いclone()?
    }
    pub fn gen_msrcnt(&self, crnt_msr: i32) -> Option<(i32, i32)> {
        if let Some(phr) = &self.loop_phrase {
            let denomirator = self.max_loop_msr;
            let numerator = crnt_msr - phr.borrow().first_msr_num() + 1; // 1origin
            //format!("{}/{}", numerator, denomirator)
            Some((numerator, denomirator))
        } else {
            None
        }
    }
    pub fn set_turnnote(&mut self, tn: i16) {
        self.turnnote = tn;
    }
    pub fn reserve_vari(&mut self, vari_num: usize) {
        if vari_num != 0 {
            self.vari_reserve = Some(vari_num); // 1-9
        }
    }
//*******************************************************************
    fn exists_same_vari(&self, vari: PhraseAs) -> Option<usize> {
        self.phr_stock.iter().enumerate().find_map(
            |(i, phr)| {
                if phr.vari == vari { Some(i) } else { None }
            },
        )
    }
    fn del_loop_phrase(&mut self) {
        if let Some(phr) = self.loop_phrase.as_mut() {
            phr.borrow_mut().set_destroy();
        }
    }
    fn exist_msr_phr(&self, crnt_: &CrntMsrTick) -> Option<usize> {
        for (i, phr) in self.phr_stock.iter().enumerate() {
            if phr.vari == PhraseAs::Measure(crnt_.msr as usize) {
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
    fn msrtop_with_no_events(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
    ) {
        if self.phr_stock[0].do_loop {
            // 何も外部からのトリガーがなく、loop 指定の場合
            if self.check_last_msr(crnt_) {
                // 今の Loop が終わったので、新しい Loop.Obj を生成する
                self.phr_idx = 0;
                self.proc_new_loop_repeatedly(crnt_, estk, pbp);
            } else {
                // 通常の Loop 中
            }
        } else if self.check_last_msr(crnt_) {
            // loop 指定でない場合
            self.clear_phr_prm();
        }
    }
    fn clear_phr_prm(&mut self) {
        self.first_msr_num = 0;
        self.max_loop_msr = 0;
        self.whole_tick = 0;
        self.loop_phrase = None;
    }
    fn check_last_msr(&self, crnt_: &CrntMsrTick) -> bool {
        self.max_loop_msr != 0 && (crnt_.msr - self.first_msr_num) % (self.max_loop_msr) == 0
    }
    fn proc_new_loop_by_evt(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
    ) {
        self.state_reserve = false;
        let prm = (crnt_.msr, crnt_.tick_for_onemsr);
        self.new_loop(prm, estk, pbp);
    }
    fn proc_new_loop_repeatedly(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
    ) {
        let prm = (crnt_.msr, crnt_.tick_for_onemsr);
        self.new_loop(prm, estk, pbp);
    }
    fn proc_replace_loop(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
    ) {
        self.state_reserve = false;
        //self.del_loop_phrase(); 今動作している Phrase を即座に消す
        let prm = (crnt_.msr, crnt_.tick_for_onemsr);
        self.new_loop(prm, estk, pbp);
    }
    fn proc_forward_by_evt(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
    ) {
        self.state_reserve = false;
        self.del_loop_phrase();

        // その時の beat 情報で、whole_tick を loop_measure に換算
        self.whole_tick = self.phr_stock[self.phr_idx].whole_tick as i32;
        let tick_for_onemsr = crnt_.tick_for_onemsr;
        let plus_one = if self.whole_tick % tick_for_onemsr == 0 {
            0
        } else {
            1
        };
        self.max_loop_msr = self.whole_tick / tick_for_onemsr + plus_one;

        // Phrase の新規生成
        self.loop_id += 1;

        let lp = PhraseLoop::new(
            self.loop_id,
            pbp.part_num,
            PhraseLoopParam::new(
                pbp.keynote,
                self.first_msr_num,
                self.phr_stock[self.phr_idx].evts.to_vec(),
                self.phr_stock[self.phr_idx].ana.to_vec(),
                self.whole_tick,
                self.turnnote,
            ),
        );

        // Phrase の更新
        self.loop_phrase = Some(Rc::clone(&lp));
        estk.add_elapse(lp);
        #[cfg(feature = "verbose")]
        println!("Replace Phrase Loop! --whole tick: {}", self.whole_tick);

        // 新しい Phrase を早送りする
        if let Some(phr) = self.loop_phrase.as_mut() {
            let elapsed_msr = crnt_.msr - self.first_msr_num;
            phr.borrow_mut().set_forward(crnt_, elapsed_msr);
        }
    }
    fn new_loop(&mut self, prm: (i32, i32), estk: &mut ElapseStack, pbp: PartBasicPrm) {
        self.first_msr_num = prm.0;

        // Phrase の更新
        let phrlen = self.phr_stock[self.phr_idx].evts.len();
        if phrlen != 0 {
            self.gen_new_loop(prm, estk, pbp);
        } else {
            // 1小節分の値を入れておき、次の小節で new_loop に入るようにする
            self.whole_tick = prm.1;
            self.max_loop_msr = 1;
            self.loop_phrase = None;
        }
        self.vari_reserve = None; // 予約は解除
    }
    fn gen_new_loop(&mut self, prm: (i32, i32), estk: &mut ElapseStack, pbp: PartBasicPrm) {
        // 新しいデータが来ていれば、新たに Loop Obj.を生成
        self.whole_tick = self.phr_stock[self.phr_idx].whole_tick as i32;
        if self.whole_tick == 0 {
            self.state_reserve = true; // 次小節冒頭で呼ばれるように
            self.loop_phrase = None;
            self.max_loop_msr = 0;
            return;
        }

        // その時の beat 情報で、whole_tick を loop_measure に換算
        let plus_one = if self.whole_tick % prm.1 == 0 { 0 } else { 1 };
        self.max_loop_msr = self.whole_tick / prm.1 + plus_one;

        self.loop_id += 1;
        let lp = PhraseLoop::new(
            self.loop_id,
            pbp.part_num,
            PhraseLoopParam::new(
                pbp.keynote,
                prm.0,
                self.phr_stock[self.phr_idx].evts.to_vec(),
                self.phr_stock[self.phr_idx].ana.to_vec(),
                self.whole_tick,
                self.turnnote,
            ),
        );

        self.loop_phrase = Some(Rc::clone(&lp));
        estk.add_elapse(lp);
        #[cfg(feature = "verbose")]
        println!("New Phrase Loop! --whole tick: {}", self.whole_tick);
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
        self.pm.state_reserve = true;
    }
    pub fn rcv_phr_msg(&mut self, msg: PhrData) {
        self.pm.rcv_phr(msg);
    }
    pub fn del_phr(&mut self) {
        self.pm.del_phr();
    }
    pub fn rcv_cmps_msg(&mut self, msg: ChordData, (msr, tick): (i32, i32)) {
        self.cm.rcv_cmp(msg, msr, tick);
    }
    /// CmpsLoopMediator を取得する
    pub fn get_cmps_med(&mut self) -> &mut CmpsLoopMediator {
        &mut self.cm
    }
    pub fn get_phr(&self) -> Option<Rc<RefCell<PhraseLoop>>> {
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
        self.pm.state_reserve = true;
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
        } else if self.pm.whole_tick != 0 {
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
