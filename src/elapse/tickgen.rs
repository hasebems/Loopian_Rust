//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::{Beat, DEFAULT_BPM, DEFAULT_TICK_FOR_ONE_MEASURE, DEFAULT_TICK_FOR_QUARTER};
use std::time::Instant;

pub struct TickGen {
    bpm: i16,
    beat: Beat,
    tick_for_onemsr: i32,    // beat によって決まる１小節の tick 数
    bpm_stock: i16,          // change bpm で BPM を変えた直後の値
    origin_time: Instant,    // start 時の絶対時間
    bpm_start_time: Instant, // tempo/beat が変わった時点の絶対時間、tick 計測の開始時間
    bpm_start_tick: i32,     // tempo が変わった時点の tick, beat が変わったとき0clear
    beat_start_msr: i32,     // beat が変わった時点の経過小節数
    crnt_msr: i32,           // start からの小節数（最初の小節からイベントを出すため、-1初期化)
    crnt_tick_inmsr: i32,    // 現在の小節内の tick 数
    crnt_time: Instant,      // 現在の時刻

    rit_state: bool,
    fermata_state: bool,    // fermata で止まっている状態
    minus_bpm_for_gui: i16, // realtime に rit. で減るテンポ
    last_addup_tick: i32,
    last_addup_time: Instant,
    t0_time: f32,       // tempo=0 到達時間
    t0_addup_tick: i32, // tempo=0 到達時の積算tick
    delta_tps: f32,     // Tick per sec: tick の時間あたりの変化量、bpm 変化量を８倍した値
    rit_bar: i32,       // rit 受信後、何回小節線をスルーするか
    rit_bar_count: i32, // rit_bar を小節頭で inc.
}
#[derive(Clone, Copy, PartialEq)]
pub struct CrntMsrTick {
    pub msr: i32,
    pub tick: i32,
    pub tick_for_onemsr: i32,
}
impl Default for CrntMsrTick {
    fn default() -> Self {
        Self {
            msr: 0,
            tick: 0,
            tick_for_onemsr: 0,
        }
    }
}

impl TickGen {
    pub fn new() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            beat: Beat(4, 4),
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            bpm_stock: DEFAULT_BPM,
            origin_time: Instant::now(),
            bpm_start_time: Instant::now(),
            bpm_start_tick: 0,
            beat_start_msr: 0,
            crnt_msr: -1,
            crnt_tick_inmsr: 0,
            crnt_time: Instant::now(),

