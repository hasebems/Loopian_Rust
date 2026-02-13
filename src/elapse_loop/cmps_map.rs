//  Created by Hasebe Masahiko on 2025/05/03
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
pub type ChordEvtMap = Vec<Vec<Option<(ChordEvt, bool)>>>;
pub type PedalEvtMap = Vec<Vec<Option<(PedalEvt, bool)>>>;
use crate::cmd::txt2seq_cmps::*;
use crate::elapse::elapse_part::PartBasicPrm;
use crate::elapse::stack_elapse::ElapseStack;
use crate::elapse::tickgen::CrntMsrTick;
use crate::lpnlib::*;

//*******************************************************************
//          Composition Loop Struct
//*******************************************************************
#[derive(Clone, Debug)]
pub struct CompositionMap {
    chord_map: ChordEvtMap,  // [msr][beat][num]
    damper_map: PedalEvtMap, // [msr][beat][num]
    vari_map: Vec<Option<i16>>,

    // for Composition
    first_msr_num: i32,
    chord_name: String,
    root_for_ui: i16,
    tbl_for_ui: i16,
    vari_num: i16,
    whole_tick: i32,
    max_msr: usize,
    max_beat: usize,
}
impl CompositionMap {
    pub fn new(evts: Vec<CmpEvt>, whole_tick: i32, max_msr: usize, max_beat: usize) -> Self {
        let (chord_map, damper_map, vari_map) =
            Self::unfold_chord_pedal_evt(evts, max_msr, max_beat, whole_tick);
        Self {
            chord_map,
            damper_map,
            vari_map,
            first_msr_num: 0,
            chord_name: String::new(),
            root_for_ui: NO_ROOT,
            tbl_for_ui: NO_TABLE,
            vari_num: 0,
            whole_tick,
            max_msr,
            max_beat,
        }
    }
    fn unfold_chord_pedal_evt(
        evts: Vec<CmpEvt>,
        msr: usize,
        beat: usize,
        whole_tick: i32,
    ) -> (ChordEvtMap, PedalEvtMap, Vec<Option<i16>>) {
        let mut chord_map = vec![vec![None; beat]; msr];
        let mut pedal_map = vec![vec![None; beat]; msr];
        let mut vari_map = vec![None; msr];
        let tick_for_onemsr = whole_tick as usize / msr;
        let tick_for_onebeat = tick_for_onemsr / beat;
        let mut crnt_idx = 0;
        let mut last_chord = None;
        let last_pedal: Option<PedalEvt> = None;
        let max_len = evts.len();
        // 1小節の中に、1拍ごとに分けて、イベントを展開する
        for i in 0..msr {
            let chd_m_map = &mut chord_map[i];
            let ped_m_map = &mut pedal_map[i];
            for j in 0..beat {
                let crnt_tick = (i * tick_for_onemsr + j * tick_for_onebeat) as i16;
                if crnt_idx < max_len && evts[crnt_idx].tick() <= crnt_tick {
                    // 同タイミングに複数イベントがある場合のため、loopにて処理
                    loop {
                        let evt = evts[crnt_idx].clone();
                        match evt {
                            CmpEvt::Chord(cd) => {
                                chd_m_map[j] = Some((cd.clone(), true));
                                last_chord = Some(cd);
                            }
                            CmpEvt::Vari(v) => {
                                vari_map[i] = Some(v.vari);
                            }
                            _ => {}
                        }
                        crnt_idx += 1;
                        if crnt_idx < max_len && evts[crnt_idx].tick() <= crnt_tick {
                            continue;
                        } else {
                            break;
                        }
                    }
                } else {
                    if let Some(ref evt) = last_chord {
                        chd_m_map[j] = Some((evt.clone(), false));
                    }
                    if let Some(ref evt) = last_pedal {
                        ped_m_map[j] = Some((evt.clone(), false));
                    }
                }
            }
        }
        #[cfg(feature = "verbose")]
        println!("Unfolded Composition!");
        (chord_map, pedal_map, vari_map)
    }
    pub fn reunfold(&mut self, new_beat: i32, whole_tick: i32) {
        // beat の変更に伴い、chord_map を再構築する
        let mut new_chord_map = vec![vec![None; new_beat as usize]; self.max_msr];
        let mut new_damper_map = vec![vec![None; new_beat as usize]; self.max_msr];
        for i in 0..self.max_msr {
            let chd_m_map = &mut new_chord_map[i];
            let ped_m_map = &mut new_damper_map[i];
            for j in 0..new_beat as usize {
                if j >= self.max_beat {
                    if let Some(ref e) = self.chord_map[i][self.max_beat - 1] {
                        chd_m_map[j] = Some((e.0.clone(), e.1));
                    }
                    if let Some(ref e) = self.damper_map[i][self.max_beat - 1] {
                        ped_m_map[j] = Some((e.0.clone(), e.1));
                    }
                } else {
                    if let Some(ref evt) = self.chord_map[i][j].clone() {
                        chd_m_map[j] = Some((evt.0.clone(), evt.1));
                    }
                    if let Some(ref evt) = self.damper_map[i][j].clone() {
                        ped_m_map[j] = Some((evt.0.clone(), evt.1));
                    }
                }
            }
        }
        self.chord_map = new_chord_map;
        self.damper_map = new_damper_map;
        self.max_beat = new_beat as usize;
        self.whole_tick = whole_tick;
    }
    pub fn first_msr_num(&self) -> i32 {
        self.first_msr_num
    }
    pub fn set_first_msr_num(&mut self, first_msr_num: i32) {
        self.first_msr_num = first_msr_num;
    }
    pub fn whole_tick(&self) -> i32 {
        self.whole_tick
    }
    fn check_msr_beat(msr: isize, beat: isize) -> (isize, isize) {
        if !(0..=100).contains(&msr) || !(0..=20).contains(&beat) {
            // ありえない値
            panic!("Unexpected Number: msr={}, beat={}", msr, beat);
        }
        (msr, beat)
    }
    pub fn loop_msr_beat(&self, crnt_: &CrntMsrTick) -> (isize, isize) {
        let tick_for_onebeat = crnt_.tick_for_onemsr / (self.max_beat as i32);
        let beat = (crnt_.tick / tick_for_onebeat) as isize;
        let loop_size = self.whole_tick as isize / crnt_.tick_for_onemsr as isize;
        let mut msr = if crnt_.msr >= self.first_msr_num {
            (crnt_.msr - self.first_msr_num) as isize
        } else {
            0 // 次の小節が１小節目
        };
        while msr >= loop_size {
            msr -= loop_size;
        }
        Self::check_msr_beat(msr, beat)
    }
    pub fn gen_damper_ev_map(&self, crnt_: &CrntMsrTick, max_beat: usize) -> Vec<PedalPos> {
        let cmsr = if crnt_.msr >= self.first_msr_num {
            (crnt_.msr - self.first_msr_num) as usize
        } else {
            1 // 1小節目から開始
        };
        let mut damper_map = vec![PedalPos::NoEvt; max_beat];
        let mut damper_exists = false;
        for (i, ele) in damper_map.iter_mut().enumerate() {
            if let Some(c) = self
                .damper_map
                .get(cmsr)
                .and_then(|row| row.get(i))
                .and_then(|opt| opt.as_ref())
            {
                *ele = if c.1 && c.0.position == PedalPos::Full {
                    PedalPos::Full
                } else if c.1 && c.0.position == PedalPos::Half {
                    PedalPos::Half
                } else {
                    PedalPos::Off
                };
                damper_exists = true;
            }
        }
        if damper_exists {
            // もし、今回の小節に Damper イベントが存在したら、その情報を返す
            println!("damper exists in msr {}", cmsr);
            return damper_map;
        }
        // Damper イベントが無かったら、和音から Damper 情報を生成する
        if self.max_msr > cmsr {
            for (j, damper) in damper_map.iter_mut().enumerate() {
                if matches!(&self.chord_map[cmsr][j], Some(c) if c.1 && c.0.tbl != NO_PED_TBL_NUM) {
                    *damper = PedalPos::Full;
                }
            }
        }
        #[cfg(feature = "verbose")]
        println!("<gen_damper_ev_map for Damper> damper_map={:?}", damper_map);
        damper_map
    }
    pub fn gen_vari_num(&mut self, crnt_: &CrntMsrTick) -> i16 {
        let (msr, _beat) = self.loop_msr_beat(crnt_);
        self.vari_map[msr as usize].unwrap_or(0)
    }
    /// 指定されたタイミングの Chord の root/table を探す
    pub fn scan_chord(&self, crnt_msr: usize, crnt_beat: usize) -> (i16, i16) {
        let mut root: i16 = 0;
        let mut tbl: i16 = NO_PED_TBL_NUM;
        let mut find: bool = false;
        let mut msr = crnt_msr as isize;
        let mut beat = crnt_beat as isize;
        loop {
            if let Some(c) = &self.chord_map[msr as usize][beat as usize] {
                root = c.0.root;
                tbl = c.0.tbl;
                find = true;
            }
            if !find {
                beat -= 1;
                if beat < 0 {
                    msr -= 1;
                    if msr < 0 {
                        break;
                    }
                    beat = (self.max_beat - 1) as isize;
                }
            } else {
                break;
            }
        }
        //println!("$$$Chord Data -> root:{}, tbl:{}", root, tbl);
        (root, tbl)
    }
    /// 表示用に定期的に呼ばれ、root と tbl を更新する
    pub fn gen_chord_name(&mut self, crnt_: &CrntMsrTick) -> String {
        let (msr, beat) = self.loop_msr_beat(crnt_);
        let mut root = NO_ROOT;
        let mut tbl = NO_TABLE;
        self.chord_map[msr as usize][beat as usize]
            .iter()
            .for_each(|evt| {
                root = evt.0.root;
                tbl = evt.0.tbl;
            });

        if root != self.root_for_ui || tbl != self.tbl_for_ui {
            // 変化があった場合
            self.root_for_ui = root;
            self.tbl_for_ui = tbl;
            self.prepare_for_display();

            let num = self.vari_num;
            let num_str = if num == 0 {
                ""
            } else {
                &("@".to_string() + self.vari_num.to_string().as_str())
            };
            #[cfg(feature = "verbose")]
            println!(
                "###Chord Data: {}, {}, {}",
                self.chord_name, self.root_for_ui, self.tbl_for_ui
            );
            self.chord_name.clone() + num_str
        } else {
            // 変化がない場合は、前回の値を返す
            self.chord_name.clone()
        }
    }
    fn prepare_for_display(&mut self) {
        // generate chord name 'character' for display
        let tbl_num = self.tbl_for_ui;
        let tbl_name = get_table_name(tbl_num);
        let cname = tbl_name.to_string();
        if cname.chars().nth(0).unwrap_or(' ') == '_' {
            let root_index = ((self.root_for_ui - 1) / 3) as usize;
            let alteration = (self.root_for_ui + 1) % 3;
            let mut root = get_root_name(root_index).to_string();
            if alteration == 1 {
                root += "#";
            } else if alteration == 2 {
                root += "b";
            }
            self.chord_name = root.to_string() + &cname[1..];
        } else {
            self.chord_name = cname;
        }
    }
}

