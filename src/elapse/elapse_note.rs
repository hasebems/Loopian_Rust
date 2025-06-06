//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use rand;
use rand::prelude::Distribution;
use rand_distr::Normal;
use std::cell::RefCell;
use std::rc::Rc;

use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use super::{elapse_base::*, stack_elapse};
use crate::lpnlib::*;

//*******************************************************************
//          Note Event Struct
//*******************************************************************
pub struct NoteParam<'a> {
    _estk: &'a mut ElapseStack,
    ev: &'a NoteEvt,
    keynote: u8,
    _deb_txt: String,
    msr: i32,
    tick: i32,
    part: u32,
}
impl<'a> NoteParam<'a> {
    pub fn new(
        _estk: &'a mut ElapseStack,
        ev: &'a NoteEvt,
        keynote: u8,
        _deb_txt: String,
        msr: i32,
        tick: i32,
        part: u32,
    ) -> Self {
        Self {
            _estk,
            ev,
            keynote,
            _deb_txt,
            msr,
            tick,
            part,
        }
    }
}
pub struct Note {
    id: ElapseId,
    priority: u32,

    note_num: u8,
    velocity: u8,
    duration: i32,
    keynote: u8,
    real_note: u8,
    noteon_started: bool,
    destroy: bool,
    next_msr: i32,
    next_tick: i32,
    part: u32,
    _deb_txt: String,
}
impl Note {
    pub fn new(
        sid: u32,
        pid: u32,
        prm: NoteParam,
        //_estk: &mut ElapseStack,
        //ev: &PhrEvt,
        //keynote: u8,
        //_deb_txt: String,
        //msr: i32,
        //tick: i32,
        //part: u32,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpNote,
            },
            priority: PRI_NOTE,
            note_num: prm.ev.note,
            velocity: prm.ev.vel as u8,
            duration: prm.ev.dur as i32,
            keynote: prm.keynote,
            real_note: 0,
            noteon_started: false,
            destroy: false,
            next_msr: prm.msr,
            next_tick: prm.tick,
            part: prm.part,
            _deb_txt: prm._deb_txt,
        }))
    }
    fn note_on(&mut self, estk: &mut ElapseStack) -> bool {
        let num = self.note_num + self.keynote;
        let bpm = estk.tg().get_bpm();
        let beat = estk.tg().get_meter();
        self.duration = Self::auto_duration(bpm, beat, self.duration);
        if Note::note_limit_available(num, MIN_NOTE_NUMBER, MAX_NOTE_NUMBER) {
            self.real_note = num;
            let vel = self.random_velocity(self.velocity);
            estk.inc_key_map(num, vel, self.part as u8);
            estk.midi_out(0x90, self.real_note, vel);
            #[cfg(feature = "verbose")]
            println!(
                "On: N{} V{} D{} Trns: {}, ",
                num, vel, self.duration, self._deb_txt
            );
            true
        } else {
            println!("NoteOn: => Note Limit Failed!! Num:{}", num);
            false
        }
    }
    fn note_off(&mut self, estk: &mut ElapseStack) {
        self.destroy = true;
        self.next_msr = FULL;
        // midi note off
        let snk = estk.dec_key_map(self.real_note);
        if snk == stack_elapse::SameKeyState::Last {
            estk.midi_out(0x90, self.real_note, 0);
            #[cfg(feature = "verbose")]
            println!("Off: N{}, ", self.real_note);
        }
    }
    fn note_limit_available(num: u8, min_value: u8, max_value: u8) -> bool {
        (min_value..=max_value).contains(&num)
    }
    fn random_velocity(&self, input_vel: u8) -> u8 {
        let mut rng = rand::rng();
        // std_dev: 標準偏差
        let dist = Normal::<f64>::new(0.0, 3.0).unwrap();
        let diff = dist.sample(&mut rng) as i32;
        if input_vel as i32 + diff > 0 && input_vel as i32 + diff < 128 {
            (input_vel as i32 + diff) as u8
        } else {
            input_vel
        }
    }
    fn auto_duration(bpm: i16, meter: Meter, dur: i32) -> i32 {
        // 0.3 秒以内の音価なら、音価はそのまま
        // それ以上の音価なら、10%程度短くなる
        // bpm が 30 ～ 300 の範囲に収まるように調整(fermata のbpm=0に反応しない)
        let bpm = bpm.clamp(30, 300) as f32;
        let beat_per_sec = bpm / 60.0;
        let note_per_beat = (dur as f32) / (1920.0 / (meter.1 as f32));
        let sec = note_per_beat / beat_per_sec;
        let real_sec = if sec > 0.3 {
            sec - (sec * 0.1 - 0.03)
        } else {
            sec
        };
        (real_sec * bpm * 1920.0 / (60.0 * (meter.1 as f32))) as i32
    }
}
impl Elapse for Note {
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
    fn start(&mut self, _msr: i32) {}
    /// User による stop 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack) {
        if self.noteon_started {
            self.note_off(estk);
        }
    }
    /// 再生データを消去
    fn clear(&mut self, estk: &mut ElapseStack) {
        if self.noteon_started {
            self.note_off(estk);
        }
        self.destroy = true;
    }
    /// 再生処理 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if (crnt_.msr == self.next_msr && crnt_.tick >= self.next_tick)
            || (crnt_.msr > self.next_msr)
        {
            if !self.noteon_started {
                // midi note on
                self.noteon_started = self.note_on(estk);
                if !self.noteon_started {
                    // illegal
                    self.destroy = true;
                    self.next_msr = FULL;
                    return;
                }

                let tk = crnt_.tick_for_onemsr;
                let mut msrcnt = 0;
                let mut off_tick = self.next_tick + self.duration;
                while off_tick >= tk {
                    off_tick -= tk;
                    msrcnt += 1;
                }
                self.next_msr += msrcnt;
                self.next_tick = off_tick;
            } else {
                self.note_off(estk);
            }
        }
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {
        self.destroy
    } // 自クラスが役割を終えた時に True を返す
}

