//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::{Meter, DEFAULT_BPM, DEFAULT_TICK_FOR_ONE_MEASURE};
use std::time::{Duration, Instant};

//*******************************************************************
//          Tick Generator Struct
//*******************************************************************
pub struct TickGen {
    bpm: i16,
    meter: Meter,
    tick_for_onemsr: i32,    // meter によって決まる１小節の tick 数
    tick_for_beat: i32,      // 1拍の tick 数
    bpm_stock: i16,          // change bpm で BPM を変えた直後の値
    origin_time: Instant,    // start 時の絶対時間
    bpm_start_time: Instant, // tempo/meter が変わった時点の絶対時間、tick 計測の開始時間
    bpm_start_tick: i32,     // tempo が変わった時点の tick, meter が変わったとき0clear
    meter_start_msr: i32,    // meter が変わった時点の経過小節数
    crnt_msr: i32,           // start からの小節数（最初の小節からイベントを出すため、-1初期化)
    crnt_tick_inmsr: i32,    // 現在の小節内の tick 数
    crnt_time: Instant,      // 現在の時刻

    prepare_rit: bool, // rit. 開始準備中
    rit_state: bool,
    fermata_state: bool, // fermata で止まっている状態
    prm: RitPrm,
    start_mt: CrntMsrTick,
    ritgen: Box<dyn Rit>,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct CrntMsrTick {
    pub msr: i32,
    pub tick: i32,
    pub tick_for_onemsr: i32,
}
#[allow(dead_code)]
pub enum RitType {
    Linear,
    LinearPrecise,
    Sigmoid,
    Control,
}
impl TickGen {
    pub fn new(tp: RitType) -> Self {
        let rit: Box<dyn Rit> = match tp {
            RitType::Linear => Box::new(RitLinear::new()),
            RitType::LinearPrecise => Box::new(RitLinearPrecise::new()),
            RitType::Sigmoid => Box::new(RitSigmoid::new()),
            RitType::Control => Box::new(RitCtrl::new()),
        };
        Self {
            bpm: DEFAULT_BPM,
            meter: Meter(4, 4),
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            tick_for_beat: DEFAULT_TICK_FOR_ONE_MEASURE / 4,
            bpm_stock: DEFAULT_BPM,
            origin_time: Instant::now(),
            bpm_start_time: Instant::now(),
            bpm_start_tick: 0,
            meter_start_msr: 0,
            crnt_msr: -1,
            crnt_tick_inmsr: 0,
            crnt_time: Instant::now(),
            prepare_rit: false,
            rit_state: false,
            fermata_state: false,
            prm: RitPrm::default(),
            start_mt: CrntMsrTick::default(),
            ritgen: rit,
        }
    }
    pub fn change_beat_event(&mut self, tick_for_onemsr: i32, meter: Meter) {
        self.rit_state = false;
        self.fermata_state = false;
        self.tick_for_onemsr = tick_for_onemsr;
        self.meter = meter;
        self.meter_start_msr = self.crnt_msr;
        self.bpm_start_time = self.crnt_time;
        self.bpm_start_tick = 0;
        // DEFAULT_TICK_FOR_ONE_MEASURE を分母で割った値が 1拍の tick 数で正しい！
        self.tick_for_beat = DEFAULT_TICK_FOR_ONE_MEASURE / self.meter.1;
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
            self.meter_start_msr = self.crnt_msr;
        } else {
            self.meter_start_msr = 0;
        }
    }
    pub fn gen_tick(&mut self, crnt_time: Instant) -> (bool, bool, i32) {
        let former_msr = self.crnt_msr;
        let former_tick = self.crnt_tick_inmsr;
        self.crnt_time = crnt_time;
        if self.rit_state {
            self.gen_rit();
        } else {
            // same bpm
            let tick_from_meter_starts = self.calc_crnt_tick();
            self.crnt_msr = tick_from_meter_starts / self.tick_for_onemsr + self.meter_start_msr;
            self.crnt_tick_inmsr = tick_from_meter_starts % self.tick_for_onemsr;
            if self.prepare_rit && self.is_over(self.start_mt) {
                self.start_rit(self.crnt_time);
            }
        }
        let new_msr = self.crnt_msr != former_msr;
        if new_msr && !self.rit_state && (self.bpm != self.bpm_stock) {
            // Tempo Change
            self.change_bpm_event(self.bpm_stock);
            if self.bpm == 0 {
                // fermata
                self.crnt_tick_inmsr = 0;
            }
        }
        let beat_num = self.crnt_tick_inmsr / self.tick_for_beat;
        let new_beat = if new_msr {
            true
        } else {
            beat_num != former_tick / self.tick_for_beat
        };
        (new_msr, new_beat, beat_num)
    }
    pub fn get_crnt_msr_tick(&self) -> CrntMsrTick {
        let msr = if self.crnt_msr < 0 { 0 } else { self.crnt_msr }; // 0以上の値にする
        CrntMsrTick {
            msr,
            tick: self.crnt_tick_inmsr,
            tick_for_onemsr: self.tick_for_onemsr,
        }
    }
    pub fn set_crnt_msr(&mut self, msr: i32) {
        self.rit_state = false;
        self.fermata_state = false;
        self.origin_time = Instant::now();
        self.crnt_time = Instant::now();
        self.bpm_start_time = Instant::now();
        self.bpm_start_tick = 0;
        self.crnt_msr = msr;
        self.meter_start_msr = msr;
        self.crnt_tick_inmsr = 0;
    }
    pub fn get_tick(&self) -> (i32, i32, i32, i32) {
        (
            self.crnt_msr + 1,                               // measure
            (self.crnt_tick_inmsr / self.tick_for_beat) + 1, // beat(1,2,3...)
            self.crnt_tick_inmsr % self.tick_for_beat,       // tick
            self.tick_for_onemsr / self.tick_for_beat,
        )
    }
    pub fn get_beat_tick(&self) -> (i32, i32) {
        (self.tick_for_onemsr, self.tick_for_beat)
    }
    pub fn get_bpm(&self) -> i16 {
        self.bpm
    }
    pub fn get_real_bpm(&self) -> i16 {
        if self.rit_state {
            self.ritgen.get_real_bpm()
        } else {
            self.bpm
        }
    }
    pub fn get_meter(&self) -> Meter {
        self.meter
    }
    pub fn get_origin_time(&self) -> Instant {
        self.origin_time
    }
    fn calc_crnt_tick(&self) -> i32 {
        let diff = self.crnt_time - self.bpm_start_time;
        let elapsed_tick =
            ((self.tick_for_beat as f32) * (self.bpm as f32) * diff.as_secs_f32()) / 60.0;
        elapsed_tick as i32 + self.bpm_start_tick
    }
    /// rit. を開始準備する
    /// 現在の時間と tick を得るが、rit. 開始は拍にクオンタイズされるため、すぐに開始しない
    pub fn prepare_rit(&mut self, ratio: i32, bar: i32, target_bpm: i16) {
        self.prm = RitPrm {
            ratio,
            bar,
            tick_for_onemsr: self.tick_for_onemsr,
            tick_for_beat: self.tick_for_beat,
        };
        self.prepare_rit = true;
        self.rit_state = false;
        let crnt = self.get_crnt_msr_tick();
        self.start_mt = self.quantize_tick(crnt, self.meter.1);
        self.bpm_stock = target_bpm;
    }
    // rit. 開始
    fn start_rit(&mut self, start_time: Instant) {
        if self.prm.ratio < 100 && !self.rit_state && !self.fermata_state {
            self.ritgen
                .set_rit(self.bpm as f32, start_time, self.crnt_tick_inmsr, self.prm);
        }
        self.rit_state = true;
        self.meter_start_msr = self.crnt_msr;
        self.bpm_start_time = start_time;
    }
    fn gen_rit(&mut self) {
        let (addup_tick, cross_barline, rit_end) = self.ritgen.calc_tick_rit(self.crnt_time);
        self.crnt_msr += if cross_barline { 1 } else { 0 };
        self.crnt_tick_inmsr = addup_tick % self.tick_for_onemsr;
        if rit_end {
            // rit 終了
            self.prepare_rit = false;
            self.rit_state = false;
            self.meter_start_msr = self.crnt_msr;
            self.bpm_start_time = self.crnt_time;
            self.bpm_start_tick = self.crnt_tick_inmsr;
            self.prm = RitPrm::default();
            self.start_mt = CrntMsrTick::default();
            self.bpm = self.bpm_stock;
        }
    }
    fn is_over(&self, tgt: CrntMsrTick) -> bool {
        self.crnt_msr > tgt.msr || (self.crnt_msr == tgt.msr && self.crnt_tick_inmsr >= tgt.tick)
    }
    fn quantize_tick(&self, crnt: CrntMsrTick, denominator: i32) -> CrntMsrTick {
        let tick_for_beat = DEFAULT_TICK_FOR_ONE_MEASURE / denominator;
        let mut msr = crnt.msr;
        let mut tick = ((crnt.tick / tick_for_beat) + 1) * tick_for_beat;
        if tick >= crnt.tick_for_onemsr {
            msr += 1;
            tick = 0;
        }
        CrntMsrTick {
            msr,
            tick,
            tick_for_onemsr: self.tick_for_onemsr,
        }
    }
}

