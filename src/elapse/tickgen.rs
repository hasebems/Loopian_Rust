//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::{Beat, DEFAULT_BPM, DEFAULT_TICK_FOR_ONE_MEASURE, DEFAULT_TICK_FOR_QUARTER};
use std::time::Instant;

//*******************************************************************
//          Tick Generator Struct
//*******************************************************************
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
    fermata_state: bool, // fermata で止まっている状態
    ritgen: Box<dyn Rit>,
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
    pub fn new(tp: usize) -> Self {
        let rit: Box<dyn Rit>;
        match tp {
            0 => rit = Box::new(RitLinear::new()),
            _ => rit = Box::new(RitCtrl::new()),
        }
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
            ritgen: rit,
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
            // rit.
            let (addup_tick, rit_end) = self.ritgen.calc_tick_rit(crnt_time);
            if rit_end {
                // rit 終了
                let addup_msr = addup_tick / self.tick_for_onemsr;
                let real_tick = addup_tick % self.tick_for_onemsr;
                self.rit_state = false;
                self.crnt_msr = self.beat_start_msr + addup_msr;
                self.crnt_tick_inmsr = real_tick;
                self.beat_start_msr = self.crnt_msr;
                self.bpm_start_time = crnt_time;
                self.bpm_start_tick = real_tick;
                self.bpm = self.bpm_stock;
            } else {
                self.crnt_msr += addup_tick / self.tick_for_onemsr;
                self.crnt_tick_inmsr = addup_tick % self.tick_for_onemsr;
            }
        } else {
            // same bpm
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
        self.ritgen.get_real_bpm()
    }
    pub fn get_beat(&self) -> Beat {
        self.beat
    }
    pub fn start_rit(&mut self, start_time: Instant, ratio: i32, bar: i32, target_bpm: i16) {
        if ratio < 100 && !self.rit_state && !self.fermata_state {
            self.ritgen.set_rit(
                ratio,
                bar,
                self.bpm as f32,
                start_time,
                self.crnt_tick_inmsr,
                self.tick_for_onemsr,
            );
        }
        self.rit_state = true;
        self.beat_start_msr = self.crnt_msr;
        self.bpm_start_time = start_time;
        self.bpm_start_tick = self.crnt_tick_inmsr;
        self.bpm_stock = target_bpm;
    }
    fn calc_crnt_tick(&self) -> i32 {
        let diff = self.crnt_time - self.bpm_start_time;
        let elapsed_tick =
            ((DEFAULT_TICK_FOR_QUARTER as f32) * (self.bpm as f32) * diff.as_secs_f32()) / 60.0;
        elapsed_tick as i32 + self.bpm_start_tick
    }
}

//*******************************************************************
//          Rit. Trait (Super Class)
//*******************************************************************
pub trait Rit {
    // rit 開始時
    fn set_rit(
        &mut self,
        ratio: i32,           // 実装によって自由な値
        bar: i32,             // これから rit.する小節数, 0: 次の小節まで、1: 次の次の小節まで (何回小節跨ぎをスルーするか)
        bpm: f32,             // rit.前のテンポ
        start_time: Instant,  // 現在の時間
        start_tick: i32,      // 現在のtick
        tick_for_onemsr: i32, // 1小節の tick 数
    );

    // rit 中、定期的に呼ぶ None:rit終了、Some():rit開始時からの積算tick
    fn calc_tick_rit(
        &mut self,
        crnt_time: Instant,     // 現在の時間
    ) -> (i32, bool);

    //  現在の bpm を得る
    fn get_real_bpm(&self) -> i16;
}

//*******************************************************************
//          Rit. Linear Struct
//*******************************************************************
pub struct RitLinear {
    original_bpm: f32,
    start_time: Instant,
    start_tick: i32,
    tick_for_onemsr: i32,
    delta_bpm: i16,     // realtime に rit. で減るテンポ（微分値）
    delta_tps: f32,     // Tick per sec: tick の時間あたりの変化量、bpm 変化量を８倍した値
    rit_bar: i32,       // rit 受信後、何回小節線をスルーするか
    rit_bar_count: i32, // rit_bar を小節頭で inc.
    last_addup_tick: i32,
    last_addup_time: Instant,
    t0_time: f32,       // tempo=0 到達時間
    t0_addup_tick: i32, // tempo=0 到達時の積算tick
}

