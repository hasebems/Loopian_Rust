//  Created by Hasebe Masahiko on 2025/05/03
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::elapse_part::PartBasicPrm;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::txt2seq_cmps::*;
use crate::lpnlib::*;

//*******************************************************************
//          Composition Loop Struct
//*******************************************************************
#[derive(Clone, Debug)]
pub struct UnfoldedComposition {
    cmps_map: Vec<Vec<Vec<(ChordEvt, bool)>>>, // [msr][beat][num]

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
impl UnfoldedComposition {
    pub fn new(evts: Vec<ChordEvt>, whole_tick: i32, max_msr: usize, max_beat: usize) -> Self {
        let cmps_map = Self::unfold_cmp_evt(evts, max_msr, max_beat, whole_tick);
        Self {
            cmps_map,
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
    /// Composition Event を受け取り、Composition Map に展開する
    fn unfold_cmp_evt(
        evts: Vec<ChordEvt>,
        msr: usize,
        beat: usize,
        whole_tick: i32,
    ) -> Vec<Vec<Vec<(ChordEvt, bool)>>> {
        let mut cmps_map = vec![vec![Vec::new(); beat]; msr];
        let tick_for_onemsr = whole_tick as usize / msr;
        let tick_for_onebeat = tick_for_onemsr / beat;
        let mut crnt_idx = 0;
        let mut last_evt = None;
        let max_len = evts.len();
        // 1小節の中に、1拍ごとに分けて、イベントを展開する
        for (i, msr_map) in cmps_map.iter_mut().enumerate() {
            for (j, beat_map) in msr_map.iter_mut().enumerate() {
                let crnt_tick = (i * tick_for_onemsr + j * tick_for_onebeat) as i16;
                if crnt_idx < max_len && evts[crnt_idx].tick <= crnt_tick {
                    // 同タイミングに複数イベントがある場合のため、loopにて処理
                    loop {
                        beat_map.push((evts[crnt_idx].clone(), true));
                        if evts[crnt_idx].mtype == TYPE_CHORD {
                            last_evt = Some(evts[crnt_idx].clone());
                        }
                        crnt_idx += 1;
                        if crnt_idx < max_len && evts[crnt_idx].tick <= crnt_tick {
                            continue;
                        } else {
                            break;
                        }
                    }
                } else if let Some(ref evt) = last_evt {
                    beat_map.push((evt.clone(), false));
                }
            }
        }
        #[cfg(feature = "verbose")]
        {
            println!("cmps_map={:?}", cmps_map);
            println!("Unfolded Composition!");
        }
        cmps_map
    }
    pub fn reunfold(&mut self, new_beat: i32, whole_tick: i32) {
        // beat の変更に伴い、cmps_map を再構築する
        let mut new_cmps_map = vec![vec![Vec::new(); new_beat as usize]; self.max_msr];
        for (i, msr_map) in new_cmps_map.iter_mut().enumerate() {
            for (j, beat_map) in msr_map.iter_mut().enumerate() {
                if j >= self.max_beat {
                    for evt in &self.cmps_map[i][self.max_beat - 1] {
                        beat_map.push((evt.0.clone(), evt.1));
                    }
                } else {
                    for evt in &self.cmps_map[i][j] {
                        beat_map.push((evt.0.clone(), evt.1));
                    }
                }
            }
        }
        self.cmps_map = new_cmps_map;
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
            1   // 1小節目から開始
        };
        while msr >= loop_size {
            msr -= loop_size;
        }
        Self::check_msr_beat(msr, beat)
    }
    pub fn gen_chord_map(&self, crnt_: &CrntMsrTick, max_beat: usize) -> Vec<bool> {
        let cmsr = if crnt_.msr >= self.first_msr_num {
            (crnt_.msr - self.first_msr_num) as usize
        } else {
            1   // 1小節目から開始
        };
        let mut chord_map = vec![false; max_beat];
        if self.max_msr > cmsr {
            for (j, chord) in chord_map.iter_mut().enumerate() {
                for evt in &self.cmps_map[cmsr][j] {
                    if evt.1 && evt.0.mtype == TYPE_CHORD && evt.0.tbl != NO_PED_TBL_NUM {
                        *chord = true;
                    }
                }
            }
        }
        #[cfg(feature = "verbose")]
        println!("<gen_chord_map for Damper> chord_map={:?}", chord_map);
        chord_map
    }
    pub fn gen_vari_num(&mut self, crnt_: &CrntMsrTick) -> i16 {
        let (msr, beat) = self.loop_msr_beat(crnt_);
        let mut vari_num = 0;
        self.cmps_map[msr as usize][beat as usize]
            .iter()
            .for_each(|evt| {
                if evt.0.mtype == TYPE_VARI {
                    vari_num = evt.0.root;
                }
            });
        vari_num
    }
    /// 現在のタイミングに合った Chord の root/table を探す
    pub fn scan_chord(&self, crnt_msr: usize, crnt_beat: usize) -> (i16, i16) {
        let mut root: i16 = 0;
        let mut tbl: i16 = NO_PED_TBL_NUM;
        let mut find: bool = false;
        let mut msr = crnt_msr as isize;
        let mut beat = crnt_beat as isize;
        loop {
            if !self.cmps_map[msr as usize][beat as usize].is_empty() {
                for evt in &self.cmps_map[msr as usize][beat as usize] {
                    if evt.0.mtype == TYPE_CHORD {
                        root = evt.0.root;
                        tbl = evt.0.tbl;
                        find = true;
                        break;
                    }
                }
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
        self.cmps_map[msr as usize][beat as usize]
            .iter()
            .for_each(|evt| {
                if evt.0.mtype == TYPE_CHORD {
                    root = evt.0.root;
                    tbl = evt.0.tbl;
                }
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
    first_msr_num: i32,
    loop_id: u32, // loop sid
    cmps: Option<Box<UnfoldedComposition>>,
    do_loop: bool,
    max_msr: i32,
    max_beat: i32,
    next_cmps: Option<Box<UnfoldedComposition>>,
}
impl CmpsLoopMediator {
    pub fn new() -> Self {
        Self {
            state_reserve: false,
            first_msr_num: 0,
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
    /// Composition Event を受け取り、UnfoldedComposition を生成する
    pub fn rcv_cmp(&mut self, msg: ChordData, tick_for_onemsr: i32, tick_for_onebeat: i32) {
        if msg.evts.is_empty() && msg.whole_tick == 0 {
            self.next_cmps = None;
        } else {
            let max_msr = (msg.whole_tick as i32 / tick_for_onemsr) as usize;
            let max_beat = (tick_for_onemsr / tick_for_onebeat) as usize;
            self.next_cmps = Some(Box::new(UnfoldedComposition::new(
                msg.evts,
                msg.whole_tick as i32,
                max_msr,
                max_beat,
            )));
        }
        #[cfg(feature = "verbose")]
        println!(
            "Received next_cmps >next_cmps is {:?}",
            self.next_cmps.is_some()
        );
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
        } else {
            // 新しい Composition が空のとき、self.cmps をそのまま再生
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
        println!("Replace Composition Loop! --whole tick: {}", whole_tick);
    }
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

    /// 一度 Mediator（仲介者）を通してから、UnfoldedComposition のサービスを利用する
    /// Not Yet:
    /// いずれ、未来の小節の情報を取得できるようにする
    /// cmps（現在）か next_cmps（未来）のどちらかを選択する
    pub fn get_chord_map(&self, crnt_: &CrntMsrTick, max_beat: usize) -> Vec<bool> {
        if let Some(ref cmp) = self.cmps {
            cmp.gen_chord_map(crnt_, max_beat)
        } else {
            vec![false; max_beat]
        }
    }
    pub fn get_chord(&self, crnt_: &CrntMsrTick, _future: &CrntMsrTick) -> (i16, i16) {
        if let Some(ref cmp) = self.cmps {
            let (msr, beat) = cmp.loop_msr_beat(crnt_);
            cmp.scan_chord(msr as usize, beat as usize)
        } else {
            (NO_ROOT, NO_PED_TBL_NUM)
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
