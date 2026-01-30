//  Created by Hasebe Masahiko on 2024/10/12
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::floating_tick::*;
use super::note_translation::*;
use crate::cmd::{txt2seq_cmps, txt2seq_cmps::*};
use crate::elapse::elapse_base::*;
use crate::elapse::elapse_note::*;
use crate::elapse::stack_elapse::ElapseStack;
use crate::elapse::tickgen::CrntMsrTick;
use crate::lpnlib::*;

//*******************************************************************
//          Dynamic Pattern Struct
//*******************************************************************
pub struct BrokenPattern {
    id: ElapseId,
    priority: u32,

    ptn_tick: i32,
    ptn_vel: i32,
    ptn_amp: Amp,
    ptn_each_dur: i32,
    ptn_arp_type: i32,
    next_index: usize,
    oct_up: i16,
    note_close_to: i16,
    analys: Vec<AnaEvt>,

    part: u32,
    keynote: u8,
    play_counter: usize,
    last_note: i16,
    para: bool,
    staccato_rate: i32,
    flt: FloatingTick,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl BrokenPattern {
    pub fn new(
        sid: u32,
        pid: u32,
        part: u32, // loop pid
        keynote: u8,
        msr: i32, // crnt_msr
        ptn: BrkPatternEvt,
        ana: Vec<AnaEvt>,
    ) -> Rc<RefCell<Self>> {
        // generate para_note_base
        let mut para = false;
        ana.iter().for_each(|x| {
            if let AnaEvt::Exp(e) = x
                && e.atype == ExpType::ParaRoot
            {
                para = true;
            }
        });
        // generate staccato rate
        let mut staccato_rate = 90;
        ana.iter().for_each(|x| {
            if let AnaEvt::Exp(e) = x
                && e.atype == ExpType::ParaRoot
            {
                staccato_rate = e.cnt as i32;
            }
        });
        #[cfg(feature = "verbose")]
        println!("New BrkPtn: para:{para}");

        // new Dynamic Pattern
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpBrokenPattern,
            },
            priority: PRI_DYNPTN,
            ptn_tick: ptn.tick as i32,
            ptn_vel: ptn.vel as i32,
            ptn_amp: ptn.amp,
            ptn_each_dur: ptn.each_dur as i32,
            ptn_arp_type: ptn.figure as i32,
            next_index: 0,
            oct_up: 0,
            note_close_to: ptn.lowest,
            analys: ana,
            part,
            keynote,
            play_counter: 0,
            last_note: NO_NOTE as i16,
            para,
            staccato_rate,
            flt: FloatingTick::new(false),
            // for super's member
            whole_tick: ptn.dur as i32,
            destroy: false,
            first_msr_num: msr,
            next_msr: msr,
            next_tick: 0,
        }))
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> i32 {
        if let Some(pt) = estk.part(self.part) {
            // 対応する Part が存在する場合
            let mut pt_borrowed = pt.borrow_mut();
            let cmp_med = pt_borrowed.get_cmps_med();
            // 和音情報を読み込む
            let (rt, mut tbl) = cmp_med.get_chord(crnt_);
            let root = get_note_from_root(rt);
            if tbl == NO_TABLE {
                #[cfg(feature = "verbose")]
                println!("BrokenPattern: No Chord Table!!");
                tbl = 0; //  No Table の場合は、Table 0 を使用する
            } else {
                #[cfg(feature = "verbose")]
                println!("BrokenPattern: root-{root}, table-{tbl}");
            }
            self.gen_each_note(crnt_, estk, root, tbl);
        }
        self.recalc_next_tick(crnt_)
    }
    fn recalc_next_tick(&mut self, crnt_: &CrntMsrTick) -> i32 {
        // 次の Tick を計算する
        let next_tick = self.next_tick + self.ptn_each_dur;
        if next_tick >= crnt_.tick_for_onemsr || next_tick >= self.whole_tick {
            END_OF_DATA
        } else {
            next_tick
        }
    }
    fn gen_each_note(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, root: i16, tbl: i16) {
        let (tblptr_cow, _take_upper) = txt2seq_cmps::get_table(tbl as usize);
        let tblptr: &[i16] = tblptr_cow.as_ref();
        let vel = self.calc_dynamic_vel(
            crnt_.tick_for_onemsr,
            estk.get_bpm(),
            estk.tg().get_meter().1,
        );
        self.play_arpeggio(estk, root, tblptr, vel);
        self.play_counter += 1;
    }
    fn calc_dynamic_vel(&self, tick_for_onemsr: i32, bpm: i16, denomi: i32) -> i16 {
        let mut vel: i16 = self.ptn_vel as i16;
        if denomi == 8 {
            if (tick_for_onemsr / (DEFAULT_TICK_FOR_QUARTER / 2)) % 3 == 0 {
                vel = calc_vel_for3_8(self.ptn_vel as i16, self.next_tick as f32, bpm);
            }
        } else {
            // denomi == 4
            if tick_for_onemsr == TICK_4_4 as i32 {
                vel = calc_vel_for4(self.ptn_vel as i16, self.next_tick as f32, bpm);
            } else if tick_for_onemsr == TICK_3_4 as i32 {
                vel = calc_vel_for3(self.ptn_vel as i16, self.next_tick as f32, bpm);
            }
        }
        vel
    }
    /// アルペジオを再生する
    fn play_arpeggio(&mut self, estk: &mut ElapseStack, root: i16, tblptr: &[i16], vel: i16) {
        let max_tbl_num = tblptr.len();
        let incdec_idx = |inc: bool, mut x, mut oct| -> (usize, i16) {
            if inc {
                x += 1;
                if x >= max_tbl_num {
                    x -= max_tbl_num;
                    oct += 1;
                }
            } else if x == 0 {
                x = max_tbl_num - 1;
                oct -= 1;
            } else {
                x -= 1;
            }
            (x, oct)
        };

        let up = self.ptn_arp_type % 2 == 0;
        let mut pre_add_nt = DEFAULT_NOTE_NUMBER as i16;
        let mut post_add_nt = 0;
        if self.para {
            post_add_nt = root;
            if !up {
                pre_add_nt += 12;
            }
        } else if up {
            pre_add_nt += root - 12;
        } else {
            pre_add_nt += root + 12;
        }

        let mut note: i16;
        if self.play_counter == 0 {
            // アルペジオの最初の音を決める
            let mut index = 0;
            let mut oct_up: i16 = 0;
            let mut old_inc: Option<bool> = None;
            loop {
                note = tblptr[index] + pre_add_nt + oct_up * 12;
                if note == self.note_close_to {
                    self.next_index = index;
                    self.oct_up = oct_up;
                    break;
                }
                let inc = note < self.note_close_to;
                match old_inc {
                    None => {
                        old_inc = Some(inc);
                    }
                    Some(oinc) if oinc != inc => {
                        self.next_index = index;
                        self.oct_up = oct_up;
                        break;
                    }
                    _ => {}
                }
                (index, oct_up) = incdec_idx(inc, index, oct_up);
            }
            note += post_add_nt;
        } else {
            (self.next_index, self.oct_up) = incdec_idx(up, self.next_index, self.oct_up);
            let tbl_val = if self.next_index < max_tbl_num {
                tblptr[self.next_index]
            } else {
                0
            };
            note = tbl_val + pre_add_nt + self.oct_up * 12;
            note += post_add_nt;
        }
        self.gen_note_ev(estk, note, vel);
    }
    fn gen_note_ev(&mut self, estk: &mut ElapseStack, note: i16, vel: i16) {
        let mut crnt_ev = NoteEvt {
            dur: self.ptn_each_dur as i16,
            note: note as u8,
            vel,
            amp: self.ptn_amp,
            ..NoteEvt::default()
        };

        //  Generate Note Struct
        if self.staccato_rate != 100 {
            let old = crnt_ev.dur as i32;
            crnt_ev.dur = ((old * self.staccato_rate) / 100) as i16;
        }

        let mut evt_tick = CrntMsrTick {
            msr: self.first_msr_num,
            tick: self.ptn_tick + self.ptn_each_dur * (self.play_counter as i32),
            tick_for_onemsr: estk.tg().get_beat_tick().0,
            ..Default::default()
        };
        (evt_tick.msr, evt_tick.tick) = self.flt.disperse_tick(&evt_tick, estk.tg().get_bpm());
        let nt: Rc<RefCell<dyn Elapse>> = Note::new(
            self.play_counter as u32, //  read pointer
            self.id.sid,              //  loop.sid -> note.pid
            NoteParam::new(
                &crnt_ev,
                format!(" / Pt:{} Lp:{}", &self.part, &self.id.sid),
                (self.keynote, evt_tick, self.part, false),
            ),
        );
        estk.add_elapse(Rc::clone(&nt));
    }
}

//*******************************************************************
//          Elapse IF for Dynamic Pattern
//*******************************************************************
impl Elapse for BrokenPattern {
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
        self.next_tick = 0;
        self.next_msr = FULL;
        self.destroy = true;
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.analys = Vec::new();
        self.play_counter = 0;
        self.last_note = NO_NOTE as i16;
        self.next_msr = 0;
        self.next_tick = 0;
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {
        self.destroy
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.destroy {
            return;
        }

        if crnt_.msr > self.next_msr || crnt_.tick >= self.whole_tick + self.ptn_tick {
            self.next_msr = FULL;
            self.destroy = true;
        } else if crnt_.tick >= self.next_tick {
            let next_tick = self.generate_event(crnt_, estk);
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            } else {
                self.next_tick = next_tick;
            }
        }
    }
}
