//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::time::Instant;
use crate::lpnlib::{Beat, DEFAULT_TICK_FOR_ONE_MEASURE, DEFAULT_TICK_FOR_QUARTER, DEFAULT_BPM};

pub struct TickGen {
    bpm: u32,
    beat: Beat,
    tick_for_onemsr: i32,
    origin_time: Instant,       // start 時の絶対時間
    bpm_start_time: Instant,    // tempo/beat が変わった時点の絶対時間、tick 計測の開始時間
    bpm_start_tick: i32,        // tempo が変わった時点の tick, beat が変わったとき0clear
    beat_start_msr: i32,        // beat が変わった時点の経過小節数
    crnt_msr: i32,              // start からの小節数（最初の小節からイベントを出すため、-1初期化)
    crnt_tick_inmsr: i32,       // 現在の小節内の tick 数
    crnt_time: Instant,         // 現在の時刻
}
pub struct CrntMsrTick {
    pub msr: i32,
    pub tick: i32,
    pub tick_for_onemsr: i32,
    pub new_msr: bool,
}

impl TickGen {
    pub fn new() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            beat: Beat(4,4),
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            origin_time: Instant::now(),
            bpm_start_time: Instant::now(),
            bpm_start_tick: 0,
            beat_start_msr: 0,
            crnt_msr: -1,
            crnt_tick_inmsr: 0,
            crnt_time: Instant::now(),
        }
    }
    pub fn _change_beat_event(&mut self, tick_for_onemsr: i32, beat: Beat) {
        self.tick_for_onemsr = tick_for_onemsr;
        self.beat = beat;
        self.beat_start_msr = self.crnt_msr;
        self.bpm_start_time = self.crnt_time;
        self.bpm_start_tick = 0;
    }
    pub fn change_bpm_event(&mut self, bpm: u32) {
        self.bpm_start_tick = self.calc_crnt_tick();
        self.bpm_start_time = self.crnt_time;  // Get current time
        self.bpm = bpm;
    }
    //pub fn calc_tick(&mut self)
    pub fn start(&mut self, time: Instant) {
        self.origin_time = time;
        self.bpm_start_time = time;
    }
    pub fn get_crnt_msr_tick(&mut self, crnt_time: Instant) -> CrntMsrTick {
        let former_msr = self.crnt_msr;
        self.crnt_time = crnt_time;
        let tick_from_beat_starts = self.calc_crnt_tick();
        self.crnt_msr = (tick_from_beat_starts/self.tick_for_onemsr + self.beat_start_msr) as i32;
        self.crnt_tick_inmsr = tick_from_beat_starts%self.tick_for_onemsr;
        CrntMsrTick {
            msr: self.crnt_msr,
            tick: self.crnt_tick_inmsr,
            tick_for_onemsr: self.tick_for_onemsr,
            new_msr: self.crnt_msr != former_msr,
        }
    }
    pub fn get_tick(&self) -> (i32, i32, i32, i32) {
        let tick_for_beat = DEFAULT_TICK_FOR_ONE_MEASURE/(self.beat.1 as i32);  // 一拍のtick数
        (   (self.crnt_msr + 1).try_into().unwrap(),    // measure
            (self.crnt_tick_inmsr/tick_for_beat) + 1, // beat(1,2,3...)
            self.crnt_tick_inmsr%tick_for_beat,       // tick
            self.tick_for_onemsr/tick_for_beat)
    }
    //pub fn get_tick_for_onemsr(&self) -> i32 {self.tick_for_onemsr}
    pub fn get_bpm(&self) -> u32 {self.bpm}
    pub fn _get_beat(&self) -> Beat {self.beat}
    fn calc_crnt_tick(&self) -> i32 {
        let diff = self.crnt_time - self.bpm_start_time;
        let elapsed_tick = ((DEFAULT_TICK_FOR_QUARTER as f32)*(self.bpm as f32)*diff.as_secs_f32())/60.0;
        elapsed_tick as i32 + self.bpm_start_tick
    }
}