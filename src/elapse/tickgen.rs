//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::{Beat, DEFAULT_BPM, DEFAULT_TICK_FOR_ONE_MEASURE, DEFAULT_TICK_FOR_QUARTER};
use std::time::{Duration, Instant};

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
            1 => rit = Box::new(RitSigmoid::new()),
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
            self.gen_rit();
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
        self.beat_start_msr = msr;
        self.crnt_tick_inmsr = 0;
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
        if self.rit_state {
            self.ritgen.get_real_bpm()
        } else {
            self.bpm
        }
    }
    pub fn get_beat(&self) -> Beat {
        self.beat
    }
    pub fn get_origin_time(&self) -> Instant {
        self.origin_time
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
    fn gen_rit(&mut self) {
        let (addup_tick, rit_end) = self.ritgen.calc_tick_rit(self.crnt_time);
        if rit_end {
            // rit 終了
            let addup_msr = addup_tick / self.tick_for_onemsr;
            let real_tick = addup_tick % self.tick_for_onemsr;
            self.rit_state = false;
            self.crnt_msr = self.beat_start_msr + addup_msr;
            self.crnt_tick_inmsr = real_tick;
            self.beat_start_msr = self.crnt_msr;
            self.bpm_start_time = self.crnt_time;
            self.bpm_start_tick = real_tick;
            self.bpm = self.bpm_stock;
        } else {
            self.crnt_msr += addup_tick / self.tick_for_onemsr;
            self.crnt_tick_inmsr = addup_tick % self.tick_for_onemsr;
        }
    }
}

//*******************************************************************
//          Rit. Trait (Super Class)
//*******************************************************************
pub trait Rit {
    // rit 開始時に呼ばれる
    fn set_rit(
        &mut self,
        ratio: i32, // 継承によって自由な単位とする。通常は 0-100 の間で rit. の遅くなる度合いを調整する
        bar: i32, // これから rit.する小節数, 0: 次の小節まで、1: 次の次の小節まで (何回小節跨ぎをスルーするか)
        bpm: f32, // rit.開始時のテンポ
        start_time: Instant, // rit.開始時の時間
        start_tick: i32, // rit.開始時のtick
        tick_for_onemsr: i32, // 1小節の tick 数
    );

    // rit 中、定期的に呼ぶ None:rit終了、Some():rit開始時からの積算tick
    fn calc_tick_rit(
        &mut self,
        crnt_time: Instant, // 現在の時間
    ) -> (i32, bool); // (経過tick, true/false: rit終了したか)
                      // 2小節以上のrit.の場合、経過tickはrit開始小節からの累積tick

