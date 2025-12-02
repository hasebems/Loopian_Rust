//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::elapse_broken_ptn::BrokenPattern;
use super::elapse_cluster_ptn::ClusterPattern;
use super::elapse_note::*;
use super::floating_tick::FloatingTick;
use super::note_translation::*;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::{txt2seq_cmps, txt2seq_cmps::*};
use crate::lpnlib::*;

//*******************************************************************
//          Phrase Loop Parameter
//*******************************************************************
pub struct PhraseLoopParam {
    keynote: u8,
    msr: i32,
    phr: Vec<PhrEvt>,
    ana: Vec<AnaEvt>,
    whole_tick: i32,
    turnnote: i16,
}
impl PhraseLoopParam {
    pub fn new(
        keynote: u8,
        msr: i32,
        phr: Vec<PhrEvt>,
        ana: Vec<AnaEvt>,
        whole_tick: i32,
        turnnote: i16,
    ) -> Self {
        Self {
            keynote,
            msr,
            phr,
            ana,
            whole_tick,
            turnnote,
        }
    }
}
//*******************************************************************
//          Phrase Loop Struct
//*******************************************************************
pub struct PhraseLoop {
    id: ElapseId,
    priority: u32,

    phrase: Vec<PhrEvt>,
    analys: Vec<AnaEvt>,
    keynote: u8,
    play_counter: usize,
    next_tick_in_phrase: i32,
    last_note: i16,
    noped: bool,
    turnnote: i16,
    para_root_base: i16,
    same_time_stuck: Vec<(u8, String)>,
    staccato_rate: i32,
    flt: FloatingTick, //  FloatingTick を保持する

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
//*******************************************************************
//          Phrase Loop Implementation
//*******************************************************************
impl PhraseLoop {
    const MAX_FRONT_DISPERSE: i32 = 120; // Tick の前への最大散らし幅
    const EACH_DISPERSE: i32 = 60; // Tick の散らし幅の単位

