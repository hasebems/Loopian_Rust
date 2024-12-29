//  Created by Hasebe Masahiko on 2024/10/12
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::elapse_note::*;
use super::note_translation::*;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::txt2seq_ana;
use crate::cmd::txt2seq_cmps;
use crate::lpnlib::*;

//*******************************************************************
//          Dynamic Pattern Struct
//*******************************************************************
pub struct DynamicPattern {
    id: ElapseId,
    priority: u32,

    arp_available: bool,
    ptn_tick: i32,
    ptn_min_nt: i16,
    ptn_vel: i32,
    ptn_each_dur: i32,
    ptn_max_vce: i32,
    ptn_arp_type: i32,
    next_index: usize,  // for arp
    oct_up: i16,        // for arp
    note_close_to: i16, // for arp
    analys: Vec<AnaEvt>,

    part: u32,
    keynote: u8,
    play_counter: usize,
    last_note: i16,
    para: bool,
    staccato_rate: i32,

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl DynamicPattern {
    pub fn new(
        sid: u32,
        pid: u32,
        part: u32, // loop pid
        keynote: u8,
        msr: i32, // crnt_msr
        ptn: PhrEvt,
        ana: Vec<AnaEvt>,
    ) -> Rc<RefCell<Self>> {
        // generate para_note_base
        let mut para = false;
        ana.iter().for_each(|x| {
            if x.mtype == TYPE_EXP && x.atype == TRNS_PARA {
                para = true;
            }
        });
        // generate staccato rate
        let mut staccato_rate = 90;
        ana.iter().for_each(|x| {
            if x.mtype == TYPE_EXP && x.atype == ARTIC {
                staccato_rate = x.cnt as i32;
            }
        });
        let arp_available = ptn.mtype == TYPE_ARP;

        #[cfg(feature = "verbose")]
        println!("New DP: para:{}", para);

        // new Dynamic Pattern
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpDynamicPattern,
            },
            arp_available,
            priority: PRI_DYNPTN,
            ptn_tick: ptn.tick as i32,
            ptn_min_nt: ptn.note,
            ptn_vel: ptn.vel as i32,
            ptn_each_dur: ptn.each_dur as i32,
            ptn_max_vce: ptn.trns as i32,
            ptn_arp_type: ptn.trns as i32,
            next_index: 0,
            oct_up: 0,
            note_close_to: ptn.note,
            analys: ana,
            part,
            keynote,
            play_counter: 0,
            last_note: NO_NOTE as i16,
            para,
            staccato_rate,

            // for super's member
            whole_tick: ptn.dur as i32,
            destroy: false,
            first_msr_num: msr,
            next_msr: msr,
            next_tick: 0,
        }))
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> i32 {
        let root: i16;
        let tblptr: &[i16];
        if let Some(cmps) = estk.get_cmps(self.part as usize) {
            // 和音情報を読み込む
            let (rt, tbl) = cmps.borrow().get_chord();
            root = ROOT2NTNUM[rt as usize];
            let (ctbl, _take_upper) = txt2seq_cmps::get_table(tbl as usize);
            tblptr = ctbl;
        } else {
            #[cfg(feature = "verbose")]
            println!("DynamicPattern: No Chord Data!!");
            return END_OF_DATA;
        }
        let vel = self.calc_dynamic_vel(crnt_.tick_for_onemsr, estk.get_bpm(), estk.tg().get_meter().1);

        if self.arp_available {
            // Arpeggio
            self.play_arpeggio(estk, root, tblptr, vel);
        } else {
            // Cluster
            self.play_cluster(estk, root, tblptr, vel);
        }
        self.play_counter += 1;

        // 次回 tick 算出と終了の確認
        let next_tick = self.next_tick + self.ptn_each_dur;
        if next_tick >= crnt_.tick_for_onemsr || next_tick >= self.whole_tick {
            END_OF_DATA
        } else {
            next_tick
        }
    }
    fn calc_dynamic_vel(&self, tick_for_onemsr: i32, bpm: i16, denomi: i32) -> i16 {
        let mut vel: i16 = self.ptn_vel as i16;
        if denomi == 8 {
            if (tick_for_onemsr / (DEFAULT_TICK_FOR_QUARTER / 2)) % 3 == 0 {
                vel = txt2seq_ana::calc_vel_for3_8(self.ptn_vel as i16, self.next_tick as f32, bpm);
            }
        } else { // denomi == 4
            if tick_for_onemsr == TICK_4_4 as i32 {
                vel = txt2seq_ana::calc_vel_for4(self.ptn_vel as i16, self.next_tick as f32, bpm);
            } else if tick_for_onemsr == TICK_3_4 as i32 {
                vel = txt2seq_ana::calc_vel_for3(self.ptn_vel as i16, self.next_tick as f32, bpm);
            }
        }
        vel
    }
    fn play_cluster(&mut self, estk: &mut ElapseStack, root: i16, tblptr: &[i16], vel: i16) {
        // 最低ノートとpara設定から、各ノートのオクターブを算出
        let mut ntlist: Vec<i16> = Vec::new();
        for nt in tblptr {
            let mut note = *nt + DEFAULT_NOTE_NUMBER as i16;
            if self.para {
                while note < self.ptn_min_nt {
                    //展開
                    note += 12;
                }
                //並行移動
                note += root;
            } else {
                //並行移動
                note += root;
                while note < self.ptn_min_nt {
                    //最低音以下の音をオクターブアップ
                    note += 12;
                }
                while self.ptn_min_nt <= (note - 12) {
                    //最低音のすぐ上に降ろす
                    note -= 12;
                }
            }
            ntlist.push(note);
        }

        // 低い順に並べ、同時発音数を決定する
        ntlist.sort();
        //println!("Cluster::{:?}/{}", ntlist, self.keynote);
        let maxnt = if self.ptn_max_vce as usize > ntlist.len() {
            ntlist.len()
        } else {
            self.ptn_max_vce as usize
        };

        // Cluster発音
        for &note in ntlist.iter().take(maxnt) {
            self.gen_note_ev(estk, note, vel);
        }
    }
    fn play_arpeggio(&mut self, estk: &mut ElapseStack, root: i16, tblptr: &[i16], vel: i16) {
        let max_tbl_num = tblptr.len();
        let incdec_idx = |inc: bool, mut x, mut oct| -> (usize, i16) {
            if inc {
                x += 1;
                if x == max_tbl_num {
                    x = 0;
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
                if old_inc.is_none() {
                    old_inc = Some(inc);
                } else if let Some(oinc) = old_inc {
                    if oinc != inc {
                        self.next_index = index;
                        self.oct_up = oct_up;
                        break;
                    }
                }
                (index, oct_up) = incdec_idx(inc, index, oct_up);
            }
            note += post_add_nt;
        } else {
            (self.next_index, self.oct_up) = incdec_idx(up, self.next_index, self.oct_up);
            note = tblptr[self.next_index] + pre_add_nt + self.oct_up * 12;
            note += post_add_nt;
        }
        self.gen_note_ev(estk, note, vel);
    }
    fn gen_note_ev(&mut self, estk: &mut ElapseStack, note: i16, vel: i16) {
        let mut crnt_ev = PhrEvt {
            dur: self.ptn_each_dur as i16,
            note,
            vel,
            ..PhrEvt::default()
        };

        //  Generate Note Struct
        if self.staccato_rate != 100 {
            let old = crnt_ev.dur as i32;
            crnt_ev.dur = ((old * self.staccato_rate) / 100) as i16;
        }

        let nt: Rc<RefCell<dyn Elapse>> = Note::new(
            self.play_counter as u32, //  read pointer
            self.id.sid,              //  loop.sid -> note.pid
            NoteParam::new(
                estk,
                &crnt_ev,
                self.keynote,
                format!(" / Pt:{} Lp:{}", &self.part, &self.id.sid),
                self.first_msr_num,
                self.ptn_tick + self.ptn_each_dur * (self.play_counter as i32),
                self.part,
            ),
        );
        estk.add_elapse(Rc::clone(&nt));
    }
}

//*******************************************************************
//          Elapse IF for Dynamic Pattern
//*******************************************************************
impl Elapse for DynamicPattern {
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