//*******************************************************************
//          Composition Loop Mediator Struct
//*******************************************************************
pub struct CmpsLoopMediator {
    pub state_reserve: bool,
    clear_cmps_ev: bool, // clear されたかどうか
    first_msr_num: i32,
    loop_id: u32, // loop sid
    cmps: Option<Box<CompositionMap>>,
    do_loop: bool,
    max_msr: i32,
    max_beat: i32,
    next_cmps: Option<Box<CompositionMap>>,
}
impl CmpsLoopMediator {
    pub fn new() -> Self {
        Self {
            state_reserve: false,
            clear_cmps_ev: false,
            first_msr_num: 1,
            loop_id: 0,
            cmps: None,
            do_loop: true,
            max_msr: 0,
            max_beat: 0,
            next_cmps: None,
        }
    }
    pub fn start(&mut self) {
        self.first_msr_num = 1; // 1小節目から開始
        self.state_reserve = true;
    }
    pub fn msrtop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, pbp: PartBasicPrm) {
        if self.state_reserve {
            // 前小節にて phrase/pattern 指定された時
            if crnt_.msr == 0 {
                // 今回 start したとき
                self.state_reserve = true;
            } else if crnt_.msr == 1 {
                // start 1小節後
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else if self.max_msr == 0 {
                // データのない状態で start し、今回初めて指定された時
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else if self.max_msr != 0 && (crnt_.msr - self.first_msr_num) % (self.max_msr) == 0 {
                // 前小節にて Loop Obj が終了した時
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else if self.max_msr != 0 && pbp.sync_flag {
                // sync コマンドによる強制リセット
                self.state_reserve = false;
                self.new_loop(crnt_, estk);
            } else {
                // 現在の Loop Obj が終了していない時
                // 現在の Phrase より新しい Phrase の whole_tick が大きい場合、
                // 新しい Phrase に更新する
                if self.next_whole_tick() >= self.whole_tick() {
                    self.state_reserve = false;
                    self.proc_forward_cmps_by_evt(estk);
                }
            }
        } else if self.max_msr != 0 && (crnt_.msr - self.first_msr_num) % self.max_msr == 0 {
            // Loop Obj が終了した時
            if self.do_loop {
                // 同じ Loop.Obj を再生する
                self.stay_loop(crnt_);
            } else {
                self.clear_cmp_prm();
            }
        }
    }
    /// Composition Event を受け取り、CompositionMap を生成する
    pub fn rcv_cmp(&mut self, msg: CmpData, tick_for_onemsr: i32, tick_for_onebeat: i32) {
        if msg.evts.is_empty() && msg.whole_tick == 0 {
            self.next_cmps = None;
            self.clear_cmps_ev = true;
        } else {
            let max_msr = (msg.whole_tick as i32 / tick_for_onemsr) as usize;
            let max_beat = (tick_for_onemsr / tick_for_onebeat) as usize;
            self.next_cmps = Some(Box::new(CompositionMap::new(
                msg.evts,
                msg.whole_tick as i32,
                max_msr,
                max_beat,
            )));
            // 新しい CompositionMap の first_msr_num を設定
            if let Some(ref mut nxt) = self.next_cmps {
                let fm = self.first_msr_num + msg.whole_tick as i32 / tick_for_onemsr;
                nxt.set_first_msr_num(fm);
            }
        }
        #[cfg(feature = "verbose")]
        println!("Received next_cmps >next_cmps is {:?}", self.next_cmps);
        self.do_loop = msg.do_loop;
        self.state_reserve = true;
    }
    fn clear_cmp_prm(&mut self) {
        self.first_msr_num = 0;
        self.max_msr = 0;
        self.max_beat = 0;
        self.cmps = None;
        self.next_cmps = None;
        self.do_loop = true;
    }
    /// 新たに Loop Obj.を生成
    fn new_loop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if let Some(ref nxcmps) = self.next_cmps {
            #[cfg(feature = "verbose")]
            println!("New Composition Loop! M:{:?},T:{:?}", crnt_.msr, crnt_.tick);
            self.first_msr_num = crnt_.msr; // 計測開始の更新
            let whole_tick = nxcmps.whole_tick();
            let (tick_for_onemsr, tick_for_beat) = estk.tg().get_beat_tick();
            (self.max_msr, self.max_beat) =
                Self::calc_msr_beat(whole_tick, tick_for_onemsr, tick_for_beat);

            if whole_tick == 0 {
                self.state_reserve = true; // 次小節冒頭で呼ばれるように
                self.cmps = None;
            } else {
                self.loop_id += 1;
                self.cmps = self.next_cmps.take();
                if let Some(ref mut cmp) = self.cmps {
                    cmp.set_first_msr_num(self.first_msr_num);
                }
            }
        } else if self.clear_cmps_ev {
            self.clear_cmp_prm();
            self.clear_cmps_ev = false;
        } else {
            // 拍子が変わってイベントが発生したとき
            // next_cmps が空なら、self.cmps をそのまま再生
            self.first_msr_num = crnt_.msr;
            let mut max_msr = 0;
            let mut max_beat = 0;
            if let Some(ref mut cmp) = self.cmps {
                cmp.set_first_msr_num(self.first_msr_num);
                max_msr = cmp.max_msr as i32;
                max_beat = cmp.max_beat as i32;
            }
            let (tick_for_onemsr, tick_for_beat) = estk.tg().get_beat_tick();
            let whole_tick = max_msr * tick_for_onemsr;
            let (_new_msr, new_beat) =
                Self::calc_msr_beat(whole_tick, tick_for_onemsr, tick_for_beat);
            if max_beat != new_beat {
                // 拍子が変わっていたら、Chord Map を更新する
                if let Some(ref mut cmp) = self.cmps {
                    cmp.reunfold(new_beat, whole_tick);
                }
            }
        }
    }
    fn stay_loop(&mut self, crnt_: &CrntMsrTick) {
        self.first_msr_num = crnt_.msr;
        if let Some(ref mut cmp) = self.cmps {
            cmp.set_first_msr_num(self.first_msr_num);
        }
    }
    /// 新しい Phrase を早送りして更新する
    fn proc_forward_cmps_by_evt(&mut self, estk: &mut ElapseStack) {
        // その時の beat 情報で、whole_tick を loop_measure に換算
        let whole_tick = if let Some(ref nxcmps) = self.next_cmps {
            nxcmps.whole_tick()
        } else {
            0
        };
        let (tick_for_onemsr, tick_for_beat) = estk.tg().get_beat_tick();
        (self.max_msr, self.max_beat) =
            Self::calc_msr_beat(whole_tick, tick_for_onemsr, tick_for_beat);

        // Composition の更新
        self.loop_id += 1;
        let mut fmsrnum = 0;
        if let Some(ref mut cmp) = self.cmps {
            fmsrnum = cmp.first_msr_num();
        }
        self.cmps = self.next_cmps.take();
        if let Some(ref mut cmp) = self.cmps {
            cmp.set_first_msr_num(fmsrnum);
        }
        #[cfg(feature = "verbose")]
        println!("Replace Composition Loop! --whole tick: {whole_tick}");
    }
    /// whole_tick を loop_measure と beat に換算する
    fn calc_msr_beat(whole_tick: i32, tick_for_onemsr: i32, tick_for_beat: i32) -> (i32, i32) {
        // その時の beat 情報で、whole_tick を loop_measure に換算
        let plus_one = if whole_tick % tick_for_onemsr == 0 {
            0
        } else {
            1
        };
        (
            whole_tick / tick_for_onemsr + plus_one,
            tick_for_onemsr / tick_for_beat,
        )
    }
    fn next_whole_tick(&self) -> i32 {
        if let Some(ref nxcmps) = self.next_cmps {
            nxcmps.whole_tick()
        } else {
            0
        }
    }
    fn whole_tick(&self) -> i32 {
        if let Some(ref cmp) = self.cmps {
            cmp.whole_tick()
        } else {
            0
        }
    }

    /// 一度 Mediator（仲介者）を通してから、CompositionMap のサービスを利用する
    /// Not Yet:
    /// いずれ、未来の小節の情報を取得できるようにする
    /// cmps（現在）か next_cmps（未来）のどちらかを選択する
    pub fn get_damper_ev_map(&self, crnt_: &CrntMsrTick, max_beat: usize) -> Vec<PedalPos> {
        if let Some(ref cmp) = self.cmps {
            cmp.gen_damper_ev_map(crnt_, max_beat)
        } else {
            vec![PedalPos::NoEvt; max_beat]
        }
    }
    pub fn get_chord(&self, designated_: &CrntMsrTick) -> (i16, i16) {
        if designated_.msr >= self.first_msr_num + self.max_msr {
            // 指定された小節が、ループサイズを超えている場合
            if let Some(ref cmp) = self.next_cmps {
                let (msr, beat) = cmp.loop_msr_beat(designated_);
                return cmp.scan_chord(msr as usize, beat as usize);
            }
        }
        // 現在と同じ Composition Loop から、情報を取得する
        if let Some(ref cmp) = self.cmps {
            let (msr, beat) = cmp.loop_msr_beat(designated_);
            cmp.scan_chord(msr as usize, beat as usize)
        } else {
            (NO_ROOT, NO_TABLE)
        }
    }
    pub fn get_chord_name(&mut self, crnt_: &CrntMsrTick) -> String {
        if let Some(ref mut cmp) = self.cmps {
            cmp.gen_chord_name(crnt_)
        } else {
            "-".to_string()
        }
    }
    pub fn get_vari_num(&mut self, crnt_: &CrntMsrTick) -> i16 {
        if let Some(ref mut cmp) = self.cmps {
            cmp.gen_vari_num(crnt_)
        } else {
            0
        }
    }
}
