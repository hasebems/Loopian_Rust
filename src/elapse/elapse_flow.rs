//  Created by Hasebe Masahiko on 2023/05/18.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse::*;
use super::note_translation::*;
use super::stack_elapse;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
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
//  ・Event Stock
//      GenStock (note:u8, vel:u8, org_locate:u8) : 実際に鳴っている原因のイベントを保持する
//      gen_stock: Vec<GenStock>
//
//  ・Event State
//      raw_state[95] : Index は触った位置。イベントがあったタイミングが記載、ないときは NO_DATA

pub const LOCATION_ALL: usize = 96;
pub const _FLOWNOTE_ALL: usize = 72;
pub const TICK_RESOLUTION: i32 = 120;

struct RawEv(i32, i32, u8, u8, u8); //  0:msr, 1:tick, 2:status, 3:locate, 4: vel
struct GenStock(u8, u8, u8); // 0:note, 1:vel, 2:locate

pub struct Flow {
    id: ElapseId,
    priority: u32,

    old_msr_tick: CrntMsrTick,
    raw_state: [i32; LOCATION_ALL], // tickを格納 同じ場所に複数のイベントが来た場合に排除
    raw_ev: Vec<RawEv>,             // 外部からの MIDI In Ev 受信時に格納し、処理後に削除
    gen_stock: Vec<GenStock>,       // MIDI In Ev処理し、外部音源発音時に生成される
    keynote: u8,
    root: i16,
    translation_tbl: i16,

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
            },
            raw_state: [NO_DATA; LOCATION_ALL],
            raw_ev: Vec::new(),
            gen_stock: Vec::new(),
            keynote: 0,
            root: 0,
            translation_tbl: NO_TABLE,

            // for super's member
            during_play,
            destroy: false,
            next_msr: FULL, // not called process()
            next_tick: 0,
        }))
    }
    /// Flow オブジェクトを消滅させ、MIDI IN による発音を終了
    pub fn deactivate(&mut self) {
        // 発音中の音をキャンセル
        self.destroy = true;
        self.during_play = false;
    }
    pub fn set_keynote(&mut self, keynote: u8) {
        self.keynote = keynote;
    }
    pub fn rcv_midi(
        &mut self,
        estk_: &mut ElapseStack,
        crnt_: &CrntMsrTick,
        status: u8,
        locate: u8,
        vel: u8,
    ) {
        //println!("MIDI IN >> {:x}-{:x}-{:x}", status, locate, vel);
        if !self.during_play {
            // ORBIT 自身の Pattern が鳴っていない時
            if self.translation_tbl != NO_TABLE {
                if status & 0xf0 == 0x90 {
                    if vel != 0 {
                        self.flow_note_on(estk_, locate, vel);
                    } else {
                        self.flow_note_off(estk_, locate);
                    }
                } if status & 0xf0 == 0x80 {
                    self.flow_note_off(estk_, locate);
                }
            } else if locate >= 4 && locate < 92 {
                // 外部から Chord 情報が来ていない時
                // 4->21 A0, 91->108 C8
                estk_.midi_out_flow(status, locate + 17, vel);
            }
        } else {
            self.raw_ev
                .insert(0, RawEv(crnt_.msr, crnt_.tick, status, locate, vel));
            let tk = (crnt_.tick / TICK_RESOLUTION + 1) * TICK_RESOLUTION;
            if tk >= crnt_.tick_for_onemsr {
                self.next_msr = crnt_.msr + 1;
                self.next_tick = tk - crnt_.tick_for_onemsr;
            } else {
                self.next_msr = crnt_.msr;
                self.next_tick = tk;
            }
        }
    }
    /// 考え方：
    ///  on なら、まずノート変換し、同じ音が現在鳴っていなければ発音
    ///  鳴っていれば、位置を新しいイベントのものに差し替え
    ///  off なら、この音を鳴らしたイベントを locate から探し、その音を消す
    fn convert_evt(&mut self, estk: &mut ElapseStack) {
        loop {
            if let Some(ev) = self.raw_ev.pop() {
                let _ = ev.0; // warning 対策
                let ch_status = ev.2 & 0xf0;
                let locate_idx = if (ev.3 as usize) < LOCATION_ALL {
                    ev.3 as usize
                } else {
                    break;
                };
                if ch_status == 0x90 && ev.4 != 0x00 {
                    // on
                    if self.raw_state[locate_idx] != NO_DATA {
                        break;
                    }
                    self.raw_state[locate_idx] = ev.1;
                    self.flow_note_on(estk, ev.3, ev.4);
                } else if ch_status == 0x80 || (ch_status == 0x90 && ev.4 == 0x00) {
                    // off
                    self.raw_state[locate_idx] = NO_DATA;
                    self.flow_note_off(estk, ev.3);
                }
            } else {
                break;
            }
        }
        self.next_msr = FULL; // process() は呼ばれないようになる
    }
    fn flow_note_on(&mut self, estk: &mut ElapseStack, locate: u8, vel: u8) {
        let rnote = self.detect_real_note(estk, locate as i16);
        if let Some(idx) = self.same_note_index(rnote) {
            self.gen_stock[idx].2 = locate; // locate 差し替え
        } else {
            estk.inc_key_map(rnote, vel, self.id.pid as u8);
            estk.midi_out_flow(0x90, rnote, vel);
            println!("MIDI OUT<< 0x90:{:x}:{:x}", rnote, vel);
            self.gen_stock.push(GenStock(rnote, vel, locate));
        }
    }
    fn flow_note_off(&mut self, estk: &mut ElapseStack, locate: u8) {
        if let Some(idx) = self.same_locate_index(locate) {
            let rnote = self.gen_stock[idx].0;
            let snk = estk.dec_key_map(rnote);
            if snk == stack_elapse::SameKeyState::LAST {
                estk.midi_out_flow(0x90, rnote, 0); // test
            }
            println!("MIDI OUT<< 0x90:{:x}:0", rnote);
            self.gen_stock.remove(idx);
        }
    }
    fn detect_real_note(&mut self, estk: &mut ElapseStack, locate: i16) -> u8 {
        let mut temp_note = (locate * 12) / 16;
        //if self.id.pid / 2 == 0 {
        //    temp_note += 24
        //} else {
        temp_note += 36;
        //}
        if temp_note >= 128 {
            temp_note = 127;
        }
        let mut real_note: u8 = temp_note as u8;
        if self.during_play {
            if let Some(cmps) = estk.get_cmps(self.id.pid as usize) {
                let (rt, ctbl) = cmps.borrow().get_chord();
                let root: i16 = ROOT2NTNUM[rt as usize];
                real_note = translate_note_com(root, ctbl, temp_note) as u8;
            }
        } else {
            real_note = translate_note_com(self.root, self.translation_tbl, temp_note) as u8;
        }

        real_note += self.keynote;
        if real_note >= MAX_NOTE_NUMBER {
            real_note = MAX_NOTE_NUMBER;
        } else if real_note < MIN_NOTE_NUMBER {
            real_note = MIN_NOTE_NUMBER;
        }
        real_note
    }
    fn same_note_index(&self, rnote: u8) -> Option<usize> {
        for (i, x) in self.gen_stock.iter().enumerate() {
            if x.0 == rnote && x.1 != 0 {
                return Some(i);
            }
        }
        None
    }
    fn same_locate_index(&self, locate: u8) -> Option<usize> {
        for (i, x) in self.gen_stock.iter().enumerate() {
            if x.2 == locate {
                return Some(i);
            }
        }
        None
    }
    pub fn set_chord_for_noplay(&mut self, root: u8, tblnum: u8) {
        self.root = root as i16;
        self.translation_tbl = tblnum as i16;
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
    fn next(&self) -> (i32, i32) {
        (self.next_msr, self.next_tick)
    }
    /// User による start/play 時にコールされる
    fn start(&mut self) {
        self.during_play = true;
    }
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {
        self.during_play = false;
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if (crnt_.msr == self.next_msr
            && crnt_.tick / TICK_RESOLUTION == self.next_tick / TICK_RESOLUTION)
            || (crnt_.msr == self.next_msr + 1)
        {
            self.convert_evt(estk);
        }
        self.old_msr_tick = crnt_.clone();
    }
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {
        self.destroy
    }
}
