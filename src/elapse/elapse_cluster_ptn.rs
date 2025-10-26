//  Created by Hasebe Masahiko on 2025/06/21
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::elapse_note::*;
use super::floating_tick::FloatingTick;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::cmd::txt2seq_ana;
use crate::cmd::{txt2seq_cmps, txt2seq_cmps::*};
use crate::lpnlib::*;

//*******************************************************************
//          Dynamic Pattern Struct
//*******************************************************************
pub struct ClusterPattern {
    id: ElapseId,
    priority: u32,

    ptn_tick: i32,
    ptn_min_nt: i16,
    ptn_vel: i32,
    ptn_each_dur: i32,
    ptn_max_vce: i32,
    analys: Vec<AnaEvt>,
    arpeggio: bool,

    part: u32,
    keynote: u8,
    play_counter: usize,
    last_note: i16,
    para: bool,
    staccato_rate: i32,
    flt: FloatingTick,    //   FloatingTick を保持する
    notational_msr: i32,  //   記譜上の小節番号
    notational_tick: i32, //   記譜上の Tick 数
    tick_for_onemsr: i32, //   1小節あたりの Tick 数

    // for super's member
    whole_tick: i32,
    destroy: bool,
    _first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl ClusterPattern {
    const MAX_FRONT_DISPERSE: i32 = 120; // Tick の前への最大散らし幅
    const EACH_DISPERSE: i32 = 60; // Tick の散らし幅の単位

    pub fn new(
        sid: u32,
        pid: u32,
        part: u32, // loop pid
        keynote: u8,
        mst: (i32, i32, i32, i32), // (notational_msr, real_msr, real_tick, tick_for_onemsr)
        ptn: ClsPatternEvt,
        ana: Vec<AnaEvt>,
    ) -> Rc<RefCell<Self>> {
        // generate para_note_base
        let mut para = false;
        ana.iter().for_each(|x| {
            match x {
                AnaEvt::Exp(e) if e.atype == ExpType::ParaRoot => para = true,
                _ => {}
            }
        });
        // generate staccato rate
        let mut staccato_rate = 90;
        ana.iter().for_each(|x| {
            match x {
                AnaEvt::Exp(e) if e.atype == ExpType::ParaRoot => staccato_rate = e.cnt as i32,
                _ => {}
            }
        });
        //   FloatingTick を生成する
        let floating = ptn.arpeggio > 0;
        let mut flt = FloatingTick::new(floating);
        flt.set_crnt(
            &CrntMsrTick {
                msr: mst.1,
                tick: mst.2,
                tick_for_onemsr: mst.3,
                ..Default::default()
            },
            &CrntMsrTick {
                msr: mst.0,
                tick: 0,
                tick_for_onemsr: mst.3,
                ..Default::default()
            },
        );

        #[cfg(feature = "verbose")]
        println!("New ClsPtn: para:{para}");
        // new Dynamic Pattern
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpClusterPattern,
            },
            priority: PRI_DYNPTN,
            ptn_tick: ptn.tick as i32,
            ptn_min_nt: ptn.lowest,
            ptn_vel: ptn.vel as i32,
            ptn_each_dur: ptn.each_dur as i32,
            ptn_max_vce: ptn.max_count as i32,
            analys: ana,
            arpeggio: floating,
            part,
            keynote,
            play_counter: 0,
            last_note: NO_NOTE as i16,
            para,
            staccato_rate,
            flt,
            notational_msr: mst.0,  //   記譜上の小節番号
            notational_tick: 0,     //   記譜上の Tick 数
            tick_for_onemsr: mst.3, //   1小節あたりの Tick 数

            // for super's member
            whole_tick: ptn.dur as i32,
            destroy: false,
            _first_msr_num: mst.0,
            next_msr: mst.1,
            next_tick: mst.2,
        }))
    }
    fn generate_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> i32 {
        if let Some(pt) = estk.part(self.part) {
            let mut pt_borrowed = pt.borrow_mut();
            let cmp_med = pt_borrowed.get_cmps_med();
            // 和音情報を読み込む
            let (rt, mut tbl) = cmp_med.get_chord(crnt_);
            let root = get_note_from_root(rt);
            if tbl == NO_TABLE {
                // 対応する Chord Table が存在しない場合
                tbl = 0;
            }
            #[cfg(feature = "verbose")]
            println!("ClusterPattern: root-{root}, table-{tbl}");
            let (tblptr_cow, vel) = self.gen_each_note(crnt_, estk, tbl);
            let tblptr: &[i16] = tblptr_cow.as_ref();
            let ntlist = self.gen_cluster_list(root, tblptr);
            self.play_cluster(estk, &ntlist, vel);

            self.play_counter += 1;
            self.recalc_next_tick(crnt_)
        } else {
            END_OF_DATA
        }
    }
    fn gen_each_note(
        &mut self,
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        tbl: i16,
    ) -> (std::borrow::Cow<'static, [i16]>, i16) {
        let (tblptr_cow, _take_upper) = txt2seq_cmps::get_table(tbl as usize);
        let vel = self.calc_dynamic_vel(
            crnt_.tick_for_onemsr,
            estk.get_bpm(),
            estk.tg().get_meter().1,
        );
        (tblptr_cow, vel)
    }
    fn calc_dynamic_vel(&self, tick_for_onemsr: i32, bpm: i16, denomi: i32) -> i16 {
        let mut vel: i16 = self.ptn_vel as i16;
        if denomi == 8 {
            if (tick_for_onemsr / (DEFAULT_TICK_FOR_QUARTER / 2)) % 3 == 0 {
                vel = txt2seq_ana::calc_vel_for3_8(
                    self.ptn_vel as i16,
                    self.notational_tick as f32,
                    bpm,
                );
            }
        } else {
            // denomi == 4
            if tick_for_onemsr == TICK_4_4 as i32 {
                vel = txt2seq_ana::calc_vel_for4(
                    self.ptn_vel as i16,
                    self.notational_tick as f32,
                    bpm,
                );
            } else if tick_for_onemsr == TICK_3_4 as i32 {
                vel = txt2seq_ana::calc_vel_for3(
                    self.ptn_vel as i16,
                    self.notational_tick as f32,
                    bpm,
                );
            }
        }
        vel
    }
    fn gen_cluster_list(&mut self, root: i16, tblptr: &[i16]) -> Vec<i16> {
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

        let max_vce = self.ptn_max_vce as usize;
        match max_vce.cmp(&ntlist.len()) {
            std::cmp::Ordering::Less => {
                // 最大発音数を超える場合は、最大発音数までにする
                ntlist.truncate(max_vce);
            }
            std::cmp::Ordering::Greater => {
                // 最大発音数が少ない場合は、最大発音数までにする
                ntlist.push(ntlist[0] + 12);
            }
            std::cmp::Ordering::Equal => {}
        }
        ntlist
    }
    fn play_cluster(&mut self, estk: &mut ElapseStack, ntlist: &[i16], vel: i16) {
        for (i, nt) in ntlist.iter().enumerate() {
            // 発音リストに従って、NoteOn イベントを生成
            let arp = if self.arpeggio {
                (i as i32 * Self::EACH_DISPERSE) - Self::MAX_FRONT_DISPERSE
            } else {
                0
            };
            let mut msr = self.notational_msr;
            let mut tick = self.notational_tick + arp;
            if tick < 0 {
                tick += self.tick_for_onemsr;
                msr -= 1;
            } else if tick >= self.tick_for_onemsr {
                tick -= self.tick_for_onemsr;
                msr += 1;
            }
            self.gen_note_ev(estk, *nt, vel, msr, tick);
        }
    }
    /// NoteOn イベントを生成
    fn gen_note_ev(&mut self, estk: &mut ElapseStack, note: i16, vel: i16, msr: i32, tick: i32) {
        let mut crnt_ev = NoteEvt {
            dur: self.ptn_each_dur as i16,
            note: note as u8,
            vel,
            ..NoteEvt::default()
        };

        //  Generate Note Struct
        if self.staccato_rate != 100 {
            let old = crnt_ev.dur as i32;
            crnt_ev.dur = ((old * self.staccato_rate) / 100) as i16;
        }

        //println!("  >>> NoteOn: {}, vel: {}, dur: {}", crnt_ev.note, crnt_ev.vel, crnt_ev.dur);
        let nt: Rc<RefCell<dyn Elapse>> = Note::new(
            self.play_counter as u32, //  read pointer
            self.id.sid,              //  loop.sid -> note.pid
            NoteParam::new(
                estk,
                &crnt_ev,
                format!(" / Pt:{} Lp:{}", &self.part, &self.id.sid),
                (self.keynote, msr, tick, self.part, self.arpeggio, false),
            ),
        );
        estk.add_elapse(Rc::clone(&nt));
    }
    fn recalc_next_tick(&mut self, crnt_: &CrntMsrTick) -> i32 {
        // 次の Tick を計算する
        let next_tick = self.notational_tick + self.ptn_each_dur;
        if next_tick >= crnt_.tick_for_onemsr || next_tick >= self.whole_tick {
            END_OF_DATA
        } else {
            next_tick
        }
    }
}