    pub fn new(sid: u32, pid: u32, prm: PhraseLoopParam) -> Rc<RefCell<Self>> {
        let noped = prm.ana.clone().iter().any(|x| {
            if let AnaEvt::Exp(e) = x {
                e.atype == ExpType::Noped
            } else {
                false
            }
        });
        let mut para_root_base = 0;
        prm.ana.iter().for_each(|x| match x {
            AnaEvt::Exp(e) if e.atype == ExpType::ParaRoot => {
                para_root_base = e.note;
            }
            _ => (),
        });
        let mut staccato_rate = 100;
        prm.ana.iter().for_each(|x| match x {
            AnaEvt::Exp(e) if e.atype == ExpType::Artic => {
                staccato_rate = e.cnt as i32;
            }
            _ => (),
        });
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpPhraseLoop,
            },
            priority: PRI_PHR_LOOP,
            phrase: prm.phr,
            analys: prm.ana,
            keynote: prm.keynote,
            play_counter: 0,
            next_tick_in_phrase: 0,
            last_note: NO_NOTE as i16,
            noped,
            turnnote: prm.turnnote,
            para_root_base,
            same_time_stuck: Vec::new(),
            staccato_rate,
            flt: FloatingTick::new(false),
            // for super's member
            whole_tick: prm.whole_tick,
            destroy: false,
            first_msr_num: prm.msr,
            next_msr: 0,
            next_tick: 0,
        }))
    }
    pub fn get_noped(&self) -> bool {
        self.noped
    }
    pub fn set_keynote(&mut self, knt: u8) {
        // Phrase Loop の keynote を変更する
        self.keynote = knt;
    }
    fn generate_event(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        elapsed_tick: i32,
    ) -> i32 {
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let phr = self.phrase.to_vec();
        let max_ev = self.phrase.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA; // means sequence finished
                break;
            }
            next_tick = phr[trace].tick() as i32;
            if next_tick <= elapsed_tick {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                let evtx = phr[trace].clone();
                match evtx {
                    PhrEvt::Note(ev) => {
                        let rt_tbl = self.get_root_tbl(estk, crnt_);
                        let (trans_note, deb_txt) = self.translate_note(rt_tbl, ev.note, next_tick);
                        self.note_event(estk, trace * 10, ev, trans_note, deb_txt, (msr, tick));
                    }
                    PhrEvt::NoteList(ev) => {
                        let rt_tbl = self.get_root_tbl(estk, crnt_);
                        self.note_on_at_the_same_time(crnt_, estk, &ev, rt_tbl, next_tick);
                    }
                    PhrEvt::BrkPtn(ev) => {
                        self.set_broken(crnt_, ev, estk, (msr, tick));
                    }
                    PhrEvt::ClsPtn(ev) => {
                        self.set_cluster(crnt_, ev, estk, (msr, tick));
                    }
                    _ => (),
                }
            } else {
                break;
            }
            trace += 1;
        }
        self.play_counter = trace;
        self.check_next_evt_and_set_floating(trace);
        next_tick
    }
    fn note_on_at_the_same_time(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        ev: &NoteListEvt,
        rt_tbl: (i16, i16),
        next_tick: i32,
    ) {
        let mut same_time_stuck = Vec::new();
        for note in ev.notes.iter() {
            let (trans_note, deb_txt) = self.translate_note(rt_tbl, *note, next_tick);
            if !same_time_stuck
                .iter()
                .any(|x: &(u8, String)| x.0 == trans_note)
            {
                same_time_stuck.push((trans_note, deb_txt.to_string()));
            }
        }
        same_time_stuck.sort_by_key(|x| x.0); // 同タイミングの音をソート
        let (ntmsr, nttick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
        //println!("|^^ Note on at the same time: {}, {}, {}", same_time_stuck.len(), ntmsr, nttick);
        for (i, nt) in same_time_stuck.iter().enumerate() {
            let arp = if ev.floating {
                (i as i32 * Self::EACH_DISPERSE) - Self::MAX_FRONT_DISPERSE
            } else {
                0
            };
            let mut msr = ntmsr;
            let mut tick = nttick + arp;
            if tick < 0 {
                tick += crnt_.tick_for_onemsr;
                msr -= 1;
            } else if tick >= crnt_.tick_for_onemsr {
                tick -= crnt_.tick_for_onemsr;
                msr += 1;
            }
            let nev_clone = NoteEvt::from_note_list(ev, nt.0);
            let trace = self.play_counter * 10 + i;
            //println!("|__ Note on at the same time: {}, {}, {}", nt.0, msr, tick);
            self.note_event(estk, trace, nev_clone, nt.0, nt.1.clone(), (msr, tick));
        }
        // 次のイベント処理の準備
        self.flt.turnoff_floating();
    }
    fn get_root_tbl(&mut self, estk: &mut ElapseStack, crnt_: &CrntMsrTick) -> (i16, i16) {
        // ルートとテーブルを取得する
        let (mut root, mut ctbl) = (NO_ROOT, NO_TABLE);
        if let Some(pt) = estk.part(self.id.pid) {
            let mut pt_borrowed = pt.borrow_mut();
            let cmp_med = pt_borrowed.get_cmps_med();
            (root, ctbl) = cmp_med.get_chord(crnt_);
        }
        (root, ctbl)
    }
    fn note_event(
        &mut self,
        estk: &mut ElapseStack,
        trace: usize,
        ev: NoteEvt, // ev: ['note', tick, duration, note, velocity]
        trans_note: u8,
        deb_txt: String,
        tk: (i32, i32), // (next_tick, msr, tick)
    ) {
        let mut crnt_ev = ev;
        crnt_ev.note = trans_note;

        //  Calculate Duration
        if crnt_ev.artic != DEFAULT_ARTIC {
            let calc = (crnt_ev.dur as i32) * (crnt_ev.artic as i32);
            crnt_ev.dur = (calc / DEFAULT_ARTIC as i32) as i16;
        } else if (self.staccato_rate as i16) != DEFAULT_ARTIC {
            let calc = (crnt_ev.dur as i32) * self.staccato_rate;
            crnt_ev.dur = (calc / DEFAULT_ARTIC as i32) as i16;
        }

        //  Generate Note Struct
        let nt: Rc<RefCell<dyn Elapse>> = Note::new(
            trace as u32, //  read pointer
            self.id.sid,  //  loop.sid -> note.pid
            NoteParam::new(
                estk,
                &crnt_ev,
                deb_txt + &format!(" / Pt:{} Lp:{}", &self.id.pid, &self.id.sid),
                (
                    self.keynote,
                    tk.0,
                    tk.1,
                    self.id.pid,
                    crnt_ev.floating,
                    false,
                ),
            ),
        );
        estk.add_elapse(Rc::clone(&nt));
    }
    fn translate_note(&mut self, rt_tbl: (i16, i16), ev_note: u8, next_tick: i32) -> (u8, String) {
        if rt_tbl.0 == NO_ROOT && rt_tbl.1 == NO_TABLE {
            return (ev_note, "no chord".to_string());
        }
        let ctbl = rt_tbl.1;
        let deb_txt: String;
        let trans_note: u8;
        let root: i16 = get_note_from_root(rt_tbl.0);
        let (movable_scale, mut para_note) = txt2seq_cmps::is_movable_scale(ctbl, root);
        if movable_scale {
            if para_note > self.turnnote {
                para_note -= 12;
            }
            trans_note = translate_note_parascl(para_note, ctbl, ev_note);
            deb_txt = "para_sc:".to_string();
        } else {
            let option = self.specify_trans_option(next_tick, ev_note);
            match option {
                TrnsType::Para => {
                    let para_root = root - self.para_root_base;
                    let mut tgt_nt = ev_note as i16 + para_root;
                    if root > self.turnnote {
                        tgt_nt -= 12;
                    }
                    trans_note = translate_note_com(root, ctbl, tgt_nt as u8);
                    deb_txt = "para:".to_string();
                }
                TrnsType::Com => {
                    trans_note = translate_note_com(root, ctbl, ev_note);
                    deb_txt = "com:".to_string();
                }
                TrnsType::NoTrns => {
                    trans_note = ev_note;
                    deb_txt = "none:".to_string();
                }
                TrnsType::Arp(nt_diff) => {
                    trans_note = translate_note_arp2(root, ctbl, ev_note, nt_diff, self.last_note);
                    deb_txt = "arp:".to_string();
                }
            }
        }
        self.last_note = trans_note as i16;
        //crnt_ev[NOTE] = trans_note;
        (
            trans_note.clamp(0, 127),
            deb_txt + &(root.to_string() + "-" + &ctbl.to_string()),
        )
    }
    fn specify_trans_option(&self, next_tick: i32, note: u8) -> TrnsType {
        for anaone in self.analys.iter() {
            if let AnaEvt::Beat(b) = anaone
                && b.tick == next_tick as i16
                && b.note == note as i16
            {
                return b.trns;
            }
        }
        TrnsType::Com
    }
    fn set_broken(
        &mut self,
        crnt_: &CrntMsrTick,
        mut ev: BrkPatternEvt,
        estk: &mut ElapseStack,
        tk: (i32, i32),
    ) {
        while ev.tick >= crnt_.tick_for_onemsr as i16 {
            // pattern は１小節内で完結
            ev.tick -= crnt_.tick_for_onemsr as i16;
        }
        let ptn: Rc<RefCell<dyn Elapse>> = BrokenPattern::new(
            crnt_.msr as u32, //  read pointer
            self.id.sid,      //  loop.sid -> note.pid
            self.id.pid,      //  part
            self.keynote,
            tk.0,
            ev,
            self.analys.to_vec(),
        );
        estk.add_elapse(Rc::clone(&ptn));
    }
    fn set_cluster(
        &mut self,
        crnt_: &CrntMsrTick,
        mut ev: ClsPatternEvt,
        estk: &mut ElapseStack,
        tk: (i32, i32),
    ) {
        while ev.tick >= crnt_.tick_for_onemsr as i16 {
            // pattern は１小節内で完結
            ev.tick -= crnt_.tick_for_onemsr as i16;
        }
        let ptn: Rc<RefCell<dyn Elapse>> = ClusterPattern::new(
            crnt_.msr as u32, //  read pointer
            self.id.sid,      //  loop.sid -> note.pid
            self.id.pid,      //  part
            self.keynote,
            (
                tk.0,
                self.flt.just_crnt().msr,
                self.flt.just_crnt().tick,
                self.flt.just_crnt().tick_for_onemsr,
            ),
            ev,
            self.analys.to_vec(),
        );
        estk.add_elapse(Rc::clone(&ptn));
    }
    fn check_next_evt_and_set_floating(&mut self, next_idx: usize) {
        self.flt.turnoff_floating();
        if next_idx >= self.phrase.len() {
            return; //  次のイベントがない場合は、何もしない
        }
        let next_evt = self.phrase[next_idx].clone();
        match next_evt {
            PhrEvt::NoteList(ev) => {
                if ev.floating {
                    self.flt.turnon_floating();
                }
            }
            PhrEvt::ClsPtn(ev) => {
                if ev.arpeggio > 0 {
                    self.flt.turnon_floating();
                }
            }
            _ => (),
        }
    }
}
//*******************************************************************
//          Phrase Loop Trait Implementation
//          Elapse Trait
//*******************************************************************
impl Elapse for PhraseLoop {
    /// id を得る
    fn id(&self) -> ElapseId {
        self.id
    }
    /// priority を得る
    fn prio(&self) -> u32 {
        self.priority
    }
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32, bool) {
        (self.next_msr, self.next_tick, false)
    }
    fn start(&mut self, _msr: i32) {} // User による start/play 時にコールされる
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {
        self.set_destroy();
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.phrase = Vec::new();
        self.analys = Vec::new();
        self.play_counter = 0;
        self.next_tick_in_phrase = 0;
        self.last_note = NO_NOTE as i16;
        self.same_time_stuck = Vec::new();
        self.destroy = false;
        self.next_msr = 0;
        self.next_tick = 0;
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {
        self.destroy()
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.destroy {
            return;
        }

        // crnt_ を記譜上の小節数に変換し、その後 elapsed_tick を計算する
        let ntcrnt_ = self.flt.convert_to_notational(crnt_);

        let elapsed_tick = self.calc_serial_tick(&ntcrnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = FULL;
            self.destroy = true;
        } else if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_event(&ntcrnt_, estk, elapsed_tick);
            self.next_tick_in_phrase = next_tick;
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            } else {
                let (msr, tick) = self.gen_msr_tick(&ntcrnt_, self.next_tick_in_phrase);
                let mt = CrntMsrTick {
                    msr,
                    tick,
                    tick_for_onemsr: ntcrnt_.tick_for_onemsr,
                    ..Default::default()
                };
                // FloatingTick を使って、次に呼ばれる実際の小節とTickを計算する
                if let Some(rlcrnt_) = self.flt.convert_to_real(&mt) {
                    // FloatingTick の変換結果を使って、次の小節とTickを設定する
                    //println!(
                    //    "|__ PhraseLoop: next_msr/tick: {}/{}, crnt_msr/tick: {}/{}, ntcrnt_msr/tick:{}/{}",
                    //    rlcrnt_.msr, rlcrnt_.tick, crnt_.msr, crnt_.tick, ntcrnt_.msr, ntcrnt_.tick
                    //);
                    self.next_msr = rlcrnt_.msr;
                    self.next_tick = rlcrnt_.tick;
                } else {
                    // FloatingTick の変換結果が None の場合は、現在の crnt_ をそのまま使用
                    self.next_msr = mt.msr;
                    self.next_tick = mt.tick;
                }
            }
        }
    }
}
//*******************************************************************
//          Phrase Loop Trait Implementation
//          Loop Trait
//*******************************************************************
impl Loop for PhraseLoop {
    fn destroy(&self) -> bool {
        self.destroy
    }
    fn set_destroy(&mut self) {
        self.next_tick = 0;
        self.next_msr = FULL;
        self.destroy = true;
    }
    fn first_msr_num(&self) -> i32 {
        self.first_msr_num
    }
    /// Loopの途中から再生するための小節数を設定
    fn set_forward(&mut self, crnt_: &CrntMsrTick, elapsed_msr: i32) {
        let elapsed_tick = elapsed_msr * crnt_.tick_for_onemsr;
        let mut next_tick: i32;
        let mut trace: usize = self.play_counter;
        let phr = self.phrase.to_vec();
        let max_ev = self.phrase.len();
        loop {
            if max_ev <= trace {
                next_tick = END_OF_DATA; // means sequence finished
                break;
            }
            next_tick = phr[trace].tick() as i32;
            if next_tick >= elapsed_tick {
                break;
            }
            trace += 1;
        }
        self.play_counter = trace;
        self.next_tick_in_phrase = next_tick;
        let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
        self.next_msr = msr;
        self.next_tick = tick;
        #[cfg(feature = "verbose")]
        println!("### Forwarded to: {}, {}", self.next_msr, self.next_tick);
    }
}
