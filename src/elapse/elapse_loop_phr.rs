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
use crate::cmd::txt2seq_cmps;
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
    same_time_shared_note: NoteEvt,
    same_time_index: usize,
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
    pub fn new(sid: u32, pid: u32, prm: PhraseLoopParam) -> Rc<RefCell<Self>> {
        let noped = prm.ana.clone().iter().any(|x| {
            if let AnaEvt::Exp(e) = x {
                e.atype == ExpType::Noped
            } else {
                false
            }
        });
        let mut para_root_base = 0;
        prm.ana.iter().for_each(|x| {
            if let AnaEvt::Exp(e) = x {
                if e.atype == ExpType::ParaRoot {
                    para_root_base = e.note;
                }
            }
        });
        let mut staccato_rate = 100;
        prm.ana.iter().for_each(|x| {
            if let AnaEvt::Exp(e) = x {
                if e.atype == ExpType::Artic {
                    staccato_rate = e.cnt as i32;
                }
            }
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
            same_time_shared_note: NoteEvt::default(),
            same_time_index: 0,
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
        // 同時発音の処理
        if !self.same_time_stuck.is_empty() {
            if self.same_time_note_on(crnt_, estk) {
                return self.phrase[self.play_counter].tick() as i32;
            } else {
                self.flt.turnoff_floating();
                self.play_counter += 1;
            }
        }

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
                self.flt.turnoff_floating();
                match evtx {
                    PhrEvt::Note(ev) => {
                        let rt_tbl = self.get_root_tbl(estk, crnt_);
                        let (trans_note, deb_txt) = self.translate_note(rt_tbl, ev.note, next_tick);
                        self.note_event(estk, trace * 10, ev, trans_note, deb_txt, (msr, tick));
                    }
                    PhrEvt::NoteList(ev) => {
                        if ev.floating {
                            self.flt.turnon_floating();
                        }
                        let rt_tbl = self.get_root_tbl(estk, crnt_);
                        self.same_time_stuck = Vec::new();
                        for note in ev.notes.iter() {
                            let (trans_note, deb_txt) =
                                self.translate_note(rt_tbl, *note, next_tick);
                            self.stuck_note(trans_note, &deb_txt);
                        }
                        self.same_time_stuck.sort_by_key(|x| x.0); // 同タイミングの音をソート
                        self.same_time_index = 0;
                        self.same_time_shared_note = NoteEvt::from_note_list(&ev, 0);
                        self.flt
                            .set_disperse_count(self.same_time_stuck.len() as i32, 0);
                        break;
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
        next_tick
    }
    fn same_time_note_on(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> bool {
        // 同タイミングの音が鳴る場合は、同タイミングの音を鳴らす
        let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
        if let Some(nt) = self.same_time_stuck.get(self.same_time_index) {
            let mut nev_clone = self.same_time_shared_note.clone();
            nev_clone.note = nt.0;
            let trace = self.play_counter * 10 + self.same_time_index;
            self.note_event(estk, trace, nev_clone, nt.0, nt.1.clone(), (msr, tick));
        }

        self.same_time_index += 1;
        if self.same_time_index >= self.same_time_stuck.len() {
            // 同タイミングの音を全て鳴らしたら、同タイミングの音をクリアする
            self.same_time_stuck.clear();
            false
        } else {
            true
        }
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
    fn stuck_note(&mut self, trans_note: u8, deb_txt: &str) {
        //  同タイミング重複音を鳴らさない
        if !self.same_time_stuck.iter().any(|x| x.0 == trans_note) {
            self.same_time_stuck.push((trans_note, deb_txt.to_string()));
        }
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
                self.keynote,
                deb_txt + &format!(" / Pt:{} Lp:{}", &self.id.pid, &self.id.sid),
                tk.0,
                tk.1,
                self.id.pid,
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
        let root: i16 = ROOT2NTNUM[rt_tbl.0 as usize];
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
            if let AnaEvt::Beat(b) = anaone {
                if b.tick == next_tick as i16 && b.note == note as i16 {
                    return b.trns;
                }
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
        if ev.arpeggio > 0 {
            // アルペジオの時は FloatingTick を有効にする
            self.flt.turnon_floating();
        } else {
            // アルペジオでない時は FloatingTick を無効にする
            self.flt.turnoff_floating();
        }
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
    fn next(&self) -> (i32, i32) {
        (self.next_msr, self.next_tick)
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
                };
                // FloatingTick を使って、次に呼ばれる実際の小節とTickを計算する
                let rlcrnt_ = self.flt.convert_to_real(&mt);
                //println!(
                //    "|__ PhraseLoop: next_msr/tick: {}/{}, crnt_msr/tick: {}/{}, ntcrnt_msr/tick:{}/{}, ntp:{}",
                //    rlcrnt_.msr, rlcrnt_.tick, crnt_.msr, crnt_.tick, ntcrnt_.msr, ntcrnt_.tick, self.next_tick_in_phrase
                //);
                self.next_msr = rlcrnt_.msr;
                self.next_tick = rlcrnt_.tick;
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