//*******************************************************************
//          Elapse IF for Dynamic Pattern
//*******************************************************************
impl Elapse for ClusterPattern {
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

        // crnt_ を記譜上の小節数に変換し、その後 elapsed_tick を計算する
        let ntcrnt_ = self.flt.convert_to_notational(crnt_);

        if ntcrnt_.msr > self.notational_msr || ntcrnt_.tick >= self.whole_tick + self.ptn_tick {
            self.next_msr = FULL;
            self.destroy = true;
        } else if ntcrnt_.tick >= self.notational_tick {
            self.notational_tick = self.generate_event(&ntcrnt_, estk);
            if self.notational_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            } else {
                let mt = CrntMsrTick {
                    msr: self.notational_msr,
                    tick: self.notational_tick,
                    tick_for_onemsr: ntcrnt_.tick_for_onemsr,
                    ..Default::default()
                };
                // FloatingTick を使って、次に呼ばれる実際の小節とTickを計算する
                if let Some(rlcrnt_) = self.flt.convert_to_real(&mt) {
                    //println!(
                    //    "  |__ ClusterPtn: next_msr/tick: {}/{}, crnt_msr/tick: {}/{}, ntcrnt_msr/tick:{}/{}",
                    //    rlcrnt_.msr, rlcrnt_.tick, crnt_.msr, crnt_.tick, ntcrnt_.msr, ntcrnt_.tick
                    //);
                    self.next_msr = rlcrnt_.msr;
                    self.next_tick = rlcrnt_.tick;
                } else {
                    self.next_msr = mt.msr;
                    self.next_tick = mt.tick;
                }
            }
        }
    }
}