//*******************************************************************
//          Rit. Trait (Super Class)
//*******************************************************************
#[derive(Clone, Copy, PartialEq, Default)]
pub struct RitPrm {
    ratio: i32, // 継承によって自由な単位とする。通常は 0-100 の間で rit. の遅くなる度合いを調整する
    bar: i32, // これから rit.する小節数, 0: 次の小節まで、1: 次の次の小節まで (何回小節跨ぎをスルーするか)
    tick_for_onemsr: i32, // 1小節の tick 数
    tick_for_beat: i32, // 1拍の tick 数
}
pub trait Rit {
    // rit 開始時に呼ばれる
    fn set_rit(
        &mut self,
        bpm: f32,            // rit.開始時のテンポ
        start_time: Instant, // rit.開始時の時間
        start_tick: i32,     // rit.開始時のtick
        prm: RitPrm,         // rit.のパラメータ
    );

    // rit 中、定期的に呼ぶ None:rit終了、Some():rit開始時からの積算tick
    fn calc_tick_rit(
        &mut self,
        crnt_time: Instant, // 現在の時間
    ) -> (i32, bool, bool); // (経過tick, true/false: 小節を超えたか？, true/false: rit終了したか)
                            // 2小節以上のrit.の場合、経過tickはrit開始小節からの累積tick

