//  Created by Hasebe Masahiko on 2023/05/18.
//  Copyright (c) 2023 Hasebe Masahiko.
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
use crate::cmd::txt2seq_cmps::*;
use crate::lpnlib::*;

//*******************************************************************
//          Flow Struct
//*******************************************************************
//  動作イメージ
//  ・elapse object であると同時に、Part にも集約される
//  ・Part から、制御命令（生成・消滅）、MIDI Inメッセージを受け取る
//      Part は、ElapseStack から、MIDI In メッセージを受け取る
//
//  ・MIDI In は 90 nn vv / 80 nn vv のみ。nn の等しいものが対となる
//      0  -  95 : 触った位置(MIDI In)
//      0  -  71 : 対応するノート番号
//

pub const TICK_RESOLUTION: i32 = 120;
struct NoteStock(Option<Rc<RefCell<Note>>>, u8, u8); // 0:note, 1:real_note, 2:locate

pub struct Flow {
    id: ElapseId,
    priority: u32,

    old_msr_tick: CrntMsrTick,
    note_stock: Vec<NoteStock>,
    keynote: u8,
    root: i16,
    translation_tbl: i16,
    tick_resolution: i32,
    set_velocity: i16,

    // for super's member
    during_play: bool,
    destroy: bool,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}

impl Flow {
    pub fn new(sid: u32, pid: u32, during_play: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpFlow,
            },
            priority: PRI_FLOW,
            old_msr_tick: CrntMsrTick {
                msr: 0,
                tick: 0,
                tick_for_onemsr: 0,
                ..Default::default()
            },
            note_stock: Vec::new(),
            keynote: 0,
            root: 0,
            translation_tbl: NO_TABLE,
            tick_resolution: TICK_RESOLUTION,
            set_velocity: 0,

