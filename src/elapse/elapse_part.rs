//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse::*;
use super::elapse_loop::{CompositionLoop, Loop, PhraseLoop};
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
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
    loop_id: u32, // loop sid
    new_data_stock: Vec<PhrData>,
    whole_tick_stock: [i16; MAX_PHRASE],
    new_ana_stock: Vec<AnaData>,
    loop_phrase: Option<Rc<RefCell<PhraseLoop>>>,
    vari_reserve: usize, // 0:no rsv, 1-9: rsv
    state_reserve: bool,
    turnnote: i16,
}
impl PhrLoopManager {
    pub fn new() -> Self {
        let (pstock, astock) = PhrLoopManager::gen_empty_stock();
        Self {
            first_msr_num: 0,
            max_loop_msr: 0,
            whole_tick: 0,
            loop_id: 0,
            new_data_stock: pstock,
            whole_tick_stock: [0; MAX_PHRASE],
            new_ana_stock: astock,
            loop_phrase: None,
            vari_reserve: 0,
            state_reserve: false,
            turnnote: DEFAULT_TURNNOTE,
        }
    }
    pub fn start(&mut self) {
        self.clear_phr_prm();
        self.state_reserve = true;
    }
    pub fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        // auftakt は別枠
        if self.proc_auftakt(crnt_, estk, pbp) {
            return;
        }

        if self.vari_reserve != 0 {
            // variation 指定があった場合
            let sr = self.state_reserve; // イベントがあれば保持
            self.proc_replace_loop(crnt_, estk, pbp);
            self.state_reserve = sr;
        }
        // User による Phrase 入力があった場合
        else if self.state_reserve {
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
                // state_reserve は持ち越す
            }
        } else if self.new_data_stock[0].do_loop {
            // 何も外部からのトリガーがなく、loop 指定の場合
            if self.check_last_msr(crnt_) {
                // 今の Loop が終わったので、同じ Loop.Obj を生成する
                self.proc_new_loop_repeatedly(crnt_, estk, pbp);
            } else {
                // 通常の Loop 中
            }
        } else if self.check_last_msr(crnt_) {
            // loop 指定でない場合
            self.clear_phr_prm();
        }
    }
    pub fn rcv_phr(&mut self, msg: PhrData, vari_num: usize) {
        if vari_num < MAX_PHRASE {
            self.whole_tick_stock[vari_num] = msg.whole_tick;
            if msg.evts.len() == 0 && msg.whole_tick == 0 {
                self.new_data_stock[vari_num] = PhrData::empty();
            } else {
                self.new_data_stock[vari_num] = msg;
            }
            if vari_num == 0 {
                self.state_reserve = true;
            }
        }
    }
    pub fn rcv_ana(&mut self, msg: AnaData, vari_num: usize) {
        if vari_num < MAX_PHRASE {
            if msg.evts.len() == 0 {
                self.new_ana_stock[vari_num] = AnaData::empty();
            } else {
                self.new_ana_stock[vari_num] = msg;
            }
            if vari_num == 0 {
                self.state_reserve = true;
            }
        }
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
        self.vari_reserve = vari_num; // 1-9
    }
    fn gen_empty_stock() -> (Vec<PhrData>, Vec<AnaData>) {
        let mut pstock: Vec<PhrData> = Vec::new();
        for _ in 0..MAX_PHRASE {
            pstock.push(PhrData::empty());
        }
        let mut astock: Vec<AnaData> = Vec::new();
        for _ in 0..MAX_PHRASE {
            astock.push(AnaData::empty());
        }
        (pstock, astock)
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
    fn proc_auftakt(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        pbp: PartBasicPrm,
    ) -> bool {
        let phr = &self.new_data_stock[self.vari_reserve];
        if phr.auftakt == 0 {
            return false;
        } // auftakt phrase でなければすぐに return

        let auftakt_cond_vari = || -> bool {
            // variation 再生時の弱起auftaktの条件
            self.max_loop_msr != 0 &&
            (crnt_.msr - self.first_msr_num)%(self.max_loop_msr) <= self.max_loop_msr-1 && // 残り１小節以上
            phr.whole_tick as i32 >= crnt_.tick_for_onemsr*2 // 新しい Phrase が２小節以上
        };
        let auftakt_cond = || -> bool {
            // 通常の弱起auftaktの条件
            self.max_loop_msr != 0
                && (crnt_.msr - self.first_msr_num) % (self.max_loop_msr) == self.max_loop_msr - 1
                && phr.whole_tick as i32 >= crnt_.tick_for_onemsr * 2 // 新しい Phrase が２小節以上
        };

        if self.vari_reserve != 0 {
            // variation : 今再生している Phrase が残り１小節以上
            if auftakt_cond_vari() {
                let prm = (crnt_.msr, crnt_.tick_for_onemsr);
                self.new_loop(prm, estk, pbp);
                return true;
            }
        } else if self.state_reserve {
            // User input : 今再生している Phrase が残り１小節
            if auftakt_cond() {
                self.state_reserve = false;
                let prm = (crnt_.msr, crnt_.tick_for_onemsr);
                self.new_loop(prm, estk, pbp);
                return true;
            }
        } else {
            // repeat : 今再生している Phrase が残り１小節 かつ loop設定の場合
            if auftakt_cond() && self.new_data_stock[0].do_loop {
                let prm = (crnt_.msr, crnt_.tick_for_onemsr);
                self.new_loop(prm, estk, pbp);
                return true;
            }
        }
        false
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
        if let Some(phr) = self.loop_phrase.as_mut() {
            phr.borrow_mut().set_destroy();
        }
        let prm = (crnt_.msr, crnt_.tick_for_onemsr);
        self.new_loop(prm, estk, pbp);
    }
    fn new_loop(&mut self, prm: (i32, i32), estk: &mut ElapseStack, pbp: PartBasicPrm) {
        self.first_msr_num = prm.0;

        // Phrase の更新
        let phrlen = self.new_data_stock[self.vari_reserve].evts.len();
        let analen = self.new_ana_stock[self.vari_reserve].evts.len();
        if phrlen != 0 && analen != 0 {
            self.gen_new_loop(prm, estk, pbp);
        } else {
            // 1小節分の値を入れておき、次の小節で new_loop に入るようにする
            self.whole_tick = prm.1;
            self.max_loop_msr = 1;
            self.loop_phrase = None;
        }
        self.vari_reserve = 0;
    }
    fn gen_new_loop(&mut self, prm: (i32, i32), estk: &mut ElapseStack, pbp: PartBasicPrm) {
        // 新しいデータが来ていれば、新たに Loop Obj.を生成
        self.whole_tick = self.whole_tick_stock[self.vari_reserve] as i32;
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
            pbp.keynote,
            prm.0,
            self.new_data_stock[self.vari_reserve].evts.to_vec(),
            self.new_ana_stock[self.vari_reserve].evts.to_vec(),
            self.whole_tick,
            self.turnnote,
        );

        self.loop_phrase = Some(Rc::clone(&lp));
        estk.add_elapse(lp);
        #[cfg(feature = "verbose")]
        println!("New Phrase Loop! --whole tick: {}", self.whole_tick);
    }
}