    //  現在の bpm を得る
    fn get_real_bpm(&self) -> i16; // 現在のテンポ
}

//*******************************************************************
//          Rit. Linear Struct
//*******************************************************************
pub struct RitLinear {
    bpm2tps: f32,
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
        //        bar: i32,
        bpm: f32,
        start_time: Instant,
        start_tick: i32,
        prm: RitPrm, // rit.のパラメータ
    ) {
        self.bpm2tps = prm.tick_for_beat as f32 / 60.0;
        self.start_time = start_time;
        self.start_tick = start_tick;
        self.tick_for_onemsr = prm.tick_for_onemsr;
        self.original_bpm = bpm;
        self.delta_tps = ((100.0 - prm.ratio as f32) / 100.0) * self.bpm2tps * bpm;
        self.t0_time = bpm * self.bpm2tps / self.delta_tps; // tempo0 time
        self.t0_addup_tick = ((self.delta_tps / 2.0) * self.t0_time * self.t0_time) as i32;
        self.rit_bar = prm.bar;
        self.rit_bar_count = 0;
    }
    fn calc_tick_rit(&mut self, crnt_time: Instant) -> (i32, bool, bool) {
        // output: self.crnt_msr の更新
        let tick_from_rit_starts = self.calc_addup_tick_rit(crnt_time) + self.start_tick;
        if self.tick_for_onemsr * (self.rit_bar + 1) < tick_from_rit_starts {
            // reached last bar, and stop rit.
            self.rit_bar = 0;
            self.delta_bpm = 0;
            (tick_from_rit_starts, true, true)
        } else {
            let r_msr = tick_from_rit_starts / self.tick_for_onemsr;
            let mut cross_barline = false;
            if r_msr > self.rit_bar_count {
                // 小節線を超えたとき
                self.rit_bar_count += 1;
                cross_barline = true;
            }
            (tick_from_rit_starts, cross_barline, false)
        }
    }
    fn get_real_bpm(&self) -> i16 {
        self.original_bpm as i16 - self.delta_bpm
    }
}
impl RitLinear {
    pub fn new() -> Self {
        Self {
            bpm2tps: 0.0,
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
        self.delta_bpm = (self.delta_tps * start_time / self.bpm2tps) as i16;
        let addup_tick: i32;
        if self.original_bpm as i16 - self.delta_bpm > MINIMUM_TEMPO {
            // target bpm が MINIMUM_TEMPO 以上
            addup_tick = self.t0_addup_tick - (time_to0 * time_to0 * self.delta_tps / 2.0) as i32; // 積算Tickの算出
            self.last_addup_tick = addup_tick;
            self.last_addup_time = crnt_time;
        } else {
            self.delta_bpm = self.original_bpm as i16 - MINIMUM_TEMPO;
            addup_tick = self.last_addup_tick
                + (self.bpm2tps
                    * (MINIMUM_TEMPO as f32)
                    * (crnt_time - self.last_addup_time).as_secs_f32()) as i32;
        }
        addup_tick
    }
}

//*******************************************************************
//          Rit. Linear Precise Struct
//*******************************************************************
pub struct RitLinearPrecise {
    bpm2tps: f32,
    start_time: Instant,
    total_time: Duration,
    start_tick: i32,
    total_tick: i32,
    original_tps: i32,
    target_tps: i32,
    crnt_tps: i32,
    tick_for_onemsr: i32,
    bar_count: i32, // rit_bar を小節頭で inc.
}

impl Rit for RitLinearPrecise {
    //==== rit. ======================
    // ratio  0:   tempo 停止
    //        50:  最終到達点でテンポが 50%(1/2)
    //        100: 何もしない
    fn set_rit(
        &mut self,
        bpm: f32,
        start_time: Instant,
        start_tick: i32,
        prm: RitPrm, // rit.のパラメータ
    ) {
        self.bpm2tps = prm.tick_for_beat as f32 / 60.0;
        self.start_time = start_time;
        self.start_tick = start_tick;
        self.tick_for_onemsr = prm.tick_for_onemsr;
        self.original_tps = (bpm * self.bpm2tps) as i32;
        self.crnt_tps = self.original_tps;
        self.target_tps = (self.original_tps * prm.ratio) / 100;
        self.total_tick = (prm.tick_for_onemsr - start_tick) + (prm.bar * prm.tick_for_onemsr);
        let milli_sec = ((self.total_tick as f32) * 2.0)
            / (self.original_tps as f32 + self.target_tps as f32)
            * 1000.0;
        self.total_time = Duration::from_millis(milli_sec as u64);
        self.bar_count = 0;
        println!(
            ">>>Rit Status: total_tick:{:?}, total_time:{:?}",
            self.total_tick, self.total_time
        );
    }
    fn calc_tick_rit(&mut self, crnt_time: Instant) -> (i32, bool, bool) {
        let elapsed_time = crnt_time - self.start_time;
        let time_ratio = elapsed_time.as_secs_f32() / self.total_time.as_secs_f32();
        self.crnt_tps =
            self.original_tps - ((self.original_tps - self.target_tps) as f32 * time_ratio) as i32;
        let addup_tick = (((self.original_tps + self.crnt_tps) as f32 * elapsed_time.as_secs_f32())
            / 2.0) as i32;
        if addup_tick >= self.total_tick {
            // reached last bar, and stop rit.
            println!(">>>Rit End: elapsed_time:{:?}", elapsed_time);
            (self.start_tick + self.total_tick, true, true)
        } else {
            let return_tick = self.start_tick + addup_tick;
            let bar_cnt = return_tick / self.tick_for_onemsr;
            let mut cross_barline = false;
            if bar_cnt > self.bar_count {
                // 小節線を超えたとき
                self.bar_count = bar_cnt;
                cross_barline = true;
                println!(
                    ">>>Rit. Crossing barline: elapsed_tick:{:?}/{}",
                    return_tick, self.bar_count
                );
            }
            (return_tick, cross_barline, false)
        }
    }
    fn get_real_bpm(&self) -> i16 {
        (self.crnt_tps as f32 / self.bpm2tps) as i16
    }
}
impl RitLinearPrecise {
    pub fn new() -> Self {
        Self {
            bpm2tps: 0.0,
            start_time: Instant::now(),
            total_time: Duration::from_secs(0),
            start_tick: 0,
            total_tick: 0,
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            original_tps: 0,
            target_tps: 0,
            crnt_tps: 0,
            bar_count: 0,
        }
    }
}

//*******************************************************************
//          Rit. Sigmoid Struct
//*******************************************************************
const IDX_MAX: usize = 201;
#[rustfmt::skip]
#[allow(clippy::excessive_precision)]
const SIGMOID: [f32; IDX_MAX] = [
    1.0, 0.998, 0.997, 0.995, 0.994, 0.992, 0.99, 0.988, 0.987, 0.985,
    0.983, 0.981, 0.979, 0.977, 0.975, 0.972, 0.97, 0.968, 0.965, 0.963,
    0.961, 0.958, 0.955, 0.953, 0.95, 0.947, 0.944, 0.941, 0.938, 0.935,
    0.932, 0.929, 0.925, 0.922, 0.918, 0.915, 0.911, 0.907, 0.904, 0.9,
    0.896, 0.892, 0.887, 0.883, 0.879, 0.874, 0.87, 0.865, 0.861, 0.856,
    0.851, 0.846, 0.841, 0.836, 0.83, 0.825, 0.819, 0.814, 0.808, 0.803,
    0.797, 0.791, 0.785, 0.779, 0.772, 0.766, 0.76, 0.753, 0.747, 0.74,
    0.733, 0.726, 0.719, 0.712, 0.705, 0.698, 0.691, 0.683, 0.676, 0.668,
    0.661, 0.653, 0.646, 0.638, 0.63, 0.622, 0.614, 0.606, 0.598, 0.59,
    0.582, 0.574, 0.566, 0.558, 0.55, 0.541, 0.533, 0.525, 0.517, 0.508,
    0.5, 0.492, 0.483, 0.475, 0.467, 0.459, 0.45, 0.442, 0.434, 0.426,
    0.418, 0.41, 0.402, 0.394, 0.386, 0.378, 0.37, 0.362, 0.354, 0.347,
    0.339, 0.332, 0.324, 0.317, 0.309, 0.302, 0.295, 0.288, 0.281, 0.274,
    0.267, 0.26, 0.253, 0.247, 0.24, 0.234, 0.228, 0.221, 0.215, 0.209,
    0.203, 0.197, 0.192, 0.186, 0.181, 0.175, 0.17, 0.164, 0.159, 0.154,
    0.149, 0.144, 0.139, 0.135, 0.13, 0.126, 0.121, 0.117, 0.113, 0.108,
    0.104, 0.1, 0.096, 0.093, 0.089, 0.085, 0.082, 0.078, 0.075, 0.071,
    0.068, 0.065, 0.062, 0.059, 0.056, 0.053, 0.05, 0.047, 0.045, 0.042,
    0.039, 0.037, 0.035, 0.032, 0.03, 0.028, 0.025, 0.023, 0.021, 0.019,
    0.017, 0.015, 0.013, 0.012, 0.01, 0.008, 0.006, 0.005, 0.003, 0.002,
    0.0
];
#[rustfmt::skip]
#[allow(clippy::excessive_precision)]
const INTEGRAL_SIGMOID: [f32; IDX_MAX] = [
    0.01,0.02,0.03,0.04,0.05,0.059,0.069,0.079,
    0.089,0.099,0.109,0.118,0.128,0.138,0.147,0.157,
    0.167,0.176,0.186,0.196,0.205,0.215,0.224,0.234,
    0.243,0.253,0.262,0.271,0.281,0.29,0.299,0.308,
    0.317,0.327,0.336,0.345,0.354,0.363,0.372,0.381,
    0.39,0.399,0.408,0.417,0.425,0.434,0.443,0.451,
    0.46,0.468,0.477,0.485,0.494,0.502,0.51,0.518,
    0.527,0.535,0.543,0.551,0.559,0.566,0.574,0.582,
    0.59,0.597,0.605,0.612,0.62,0.627,0.634,0.642,
    0.649,0.656,0.663,0.67,0.677,0.684,0.69,0.697,
    0.703,0.71,0.716,0.723,0.729,0.735,0.741,0.747,
    0.753,0.759,0.765,0.771,0.776,0.782,0.787,0.793,
    0.798,0.803,0.808,0.813,0.818,0.823,0.828,0.833,
    0.838,0.842,0.847,0.851,0.855,0.86,0.864,0.868,
    0.872,0.876,0.879,0.883,0.887,0.891,0.894,0.898,
    0.901,0.904,0.907,0.911,0.914,0.917,0.92,0.922,
    0.925,0.928,0.931,0.933,0.936,0.938,0.941,0.943,
    0.945,0.947,0.95,0.952,0.954,0.956,0.957,0.959,
    0.961,0.963,0.965,0.966,0.968,0.969,0.971,0.972,
    0.974,0.975,0.976,0.978,0.979,0.98,0.981,0.982,
    0.983,0.984,0.985,0.986,0.987,0.988,0.989,0.989,
    0.99,0.991,0.991,0.992,0.993,0.993,0.994,0.994,
    0.995,0.995,0.996,0.996,0.997,0.997,0.997,0.998,
    0.998,0.998,0.998,0.999,0.999,0.999,0.999,0.999,
    1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,
];

pub struct RitSigmoid {
    bpm2tps: f32,
    start_time: Instant,
    total_time: Duration,
    start_tick: i32,
    total_tick: i32,
    original_tps: i32,
    target_tps: i32,
    crnt_tps: i32,
    tick_for_onemsr: i32,
    tps_ratio: f32,
    bar_count: i32, // rit_bar を小節頭で inc.
}

impl Rit for RitSigmoid {
    //==== rit. ======================
    // ratio  0:   tempo 停止
    //        50:  最終到達点でテンポが 50%(1/2)
    //        100: 何もしない
    fn set_rit(
        &mut self,
        bpm: f32,
        start_time: Instant,
        start_tick: i32,
        prm: RitPrm, // rit.のパラメータ
    ) {
        self.bpm2tps = prm.tick_for_beat as f32 / 60.0;
        self.start_time = start_time;
        self.start_tick = start_tick;
        self.tick_for_onemsr = prm.tick_for_onemsr;
        self.original_tps = (bpm * self.bpm2tps) as i32;
        self.crnt_tps = self.original_tps;
        self.target_tps = (self.original_tps * prm.ratio) / 100;
        self.total_tick = (prm.tick_for_onemsr - start_tick) + (prm.bar * prm.tick_for_onemsr);
        let milli_sec = ((self.total_tick as f32) * 2.0)
            / (self.original_tps as f32 + self.target_tps as f32)
            * 1000.0;
        self.total_time = Duration::from_millis(milli_sec as u64);
        self.tps_ratio = self.original_tps as f32 / self.target_tps as f32;
        self.bar_count = 0;
        println!(
            ">>>Rit Status: total_tick:{:?}, total_time:{:?}",
            self.total_tick, self.total_time
        );
    }
    fn calc_tick_rit(&mut self, crnt_time: Instant) -> (i32, bool, bool) {
        let elapsed_time = crnt_time - self.start_time;
        let time_index =
            (IDX_MAX as f32 * elapsed_time.as_secs_f32() / self.total_time.as_secs_f32()) as usize;
        let index_rate;
        let integral_sig;
        if time_index >= IDX_MAX {
            // reached last bar, and stop rit.
            self.crnt_tps = self.target_tps;
            index_rate = 1.0;
            integral_sig = 1.0;
        } else {
            self.crnt_tps = self.target_tps
                + ((self.original_tps - self.target_tps) as f32 * SIGMOID[time_index]) as i32;
            index_rate = time_index as f32 / IDX_MAX as f32;
            integral_sig = INTEGRAL_SIGMOID[time_index];
        }
        let tps_rate =
            2.0 * self.target_tps as f32 / (self.original_tps as f32 - self.target_tps as f32);
        let addup_base = (integral_sig + (tps_rate * index_rate)) / (1.0 + tps_rate);
        let addup_tick = (addup_base * (self.total_tick as f32)) as i32;
        if addup_tick >= self.total_tick {
            // reached last bar, and stop rit.
            println!(">>>Rit End: elapsed_time:{:?}", elapsed_time);
            (self.start_tick + self.total_tick, true, true)
        } else {
            let return_tick = self.start_tick + addup_tick;
            let bar_cnt = return_tick / self.tick_for_onemsr;
            let mut cross_barline = false;
            if bar_cnt > self.bar_count {
                // 小節線を超えたとき
                self.bar_count = bar_cnt;
                cross_barline = true;
                println!(
                    ">>>Rit. Crossing barline: elapsed_tick:{:?}/{}",
                    return_tick, self.bar_count
                );
            }
            (return_tick, cross_barline, false)
        }
    }
    fn get_real_bpm(&self) -> i16 {
        (self.crnt_tps as f32 / self.bpm2tps) as i16
    }
}
impl RitSigmoid {
    pub fn new() -> Self {
        Self {
            bpm2tps: 0.0,
            start_time: Instant::now(),
            total_time: Duration::from_secs(0),
            start_tick: 0,
            total_tick: 0,
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            original_tps: 0,
            target_tps: 0,
            crnt_tps: 0,
            tps_ratio: 0.0,
            bar_count: 0,
        }
    }
}

//*******************************************************************
//          Rit. Control Struct
//*******************************************************************
pub struct RitCtrl {}

impl Rit for RitCtrl {
    //==== rit. ======================
    // ratio  0:   tempo 停止
    //        50:  1secで tempo を 50%(1/2)
    //        100: そのまま
    fn set_rit(
        &mut self,
        _bpm: f32,
        _start_time: Instant,
        _start_tick: i32,
        _prm: RitPrm, // rit.のパラメータ
    ) {
    }
    fn calc_tick_rit(&mut self, _crnt_time: Instant) -> (i32, bool, bool) {
        (0, true, true)
    }
    fn get_real_bpm(&self) -> i16 {
        0
    }
}

impl RitCtrl {
    pub fn new() -> Self {
        Self {}
    }
}