            rit_state: false,
            fermata_state: false,
            minus_bpm_for_gui: 0,
            last_addup_tick: 0,
            last_addup_time: Instant::now(),
            t0_time: 0.0,
            t0_addup_tick: 0,
            delta_tps: 0.0,
            rit_bar: 0,
            rit_bar_count: 0,
        }
    }
    pub fn change_beat_event(&mut self, tick_for_onemsr: i32, beat: Beat) {
        self.rit_state = false;
        self.fermata_state = false;
        self.tick_for_onemsr = tick_for_onemsr;
        self.beat = beat;
        self.beat_start_msr = self.crnt_msr;
        self.bpm_start_time = self.crnt_time;
        self.bpm_start_tick = 0;
    }
    pub fn change_bpm(&mut self, bpm: i16) {
        self.bpm_stock = bpm;
    }
    fn change_bpm_event(&mut self, bpm: i16) {
        self.rit_state = false;
        self.fermata_state = false;
        self.bpm_start_tick = self.calc_crnt_tick();
        self.bpm_start_time = self.crnt_time; // Get current time
        self.bpm = bpm;
    }
    fn _change_fermata_event(&mut self) {
        self.rit_state = false;
        self.bpm_start_tick = self.calc_crnt_tick();
        self.bpm_start_time = self.crnt_time; // Get current time
        self.fermata_state = true; // 次回の gen_tick で反映
    }
    //pub fn calc_tick(&mut self)
    pub fn start(&mut self, time: Instant, bpm: i16, resume: bool) {
        self.rit_state = false;
        self.fermata_state = false;
        self.origin_time = time;
        self.crnt_time = time;
        self.bpm_start_tick = 0;
        self.bpm_start_time = time;
        self.bpm = bpm;
        self.bpm_stock = bpm;
        if resume {
            self.beat_start_msr = self.crnt_msr;
        } else {
            self.beat_start_msr = 0;
        }
    }
    pub fn gen_tick(&mut self, crnt_time: Instant) -> bool {
        let former_msr = self.crnt_msr;
        self.crnt_time = crnt_time;
        if self.rit_state {
            self.calc_tick_rit(crnt_time);
        } else {
            let tick_from_beat_starts = self.calc_crnt_tick();
            self.crnt_msr =
                (tick_from_beat_starts / self.tick_for_onemsr + self.beat_start_msr) as i32;
            self.crnt_tick_inmsr = tick_from_beat_starts % self.tick_for_onemsr;
        }
        let new_msr = self.crnt_msr != former_msr;
        if new_msr {
            if !self.rit_state && (self.bpm != self.bpm_stock) {
                // Tempo Change
                self.change_bpm_event(self.bpm_stock);
                if self.bpm == 0 {
                    // fermata
                    self.crnt_tick_inmsr = 0;
                }
            }
        }
        new_msr
    }
    pub fn get_crnt_msr_tick(&self) -> CrntMsrTick {
        CrntMsrTick {
            msr: self.crnt_msr,
            tick: self.crnt_tick_inmsr,
            tick_for_onemsr: self.tick_for_onemsr,
        }
    }
    pub fn get_tick(&self) -> (i32, i32, i32, i32) {
        let tick_for_beat = DEFAULT_TICK_FOR_ONE_MEASURE / (self.beat.1 as i32); // 一拍のtick数
        (
            (self.crnt_msr + 1).try_into().unwrap(),    // measure
            (self.crnt_tick_inmsr / tick_for_beat) + 1, // beat(1,2,3...)
            self.crnt_tick_inmsr % tick_for_beat,       // tick
            self.tick_for_onemsr / tick_for_beat,
        )
    }
    pub fn get_beat_tick(&self) -> (i32, i32) {
        (
            self.tick_for_onemsr,
            DEFAULT_TICK_FOR_ONE_MEASURE / (self.beat.1 as i32),
        )
    }
    pub fn get_bpm(&self) -> i16 {
        self.bpm
    }
    pub fn get_real_bpm(&self) -> i16 {
        self.bpm - self.minus_bpm_for_gui
    }
    pub fn get_beat(&self) -> Beat {
        self.beat
    }
    fn calc_crnt_tick(&self) -> i32 {
        let diff = self.crnt_time - self.bpm_start_time;
        let elapsed_tick =
            ((DEFAULT_TICK_FOR_QUARTER as f32) * (self.bpm as f32) * diff.as_secs_f32()) / 60.0;
        elapsed_tick as i32 + self.bpm_start_tick
    }

    //==== rit. ======================
    // ratio  0:   tempo 停止
    //        50:  1secで tempo を 50%(1/2)
    //        100: 何もしない
    // bar    0: 次の小節まで、1: 次の次の小節まで (何回小節跨ぎをスルーするか)
    pub fn start_rit(&mut self, start_time: Instant, ratio: i32, bar: i32, target_bpm: i16) {
        if ratio >= 100 || self.rit_state || self.fermata_state {
            return;
        } else {
            self.delta_tps = ((100.0 - ratio as f32) / 100.0) * 8.0 * (self.bpm as f32);
        }
        self.t0_time = (self.bpm as f32) * 8.0 / self.delta_tps; // tempo0 time
        self.t0_addup_tick = ((self.delta_tps / 2.0) * self.t0_time * self.t0_time) as i32;

        self.rit_state = true;
        self.beat_start_msr = self.crnt_msr;
        self.bpm_start_time = start_time;
        self.bpm_start_tick = self.crnt_tick_inmsr;
        self.rit_bar = bar;
        self.rit_bar_count = 0;
        self.bpm_stock = target_bpm;
    }
    fn calc_tick_rit(&mut self, crnt_time: Instant) {
        // output: self.crnt_msr の更新
        let tick_from_rit_starts = self.calc_current_tick_rit(crnt_time);
        if self.tick_for_onemsr * (self.rit_bar + 1) < tick_from_rit_starts {
            // reached last bar, and stop rit.
            self.rit_state = false;
            self.crnt_msr = self.beat_start_msr + self.rit_bar + 1;
            self.crnt_tick_inmsr = 0;

            self.beat_start_msr = self.crnt_msr;
            self.bpm_start_time = crnt_time;
            self.bpm_start_tick = 0;
            self.minus_bpm_for_gui = 0;
            self.rit_bar = 0;
            self.bpm = self.bpm_stock;
        } else {
            let r_msr = tick_from_rit_starts / self.tick_for_onemsr;
            let r_tick_inmsr = tick_from_rit_starts % self.tick_for_onemsr;
            if r_msr > self.rit_bar_count {
                // 小節線を超えたとき
                self.rit_bar_count += 1;
                self.crnt_msr += 1;
            }
            self.crnt_tick_inmsr = r_tick_inmsr;
        }
    }
    fn calc_current_tick_rit(&mut self, crnt_time: Instant) -> i32 {
        const MINIMUM_TEMPO: i16 = 20;
        let start_time = (crnt_time - self.bpm_start_time).as_secs_f32();
        let time_to0 = self.t0_time - start_time;
        self.minus_bpm_for_gui = (self.delta_tps * start_time / 8.0) as i16;
        let addup_tick: i32;
        if self.bpm - self.minus_bpm_for_gui > MINIMUM_TEMPO {
            // target bpm が MINIMUM_TEMPO 以上
            addup_tick = self.t0_addup_tick - (time_to0 * time_to0 * self.delta_tps / 2.0) as i32; // 積算Tickの算出
            self.last_addup_tick = addup_tick;
            self.last_addup_time = crnt_time;
        } else {
            self.minus_bpm_for_gui = self.bpm - MINIMUM_TEMPO;
            addup_tick = self.last_addup_tick
                + (8.0 * (MINIMUM_TEMPO as f32) * (crnt_time - self.last_addup_time).as_secs_f32())
                    as i32;
        }
        addup_tick + self.bpm_start_tick // 現在の tick
    }
}