//*******************************************************************
//          Composition Loop Manager Struct
//*******************************************************************
struct CmpsLoopManager {
    first_msr_num: i32,
    max_loop_msr: i32,
    whole_tick: i32,
    loop_id: u32, // loop sid
    new_data_stock: Vec<ChordEvt>,
    whole_tick_stock: i16,
    loop_cmps: Option<Rc<RefCell<CompositionLoop>>>,
    state_reserve: bool,
    do_loop: bool,
}
impl CmpsLoopManager {
    pub fn new() -> Self {
        Self {
            first_msr_num: 0,
            max_loop_msr: 0,
            whole_tick: 0,
            loop_id: 0,
            new_data_stock: Vec::new(),
            whole_tick_stock: 0,
            loop_cmps: None,
            state_reserve: false,
            do_loop: true,
        }
    }
    pub fn start(&mut self) {
        self.clear_cmp_prm();
        self.state_reserve = true;
    }
    pub fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = false;
                self.new_loop(crnt_, estk, pbp);
            } else if self.max_loop_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_, estk, pbp);
            } else if self.max_loop_msr != 0
                && (crnt_.msr - self.first_msr_num) % (self.max_loop_msr) == 0
            {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_, estk, pbp);
            } else if self.max_loop_msr != 0 && pbp.sync_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                if let Some(phr) = self.loop_cmps.as_mut() {
                    phr.borrow_mut().set_destroy();
                }
                self.new_loop(crnt_, estk, pbp);
            } else {
                // 現在の Loop Obj が終了していない時
                // state_reserve は持ち越す
            }
        } else if self.max_loop_msr != 0
            && (crnt_.msr - self.first_msr_num) % self.max_loop_msr == 0
        {
            if self.do_loop {
                // 同じ Loop.Obj を生成する
                self.new_loop(crnt_, estk, pbp);
            } else {
                self.clear_cmp_prm();
            }
        }
    }
    pub fn rcv_cmp(&mut self, msg: ChordData) {
        if msg.evts.len() == 0 && msg.whole_tick == 0 {
            self.new_data_stock = Vec::new();
        } else {
            self.new_data_stock = msg.evts;
        }
        self.state_reserve = true;
        self.whole_tick_stock = msg.whole_tick;
        self.do_loop = msg.do_loop;
    }
    pub fn get_cmps(&self) -> Option<Rc<RefCell<CompositionLoop>>> {
        self.loop_cmps.clone() // 重いclone()?
    }
    pub fn gen_chord_name(&self) -> String {
        if let Some(cmps) = &self.loop_cmps {
            cmps.borrow().get_chord_name()
        } else {
            String::from("")
        }
    }
    fn clear_cmp_prm(&mut self) {
        self.first_msr_num = 0;
        self.max_loop_msr = 0;
        self.whole_tick = 0;
        self.loop_cmps = None;
        self.do_loop = true;
    }
    fn new_loop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        // 新たに Loop Obj.を生成
        if self.new_data_stock.len() != 0 {
            #[cfg(feature = "verbose")]
            println!("New Composition Loop! M:{:?},T:{:?}", crnt_.msr, crnt_.tick);
            self.first_msr_num = crnt_.msr; // 計測開始の更新
            self.whole_tick = self.whole_tick_stock as i32;

            // その時の beat 情報で、whole_tick を loop_measure に換算
            let plus_one = if self.whole_tick % crnt_.tick_for_onemsr == 0 {
                0
            } else {
                1
            };
            self.max_loop_msr = self.whole_tick / crnt_.tick_for_onemsr + plus_one;

            if self.whole_tick == 0 {
                self.state_reserve = true; // 次小節冒頭で呼ばれるように
                self.loop_cmps = None;
                return;
            }

            self.loop_id += 1;
            let cmplp = CompositionLoop::new(
                self.loop_id,
                pbp.part_num,
                pbp.keynote,
                crnt_.msr,
                self.new_data_stock.to_vec(),
                self.whole_tick,
            );
            self.loop_cmps = Some(Rc::clone(&cmplp));
            estk.add_elapse(cmplp);
        } else {
            // 新しい Composition が空のとき
            self.max_loop_msr = 0;
            self.whole_tick = 0;
            self.state_reserve = true;
            self.loop_cmps = None;
        }
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
    cm: CmpsLoopManager,
    flow: Option<Rc<RefCell<Flow>>>,
    sync_next_msr_flag: bool,
    start_flag: bool,
}
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
            cm: CmpsLoopManager::new(),
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
    pub fn rcv_phr_msg(&mut self, msg: PhrData, vari_num: usize) {
        self.pm.rcv_phr(msg, vari_num);
    }
    pub fn rcv_cmps_msg(&mut self, msg: ChordData) {
        self.cm.rcv_cmp(msg);
    }
    pub fn rcv_ana_msg(&mut self, msg: AnaData, vari_num: usize) {
        self.pm.rcv_ana(msg, vari_num);
    }
    pub fn get_phr(&self) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.pm.get_phr()
    }
    pub fn get_cmps(&self) -> Option<Rc<RefCell<CompositionLoop>>> {
        self.cm.get_cmps()
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
    pub fn gen_part_indicator(&self, crnt_: &CrntMsrTick) -> PartUi {
        let mut exist = true;
        let mut flow = false;
        let mut chord_name = "".to_string();
        let mut msr_in_loop = 0;
        let mut all_msrs = 0;
        if self.pm.whole_tick != 0 {
            if let Some(a) = self.pm.gen_msrcnt(crnt_.msr) {
                (msr_in_loop, all_msrs) = a;
            } else {
                exist = false;
            }
            chord_name = self.cm.gen_chord_name();
        } else if self.flow.is_some() && self.during_play {
            chord_name = self.cm.gen_chord_name().to_string();
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
    pub fn set_phrase_vari(&mut self, vari_num: usize) {
        self.pm.reserve_vari(vari_num);
    }
    pub fn set_loop_end(&mut self) {
        // nothing to do
    }
}
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
        self.cm = CmpsLoopManager::new();
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
            self.cm.process(crnt_, estk, pbp);
            self.pm.process(crnt_, estk, pbp);
            self.start_flag = false;
            // 小節最後の tick をセット
            self.next_tick = crnt_.tick_for_onemsr - 1;
        } else if self.next_tick != 0 {
            // 小節最後のtick
            let cm_crnt = CrntMsrTick {
                msr: self.next_msr + 1,
                tick: 0,
                tick_for_onemsr: crnt_.tick_for_onemsr,
            };
            self.cm.process(&cm_crnt, estk, pbp);
            // 次の小節の頭をセット
            self.next_msr = self.next_msr + 1;
            self.next_tick = 0;
        } else {
            // 小節先頭
            self.pm.process(crnt_, estk, pbp);
            self.sync_next_msr_flag = false;
            // 小節最後の tick をセット
            self.next_tick = crnt_.tick_for_onemsr - 1;
        }
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {
        false
    }
}
