//  Created by Hasebe Masahiko on 2025/06/21
//  Copyright (c) 2025 Hasebe Masahiko.
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
pub struct ClusterPattern {
    id: ElapseId,
    priority: u32,

    ptn_tick: i32,
    ptn_min_nt: i16,
    ptn_vel: i32,
    ptn_each_dur: i32,
    ptn_max_vce: i32,
    analys: Vec<AnaEvt>,
    ntlist: Vec<i16>, // only for cluster
    ntlist_vel: i16,  // for cluster

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
impl ClusterPattern {
    pub fn new(
        sid: u32,
        pid: u32,
        part: u32, // loop pid
        keynote: u8,
        msr: i32, // crnt_msr
        ptn: ClsPatternEvt,
        ana: Vec<AnaEvt>,
    ) -> Rc<RefCell<Self>> {
        // generate para_note_base
        let mut para = false;
        ana.iter().for_each(|x| {
            if let AnaEvt::Exp(e) = x {
                if e.atype == ExpType::ParaRoot {
                    para = true;
                }
            }
        });
        // generate staccato rate
        let mut staccato_rate = 90;
        ana.iter().for_each(|x| {
            if let AnaEvt::Exp(e) = x {
                if e.atype == ExpType::ParaRoot {
                    staccato_rate = e.cnt as i32;
                }
            }
        });

        #[cfg(feature = "verbose")]
        println!("New DynaPtn: para:{}", para);

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
            ntlist: Vec::new(), // only for cluster
            ntlist_vel: 0,      // for cluster
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
        if !self.ntlist.is_empty() {
            // すでに発音開始済みの Cluster の場合
            self.gen_note_ev(estk, self.ntlist[0], self.ntlist_vel);
            self.ntlist.remove(0);
            if !self.ntlist.is_empty() {
                self.next_tick
            } else {
                self.ntlist = Vec::new(); // Cluster のノートリストをクリア
                self.play_counter += 1;
                self.recalc_next_tick(crnt_)
            }
        } else if let Some(pt) = estk.part(self.part) {
            // 対応する Part が存在する場合
            let mut pt_borrowed = pt.borrow_mut();
            let cmp_med = pt_borrowed.get_cmps_med();
            // 和音情報を読み込む
            let (rt, tbl) = cmp_med.get_chord(crnt_);
            let root = ROOT2NTNUM[rt as usize];
            if tbl == NO_TABLE {
                #[cfg(feature = "verbose")]
                println!("ClusterPattern: No Chord Table!!");
            } else {
                #[cfg(feature = "verbose")]
                println!("ClusterPattern: root-{}, table-{}", root, tbl);
                self.gen_each_note(crnt_, estk, root, tbl);
            }
            // Cluster の場合は、これから数回に分けて発音リストに従って NoteOn する
            self.next_tick
        } else {
            self.recalc_next_tick(crnt_)
        }
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
        let (tblptr, _take_upper) = txt2seq_cmps::get_table(tbl as usize);
        let vel = self.calc_dynamic_vel(
            crnt_.tick_for_onemsr,
            estk.get_bpm(),
            estk.tg().get_meter().1,
        );

        // Cluster
        self.play_cluster(estk, root, tblptr, vel);
    }
    fn calc_dynamic_vel(&self, tick_for_onemsr: i32, bpm: i16, denomi: i32) -> i16 {
        let mut vel: i16 = self.ptn_vel as i16;
        if denomi == 8 {
            if (tick_for_onemsr / (DEFAULT_TICK_FOR_QUARTER / 2)) % 3 == 0 {
                vel = txt2seq_ana::calc_vel_for3_8(self.ptn_vel as i16, self.next_tick as f32, bpm);
            }
        } else {
            // denomi == 4
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

        if self.ptn_max_vce as usize <= ntlist.len() {
            // 最大発音数を超える場合は、最大発音数までにする
            ntlist.truncate(self.ptn_max_vce as usize);
        }
        self.ntlist = ntlist;
        self.gen_note_ev(estk, self.ntlist[0], vel); // 最低音を発音
        self.ntlist.remove(0);
        self.ntlist_vel = vel; // 発音ベロシティを保存
    }
    /// NoteOn イベントを生成
    fn gen_note_ev(&mut self, estk: &mut ElapseStack, note: i16, vel: i16) {
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
