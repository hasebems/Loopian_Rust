//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::elapse_note::*;
use super::elapse_pattern::DynamicPattern;
use super::note_translation::*;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::txt2seq_cmps;
use crate::lpnlib::*;

//*******************************************************************
//          Phrase Loop Struct
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
    same_note_stuck: Vec<i16>,
    same_note_msr: i32,
    same_note_tick: i32,
    staccato_rate: i32,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl PhraseLoop {
    pub fn new(sid: u32, pid: u32, prm: PhraseLoopParam) -> Rc<RefCell<Self>> {
        let noped = prm
            .ana
            .clone()
            .iter()
            .any(|x| x.mtype == TYPE_EXP && x.atype == NOPED);
        let mut para_root_base = 0;
        prm.ana.iter().for_each(|x| {
            if x.mtype == TYPE_EXP && x.atype == PARA_ROOT {
                para_root_base = x.note;
            }
        });
        let mut staccato_rate = 100;
        prm.ana.iter().for_each(|x| {
            if x.mtype == TYPE_EXP && x.atype == ARTIC {
                staccato_rate = x.cnt as i32;
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
            same_note_stuck: Vec::new(),
            same_note_msr: 0,
            same_note_tick: 0,
            staccato_rate,
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
            next_tick = phr[trace].tick as i32;
            if next_tick <= elapsed_tick {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                let tp = self.phrase[trace].mtype;
                if tp == TYPE_NOTE {
                    if self.same_note_msr != msr || self.same_note_tick != tick {
                        // 設定されているタイミングが少しでも違えば、同タイミング重複音検出をクリア
                        self.same_note_stuck = Vec::new();
                        self.same_note_msr = msr;
                        self.same_note_tick = tick;
                    }
                    self.note_event(estk, trace, phr[trace].clone(), next_tick, msr, tick);
                } else if tp == TYPE_CLS || tp == TYPE_ARP {
                    let mut ptn = self.phrase[trace].clone();
                    while ptn.tick >= crnt_.tick_for_onemsr as i16 {
                        // pattern は１小節内で完結
                        ptn.tick -= crnt_.tick_for_onemsr as i16;
                    }
                    let ptn: Rc<RefCell<dyn Elapse>> = DynamicPattern::new(
                        crnt_.msr as u32, //  read pointer
                        self.id.sid,      //  loop.sid -> note.pid
                        self.id.pid,      //  part
                        self.keynote,
                        msr,
                        ptn,
                        self.analys.to_vec(),
                    );
                    estk.add_elapse(Rc::clone(&ptn));
                }
            } else {
                break;
            }
            trace += 1;
        }

        self.play_counter = trace;
        next_tick
    }
    pub fn set_forward(&mut self, crnt_: &CrntMsrTick, elapsed_msr: i32) {
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
            next_tick = phr[trace].tick as i32;
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
    fn note_event(
        &mut self,
        estk: &mut ElapseStack,
        trace: usize,
        ev: PhrEvt,
        next_tick: i32,
        msr: i32,
        tick: i32,
    ) {
        // ev: ['note', tick, duration, note, velocity]
        let mut crnt_ev = ev.clone();
        let mut deb_txt: String = "no chord".to_string();
        let (mut rt, mut ctbl) = (NO_ROOT, NO_TABLE);
        if let Some(cmps) = estk.get_cmps(self.id.pid as usize) {
            (rt, ctbl) = cmps.borrow().get_chord();
        }

        //  Note Translation
        if rt != NO_ROOT || ctbl != NO_TABLE {
            (crnt_ev.note, deb_txt) = self.translate_note(rt, ctbl, ev, next_tick);
        }

        //  同タイミング重複音を鳴らさない
        if self.same_note_stuck.iter().any(|x| *x == crnt_ev.note) {
            return;
        } else {
            self.same_note_stuck.push(crnt_ev.note);
        }

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
                msr,
                tick,
                self.id.pid,
            ),
        );
        estk.add_elapse(Rc::clone(&nt));
    }
    fn translate_note(&mut self, rt: i16, ctbl: i16, ev: PhrEvt, next_tick: i32) -> (i16, String) {
        let deb_txt: String;
        let trans_note: i16;
        let root: i16 = ROOT2NTNUM[rt as usize];
        let (movable_scale, mut para_note) = txt2seq_cmps::is_movable_scale(ctbl, root);
        if movable_scale {
            if para_note > self.turnnote {
                para_note -= 12;
            }
            trans_note = translate_note_parascl(para_note, ctbl, ev.note);
            deb_txt = "para_sc:".to_string();
        } else {
            let option = self.specify_trans_option(next_tick, ev.note);
            if option == TRNS_PARA {
                let para_root = root - self.para_root_base;
                let mut tgt_nt = ev.note + para_root;
                if root > self.turnnote {
                    tgt_nt -= 12;
                }
                trans_note = translate_note_com(root, ctbl, tgt_nt);
                deb_txt = "para:".to_string();
            } else if option == TRNS_COM {
                trans_note = translate_note_com(root, ctbl, ev.note);
                deb_txt = "com:".to_string();
            } else if option == TRNS_NONE {
                trans_note = ev.note;
                deb_txt = "none:".to_string();
            } else {
                // Arpeggio
                //trans_note = NoteTranslation::translate_note_arp(root, ctbl, option);
                trans_note = translate_note_arp2(root, ctbl, ev.note, option, self.last_note);
                deb_txt = "arp:".to_string();
            }
        }
        self.last_note = trans_note;
        //crnt_ev[NOTE] = trans_note;
        (
            trans_note,
            deb_txt + &(root.to_string() + "-" + &ctbl.to_string()),
        )
    }
    fn specify_trans_option(&self, next_tick: i32, note: i16) -> i16 {
        for anaone in self.analys.iter() {
            if anaone.mtype == TYPE_BEAT && anaone.tick == next_tick as i16 && anaone.note == note {
                return anaone.atype;
            }
        }
        TRNS_COM
    }
}
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
        self.same_note_stuck = Vec::new();
        self.same_note_msr = 0;
        self.same_note_tick = 0;
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

        let elapsed_tick = self.calc_serial_tick(crnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = FULL;
            self.destroy = true;
            return;
        }

        if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_event(crnt_, estk, elapsed_tick);
            self.next_tick_in_phrase = next_tick;
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            } else {
                let (msr, tick) = self.gen_msr_tick(crnt_, self.next_tick_in_phrase);
                self.next_msr = msr;
                self.next_tick = tick;
            }
        }
    }
}
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
}