            // for super's member
            during_play,
            destroy: false,
            next_msr: 1,
            next_tick: 0,
        }))
    }
    /// Flow オブジェクトを消滅させ、MIDI IN による発音を終了
    pub fn _deactivate(&mut self) {
        // 発音中の音をキャンセル
        self.destroy = true;
        self.during_play = false;
    }
    pub fn set_keynote(&mut self, keynote: u8) {
        self.keynote = keynote;
    }
    pub fn set_tick_resolution(&mut self, reso: i32) {
        self.tick_resolution = reso;
    }
    pub fn set_velocity(&mut self, vel: i16) {
        self.set_velocity = vel;
    }
    pub fn set_static_scale(&mut self, scale: i16) {
        self.translation_tbl = scale;
    }
    pub fn get_keynote(&self) -> u8 {
        self.keynote
    }
    pub fn rcv_midi(
        &mut self,
        estk_: &mut ElapseStack,
        crnt_: &CrntMsrTick,
        status: u8,
        locate: u8,
        vel: u8,
    ) {
        let locate = locate - 18;
        #[cfg(feature = "verbose")]
        println!("MIDI IN >> {:x}-{:x}-{:x}", status, locate, vel);
        let vel = if self.set_velocity != 0 {
            self.set_velocity as u8
        } else {
            vel
        };
        if !self.during_play {
            // ORBIT 自身の Pattern が鳴っていない時
            if self.translation_tbl != NO_TABLE {
                let mut real_note = self.note_appropriate(locate as i16);
                real_note = translate_note_com(0, self.translation_tbl, real_note);
                self.static_note_on_off(estk_, status, real_note, vel, locate);
            } else if (4..92).contains(&locate) {
                // locate >= 4 && locate < 92
                // 外部から Chord 情報が来ていない時
                // 3->21 A0, 90->108 C8
                estk_.midi_out_flow(status, locate + 18, vel);
            }
        } else {
            // 再生中
            if status & 0xf0 == 0x90 {
                if vel != 0 {
                    let (msr, tick) = if !self.note_stock.is_empty() {
                        self.calculate_tick(crnt_)
                    } else {
                        (crnt_.msr, crnt_.tick)
                    };
                    self.note_on_flow(estk_, crnt_, locate, vel, (msr, tick));
                } else {
                    self.note_off_flow(estk_, locate);
                }
            } else if status & 0xf0 == 0x80 {
                self.note_off_flow(estk_, locate);
            }
        }
    }
    fn static_note_on_off(
        &mut self,
        estk_: &mut ElapseStack,
        status: u8,
        real_note: u8,
        vel: u8,
        locate: u8,
    ) {
        if status & 0xf0 == 0x90 && vel != 0 {
            for nt in self.note_stock.iter_mut() {
                if nt.1 == real_note {
                    nt.2 = locate;
                    return; // 同じノートが連続している場合は、locate だけ更新
                }
            }
            self.note_stock.push(NoteStock(None, real_note, locate));
            estk_.midi_out_flow(status, real_note + self.keynote, vel);
        } else if (status & 0xf0 == 0x90 && vel == 0) || (status & 0xf0 == 0x80) {
            let mut del_number = None;
            for (i, nt) in self.note_stock.iter().enumerate() {
                if nt.2 == locate {
                    del_number = Some(i);
                    break;
                }
            }
            if let Some(d) = del_number {
                self.note_stock.remove(d);
                estk_.midi_out_flow(status, real_note + self.keynote, vel);
            }
        }
    }
    fn calculate_tick(&self, crnt_: &CrntMsrTick) -> (i32, i32) {
        let (msr, tick) = {
            let msr: i32;
            let tick: i32;
            let tk = (crnt_.tick / self.tick_resolution + 1) * self.tick_resolution;
            if tk >= crnt_.tick_for_onemsr {
                msr = crnt_.msr + 1;
                tick = tk - crnt_.tick_for_onemsr;
            } else {
                msr = crnt_.msr;
                tick = tk;
            }
            (msr, tick)
        };
        (msr, tick)
    }
    fn note_on_flow(
        &mut self,
        estk: &mut ElapseStack,
        crnt_: &CrntMsrTick,
        locate: u8,
        vel: u8,
        tk: (i32, i32),
    ) {
        let real_note = self.detect_real_note(estk, crnt_, locate as i16);
        let last = self.note_stock.len();
        if last >= 1 && self.note_stock[last - 1].1 == real_note {
            self.note_stock[last - 1].2 = locate;
            return; // 同じノートが連続している場合は、locate だけ更新
        }
        let ev = NoteEvt {
            tick: crnt_.tick as i16,
            dur: 0, // 必要ない
            note: real_note,
            floating: false,
            vel: vel as i16,
            trns: TrnsType::NoTrns,
            artic: 100,
        };
        let evt_tick = CrntMsrTick {
            msr: tk.0,
            tick: tk.1,
            tick_for_onemsr: crnt_.tick_for_onemsr,
            ..Default::default()
        };
        let nt: Rc<RefCell<Note>> = Note::new(
            (crnt_.msr * crnt_.tick_for_onemsr + crnt_.tick) as u32, //  unique number
            self.id.sid,                                             //  loop.sid -> note.pid
            NoteParam::new(
                &ev,
                format!(" Pt:{} Flow:{}", &self.id.pid, &self.id.sid),
                (self.keynote, evt_tick, self.id.pid, false, true),
            ),
        );
        self.note_stock
            .push(NoteStock(Some(Rc::clone(&nt)), real_note, locate));
        estk.add_elapse(nt);
    }
    fn note_off_flow(&mut self, estk: &mut ElapseStack, locate: u8) {
        let mut del_number = None;
        for (i, nt) in self.note_stock.iter().enumerate() {
            if let Some(nte) = &nt.0
                && nt.2 == locate
                && !nte.borrow().destroy_me()
            {
                del_number = Some(i);
                break;
            }
        }
        if let Some(d) = del_number {
            if let Some(nte) = &self.note_stock[d].0 {
                nte.borrow_mut().clear(estk);
            }
            self.note_stock.remove(d);
        }
    }
    fn check_destroy(&mut self) {
        loop {
            let mut del_number = None;
            for (i, nt) in self.note_stock.iter().enumerate() {
                if let Some(nte) = &nt.0
                    && nte.borrow().destroy_me()
                {
                    del_number = Some(i);
                    break;
                }
            }
            if let Some(d) = del_number {
                self.note_stock.remove(d);
            } else {
                break;
            }
        }
    }
    pub fn set_chord_for_noplay(&mut self, root: u8, tblnum: u8, keynote: u8) {
        self.root = root as i16;
        self.translation_tbl = tblnum as i16;
        self.keynote = keynote;
    }
    fn detect_real_note(&mut self, estk: &mut ElapseStack, crnt_: &CrntMsrTick, locate: i16) -> u8 {
        let mut real_note = self.note_appropriate(locate);
        if self.during_play {
            if let Some(pt) = estk.part(self.id.pid) {
                let mut pt_borrowed = pt.borrow_mut();
                let cmp_med = pt_borrowed.get_cmps_med();
                let (rt, ctbl) = cmp_med.get_chord(crnt_);
                if ctbl == NO_TABLE {
                    return real_note;
                }
                let root: i16 = get_note_from_root(rt);
                println!(">>>Real Note: {}", real_note);
                real_note = translate_note_com(root, ctbl, real_note);
            }
        } else {
            let root: i16 = get_note_from_root(self.root);
            real_note = translate_note_com(root, self.translation_tbl, real_note);
        }

        real_note.clamp(MIN_NOTE_NUMBER, MAX_NOTE_NUMBER)
    }
    fn note_appropriate(&self, locate: i16) -> u8 {
        let temp_note = (locate * 12) / 16 + 36;
        temp_note.clamp(MIN_NOTE_NUMBER as i16, MAX_NOTE_NUMBER as i16) as u8
    }
}

impl Elapse for Flow {
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
    /// User による start/play 時にコールされる
    fn start(&mut self, _msr: i32) {
        self.during_play = true;
    }
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {
        self.during_play = false;
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {}
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, _estk: &mut ElapseStack) {
        if crnt_.msr != self.old_msr_tick.msr {
            // 小節が変わった
            self.check_destroy();
        }
        self.old_msr_tick = *crnt_;
        self.next_msr = crnt_.msr + 1;
        self.next_tick = 0;
    }
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {
        self.destroy
    }
}