//*******************************************************************
//          Damper Event Struct
//*******************************************************************
pub struct Damper {
    id: ElapseId,
    priority: u32,
    position: u8,
    duration: i32,
    damper_started: bool,
    destroy: bool,
    next_msr: i32,
    next_tick: i32,
}
impl Damper {
    pub fn new(
        sid: u32,
        pid: u32,
        _estk: &mut ElapseStack,
        ev: &DmprEvt,
        msr: i32,
        tick: i32,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpNote,
            },
            priority: PRI_NOTE,
            position: ev.position as u8,
            duration: ev.dur as i32,
            damper_started: false,
            destroy: false,
            next_msr: msr,
            next_tick: tick,
        }))
    }
    fn damper_on(&mut self, estk: &mut ElapseStack) {
        let pos = if self.position > 127 {
            127
        } else {
            self.position
        };
        estk.midi_out(0xb0, 0x40, pos);
        #[cfg(feature = "verbose")]
        println!("Damper-On: {}", self.position);
    }
    fn damper_off(&mut self, estk: &mut ElapseStack) {
        self.destroy = true;
        self.next_msr = FULL;
        // midi damper off
        estk.midi_out(0xb0, 0x40, 0);
        #[cfg(feature = "verbose")]
        println!("Damper-Off");
    }
}
impl Elapse for Damper {
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
    fn start(&mut self, _msr: i32) {}
    /// User による stop 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack) {
        if self.damper_started {
            self.damper_off(estk);
        }
    }
    /// 再生データを消去
    fn clear(&mut self, estk: &mut ElapseStack) {
        if self.damper_started {
            self.damper_off(estk);
        }
        self.destroy = true;
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if (crnt_.msr == self.next_msr && crnt_.tick >= self.next_tick)
            || (crnt_.msr > self.next_msr)
        {
            if !self.damper_started {
                self.damper_started = true;
                // midi note on
                self.damper_on(estk);

                let tk = crnt_.tick_for_onemsr;
                let mut msrcnt = 0;
                let mut off_tick = self.next_tick + self.duration;
                while off_tick >= tk {
                    off_tick -= tk;
                    msrcnt += 1;
                }
                self.next_msr += msrcnt;
                self.next_tick = off_tick;
            } else {
                self.damper_started = false;
                self.damper_off(estk);
            }
        }
    }
    fn rcv_sp(&mut self, msg: ElapseMsg, _msg_data: u8) {
        let _ = msg;
    }
    fn destroy_me(&self) -> bool {
        self.destroy
    } // 自クラスが役割を終えた時に True を返す
}