    //  現在の bpm を得る
    fn get_real_bpm(&self) -> i16; // 現在のテンポ
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
//          Rit. Sigmoid Struct
//*******************************************************************
const IDX_MAX: usize = 128;
#[rustfmt::skip]
const SIGMOID: [f32; IDX_MAX] = [// time -> tps(bpm)
    0.993307149,0.992767236,0.992184111,0.991554373,
    0.990874363,0.990140145,0.989347489,0.988491851,
    0.987568349,0.986571745,0.98549642,0.984336351,
    0.983085087,0.981735722,0.980280872,0.978712648,
    0.97702263,0.97520184,0.973240717,0.971129093,
    0.968856169,0.966410494,0.963779944,0.960951707,
    0.957912272,0.954647419,0.951142221,0.947381047,
    0.943347575,0.939024819,0.934395163,0.929440404,
    0.92414182,0.918480244,0.91243616,0.90598982,
    0.89912137,0.891811045,0.884039282,0.875786992,
    0.86703576,0.857768105,0.847967758,0.837619963,
    0.826711794,0.815232489,0.803173796,0.790530319,
    0.777299861,0.763483764,0.749087213,0.734119527,
    0.718594393,0.702530057,0.685949455,0.66888026,
    0.651354865,0.633410264,0.615087856,0.596433146,
    0.577495365,0.558326994,0.538983221,0.519521322,
    0.0,0.480478678,0.461016779,0.441673006,
    0.422504635,0.403566854,0.384912144,0.366589736,
    0.348645135,0.33111974,0.314050545,0.297469943,
    0.281405607,0.265880473,0.250912787,0.236516236,
    0.222700139,0.209469681,0.196826204,0.184767511,
    0.173288206,0.162380037,0.152032242,0.142231895,
    0.13296424,0.124213008,0.115960718,0.108188955,
    0.100878623,0.09401018,0.08756384,0.081519756,
    0.07585818,0.070559596,0.065604837,0.060975181,
    0.056652425,0.052618953,0.048857779,0.045352581,
    0.042087728,0.039048293,0.036220056,0.033589506,
    0.031143831,0.028870907,0.026759283,0.02479816,
    0.02297737,0.021287352,0.019719128,0.018264278,
    0.016914913,0.015663649,0.01450358,0.013428255,
    0.012431651,0.011508149,0.010652511,0.009859855,
    0.009125637,0.008445627,0.007815889,0.007232764,
    //0.006692851,
];
#[rustfmt::skip]
const INTEGRAL_SIGMOID: [f32; IDX_MAX] = [ // time -> tick
    0.0,0.015516206,0.031023639,0.046521595,
    0.06200932,0.077485996,0.092950743,0.108402613,
    0.123840583,0.139263553,0.154670335,0.170059654,
    0.185430134,0.200780296,0.216108551,0.231413188,
    0.24669237,0.261944123,0.277166331,0.29235672,
    0.307512855,0.322632126,0.337711738,0.352748704,
    0.367739829,0.382681702,0.397570683,0.412402896,
    0.427174214,0.441880248,0.456516342,0.471077557,
    0.485558668,0.499954153,0.514258187,0.52846464,
    0.542567072,0.556558731,0.570432562,0.584181205,
    0.597797007,0.611272038,0.624598099,0.637766753,
    0.650769345,0.663597035,0.676240834,0.688691647,
    0.70094032,0.712977693,0.724794653,0.736382206,
    0.747731533,0.758834068,0.769681564,0.780266172,
    0.790580508,0.800617736,0.810371628,0.819836635,
    0.829007952,0.837881564,0.8464543,0.854723867,
    0.862688877,0.870348867,0.8777043,0.884756564,
    0.891507952,0.897961635,0.904121628,0.909992736,
    0.915580508,0.920891172,0.925931564,0.930709068,
    0.935231533,0.939507206,0.943544653,0.947352693,
    0.95094032,0.954316647,0.957490834,0.960472035,
    0.963269345,0.965891753,0.968348099,0.970647038,
    0.972797007,0.974806205,0.976682562,0.978433731,
    0.980067072,0.98158964,0.983008187,0.984329153,
    0.985558668,0.986702557,0.987766342,0.988755248,
    0.989674214,0.990527896,0.991320683,0.992056702,
    0.992739829,0.993373704,0.993961738,0.994507126,
    0.995012855,0.99548172,0.995916331,0.996319123,
    0.99669237,0.997038188,0.997358551,0.997655296,
    0.997930134,0.998184654,0.998420335,0.998638553,
    0.998840583,0.999027613,0.999200743,0.999360996,
    0.99950932,0.999646595,0.999773639,0.999891206,
    //1.0,
];
pub struct RitSigmoid {
    ratio: i32,
    bar: i32,
    bpm: f32,
    end_bpm: f32,
    start_tick: i32,
    start_time: Instant,
    to_end: Duration, // end までの時間
    se_tick: i32,     // rit start-end の tick
    crnt_idx: usize,
    tick_for_onemsr: i32,
    crnt_tick: i32,
}
impl Rit for RitSigmoid {
    //==== rit. ======================
    // ratio  0:   目的地で tempo を 30
    //        50:  目的地で tempo を半分
    //        100: 目的地で tempo は変わらない
    fn set_rit(
        &mut self,
        ratio: i32,
        bar: i32,
        bpm: f32,
        start_time: Instant,
        start_tick: i32,
        tick_for_onemsr: i32,
    ) {
        self.ratio = ratio;
        self.bar = bar;
        self.bpm = bpm;
        self.start_tick = start_tick;
        self.start_time = start_time;
        self.tick_for_onemsr = tick_for_onemsr;
        self.se_tick = tick_for_onemsr - start_tick + tick_for_onemsr * bar;
        self.end_bpm = if bpm * (ratio as f32) / 100.0 > 30.0 {
            bpm * (ratio as f32) / 100.0
        } else {
            30.0
        };
        //self.to_end = Duration::from_micros((((self.se_tick as f32)/(bpm*8.0))*1000000.0) as u64);
        let t1 = (self.se_tick as f32) / (bpm * 8.0);
        self.to_end =
            Duration::from_micros((2.0 * bpm / (bpm + self.end_bpm) * t1 * 1000000.0) as u64);
        #[cfg(feature = "verbose")]
        {
            println!("end_bpm:::::{}", self.end_bpm);
            println!("tick:::::{}", start_tick);
            println!("se_tick:::::{}", self.se_tick);
            println!("org_end:::::{}", t1);
            println!("to_end:::::{:?}", self.to_end);
        }
    }
    fn calc_tick_rit(&mut self, crnt_time: Instant) -> (i32, bool) {
        let bunshi = crnt_time - self.start_time;
        self.crnt_idx = ((bunshi.as_secs_f32() / self.to_end.as_secs_f32()) * 128.0) as usize;
        if self.crnt_idx >= IDX_MAX {
            (self.se_tick + self.start_tick, true)
        } else {
            let tick_ratio = INTEGRAL_SIGMOID[self.crnt_idx];
            let crnt_tick = (((self.se_tick as f32) * tick_ratio) as i32) + self.start_tick;
            if self.crnt_tick != crnt_tick {
                self.crnt_tick = crnt_tick;
            }
            (crnt_tick, false)
        }
    }
    fn get_real_bpm(&self) -> i16 {
        let tps_ratio = if self.crnt_idx >= IDX_MAX {
            0.0
        } else {
            SIGMOID[self.crnt_idx]
        };
        let crnt_bpm = tps_ratio * (self.bpm - self.end_bpm) + self.end_bpm;
        crnt_bpm as i16
    }
}

impl RitSigmoid {
    pub fn new() -> Self {
        Self {
            ratio: 0,
            bar: 0,
            bpm: 0.0,
            end_bpm: 0.0,
            start_tick: 0,
            start_time: Instant::now(),
            to_end: Duration::from_secs(0),
            se_tick: 0,
            crnt_idx: 0,
            tick_for_onemsr: 0,
            crnt_tick: 0,
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
        _ratio: i32,
        _bar: i32,
        _bpm: f32,
        _start_time: Instant,
        _start_tick: i32,
        _tick_for_onemsr: i32,
    ) {
    }
    fn calc_tick_rit(&mut self, _crnt_time: Instant) -> (i32, bool) {
        (0, true)
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
