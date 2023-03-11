//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib;
use super::elapse::*;
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;

pub struct Note {
    id: ElapseId,
    priority: u32,

    note_num: u8,
    velocity: u8,
    duration: i32,
    keynote: u8,
    real_note: u8,
    noteon_started: bool,
    noteoff_enable: bool,
    destroy: bool,
    next_msr: i32,
    next_tick: i32,
    deb_txt: String,
}

impl Elapse for Note {
    fn id(&self) -> ElapseId {self.id}     // id を得る
    fn prio(&self) -> u32 {self.priority}  // priority を得る
    fn next(&self) -> (i32, i32) {    // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick)
    }
    fn start(&mut self) {}      // User による start/play 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack) {        // User による stop 時にコールされる
        if self.noteon_started {
            self.note_off(estk);
        }
    }
    fn fine(&mut self, estk: &mut ElapseStack) {        // User による fine があった次の小節先頭でコールされる
        if self.noteon_started {
            self.note_off(estk);
        }
    }
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {    // 再生 msr/tick に達したらコールされる
        if (crnt_.msr == self.next_msr && crnt_.tick >= self.next_tick) || (crnt_.msr > self.next_msr) {
            if !self.noteon_started {
                self.noteon_started = true;
                // midi note on
                self.note_on(estk);

                let tk = crnt_.tick_for_onemsr;
                let mut msrcnt = 0;
                let mut off_tick = self.next_tick + self.duration;
                while off_tick >= tk {
                    off_tick -= tk;
                    msrcnt += 1;
                }
                self.next_msr += msrcnt;
                self.next_tick = off_tick;
            }
            else {
                self.note_off(estk);
            }
        }
    }
    fn rcv_sp(&mut self, msg: ElapseMsg, msg_data: u8) {
        match msg {
            ElapseMsg::MsgNoSameNoteOff => {
                if self.real_note == msg_data {
                    self.noteoff_enable = false;
                }
            },
            _ => (),
        }
    }
    fn destroy_me(&self) -> bool {self.destroy}   // 自クラスが役割を終えた時に True を返す
}

impl Note {
    pub fn new(sid: u32, pid: u32, _estk: &mut ElapseStack, ev: &Vec<i16>, keynote: u8, deb_txt: String, 
        msr: i32, tick: i32)
      -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpNote,},
            priority: PRI_NOTE,
            note_num: ev[lpnlib::NOTE] as u8,
            velocity: ev[lpnlib::VELOCITY] as u8,
            duration: ev[lpnlib::DURATION] as i32,
            keynote,
            real_note: 0,
            noteon_started: false,
            noteoff_enable: true,
            destroy: false,
            next_msr: msr,
            next_tick: tick,
            deb_txt,
        }))
    }
    fn note_on(&mut self, estk: &mut ElapseStack) {
        let num = self.note_num + self.keynote;
        self.real_note = Note::note_limit(num, 0, 127);
        //if self.est.pianoteq_mode {
            estk.register_sp_cmnd(ElapseMsg::MsgNoSameNoteOff, self.real_note, self.id());
        //}
        self.noteoff_enable = true; // 上で false にされるので
        estk.midi_out(0x90, self.real_note, self.velocity);
        print!("NoteOn: {},{} NoteTranslate: ", self.real_note, self.velocity);
        println!("{}", self.deb_txt);
    }
    fn note_off(&mut self, estk: &mut ElapseStack) {
        self.destroy = true;
        self.next_msr = lpnlib::FULL;
        // midi note off
        if self.noteoff_enable {
            estk.midi_out(0x90, self.real_note, 0);
            println!("NoteOff: {}", self.real_note);
        }
    }
    fn note_limit(num: u8, min_value: i16, max_value: i16) -> u8 {
        let mut adj_num = num as i16;
        while adj_num > max_value {adj_num -= 12;}
        while adj_num < min_value {adj_num += 12;}
        adj_num as u8
    }
}