impl Rit for RitLinear {
    //==== rit. ======================
    // ratio  0:   tempo 停止
    //        50:  1secで tempo を 50%(1/2)
    //        100: 何もしない
    fn set_rit(
        &mut self,
        ratio: i32,
        bar: i32,
        bpm: f32,
        start_time: Instant,
        start_tick: i32,
        tick_for_onemsr: i32,
    ) {
        self.start_time = start_time;
        self.start_tick = start_tick;
        self.tick_for_onemsr = tick_for_onemsr;
        self.original_bpm = bpm;
        self.delta_tps = ((100.0 - ratio as f32) / 100.0) * 8.0 * bpm;
        self.t0_time = bpm * 8.0 / self.delta_tps; // tempo0 time
        self.t0_addup_tick = ((self.delta_tps / 2.0) * self.t0_time * self.t0_time) as i32;
        self.rit_bar = bar;
        self.rit_bar_count = 0;
    }
    fn calc_tick_rit(&mut self, crnt_time: Instant) -> (i32, bool) {
        // output: self.crnt_msr の更新
        let tick_from_rit_starts = self.calc_addup_tick_rit(crnt_time) + self.start_tick;
        if self.tick_for_onemsr * (self.rit_bar + 1) < tick_from_rit_starts {
            // reached last bar, and stop rit.
            //println!("@@@@:{}",tick_from_rit_starts);
            self.rit_bar = 0;
            self.delta_bpm = 0;
            (tick_from_rit_starts, true)
        } else {
            let r_msr = tick_from_rit_starts / self.tick_for_onemsr;
            let mut r_tick_inmsr = tick_from_rit_starts % self.tick_for_onemsr;
            if r_msr > self.rit_bar_count {
                // 小節線を超えたとき
                self.rit_bar_count += 1;
                r_tick_inmsr += self.tick_for_onemsr;
            }
            (r_tick_inmsr, false)
        }
    }
    fn get_real_bpm(&self) -> i16 {
        self.original_bpm as i16 - self.delta_bpm
    }
}
impl RitLinear {
    pub fn new() -> Self {
        Self {
            original_bpm: 0.0,
            start_time: Instant::now(),
            start_tick: 0,
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            delta_bpm: 0,
            delta_tps: 0.0,
            rit_bar: 0,
            rit_bar_count: 0,
            last_addup_tick: 0,
            last_addup_time: Instant::now(),
            t0_time: 0.0,
            t0_addup_tick: 0,
        }
    }
    fn calc_addup_tick_rit(&mut self, crnt_time: Instant) -> i32 {
        const MINIMUM_TEMPO: i16 = 20;
        let start_time = (crnt_time - self.start_time).as_secs_f32();
        let time_to0 = self.t0_time - start_time;
        self.delta_bpm = (self.delta_tps * start_time / 8.0) as i16;
        let addup_tick: i32;
        if self.original_bpm as i16 - self.delta_bpm > MINIMUM_TEMPO {
            // target bpm が MINIMUM_TEMPO 以上
            addup_tick = self.t0_addup_tick - (time_to0 * time_to0 * self.delta_tps / 2.0) as i32; // 積算Tickの算出
            self.last_addup_tick = addup_tick;
            self.last_addup_time = crnt_time;
        } else {
            self.delta_bpm = self.original_bpm as i16 - MINIMUM_TEMPO;
            addup_tick = self.last_addup_tick
                + (8.0 * (MINIMUM_TEMPO as f32) * (crnt_time - self.last_addup_time).as_secs_f32())
                    as i32;
        }
        addup_tick
    }
}

//*******************************************************************
//          Rit. Control Struct
//*******************************************************************
pub struct RitCtrl {

}

impl Rit for RitCtrl {
    //==== rit. ======================
    // ratio  0:   tempo 停止
    //        50:  1secで tempo を 50%(1/2)
    //        100: そのまま
    fn set_rit(
        &mut self,
        _ratio: i32,
        _bar: i32,
        _bpm: f32,
        _start_time: Instant,
        _start_tick: i32,
        _tick_for_onemsr: i32,
    ) {}
    fn calc_tick_rit(&mut self, _crnt_time: Instant) -> (i32, bool) {(0, false)}
    fn get_real_bpm(&self) -> i16 {0}
}

impl RitCtrl {
    pub fn new() -> Self {
        Self {

        }
    }
}
