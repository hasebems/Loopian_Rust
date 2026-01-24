//  Created by Hasebe Masahiko on 2025/06/21
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::tickgen::CrntMsrTick;
use crate::lpnlib::*;
use rand_distr::{Distribution, Normal};

/// 本クラスは、PhraseLoop の next_tick を入力すると、その値を時間方向に散らす。
/// Arpeggio のとき和音の Tick をずらしたり、Tick をランダムにしたりするために使用する。
pub struct FloatingTick {
    just_crnt: CrntMsrTick,            //   現在の小節とTickの情報が保持される
    next_real_tick: CrntMsrTick,       //   次に呼ばれる小節とTickの情報が保持される
    next_notational_tick: CrntMsrTick, //   次に呼ばれる譜面上の小節とTickの情報が保持される
    floating: bool,                    //   FloatingTick が有効かどうか
    normal_dist: Normal<f64>,
}
impl FloatingTick {
    pub const fn max_front_disperse() -> i32 {
        120 // Tick の前への最大散らし幅
    }
    pub const fn note_time_dispersed() -> f64 {
        0.01 // sec の散らし幅の標準偏差
    }

    pub fn new(floating: bool) -> Self {
        Self {
            just_crnt: CrntMsrTick::default(),
            next_real_tick: CrntMsrTick::set(FULL), // 大きな数値にしておく
            next_notational_tick: CrntMsrTick::set(FULL),
            floating,
            normal_dist: Normal::new(0.0, Self::note_time_dispersed()).unwrap(),
        }
    }
    /// 現在の Tick と、Notational Tick を設定する。new で生成した後に呼び出す。
    pub fn set_crnt(&mut self, crnt_: &CrntMsrTick, ntcrnt_: &CrntMsrTick) {
        self.next_real_tick = *crnt_;
        self.next_notational_tick = *ntcrnt_;
    }
    /// Floating を有効にする
    pub fn turnon_floating(&mut self) {
        self.floating = true;
    }
    /// Floating を無効にする
    pub fn turnoff_floating(&mut self) {
        self.floating = false;
    }
    pub fn _is_floating(&self) -> bool {
        self.floating
    }
    pub fn just_crnt(&self) -> &CrntMsrTick {
        &self.just_crnt
    }
    /// 現実の Tick を Notational Tick に変換する。
    pub fn convert_to_notational(&mut self, crnt_: &CrntMsrTick) -> CrntMsrTick {
        self.just_crnt = *crnt_;
        if self.next_real_tick._is_older_than(crnt_) {
            //println!(">> FloatingTick crnt_: {}/{}", crnt_.msr, crnt_.tick);
            self.next_notational_tick
        } else if self.next_real_tick._is_same_as(crnt_) {
            self.next_notational_tick
        } else {
            *crnt_
        }
    }
    /// Notational Tick を現実に鳴るべき Tick に変換する。
    pub fn convert_to_real(&mut self, next_: &CrntMsrTick) -> Option<CrntMsrTick> {
        self.next_notational_tick = *next_;
        self.next_real_tick = *next_;
        if self.floating {
            let real_tick = self.next_real_tick.tick - Self::max_front_disperse();
            if real_tick < 0 {
                self.next_real_tick.tick = self.next_real_tick.tick_for_onemsr + real_tick;
                self.next_real_tick.msr -= 1;
            } else if real_tick >= self.next_real_tick.tick_for_onemsr {
                self.next_real_tick.msr += 1;
                self.next_real_tick.tick = real_tick - self.next_real_tick.tick_for_onemsr;
            } else {
                self.next_real_tick.tick = real_tick;
            }
            //println!(">> FloatingTick next_: {}/{}", self.next_real_tick.msr, self.next_real_tick.tick);
            Some(self.next_real_tick)
        } else {
            None
        }
    }
    pub fn disperse_tick(&mut self, evt_tick: &CrntMsrTick, bpm: i16) -> (i32, i32) {
        let disperse_time = self.normal_dist.sample(&mut rand::rng());
        let tt = evt_tick.tick + (disperse_time * (bpm as f64) * (1920.0 / (60.0 * 4.0))) as i32; // 4分音符基準
        let (nmsr, ntick) = if tt < 0 {
            (evt_tick.msr - 1, evt_tick.tick_for_onemsr + tt)
        } else if tt >= evt_tick.tick_for_onemsr {
            (evt_tick.msr + 1, tt - evt_tick.tick_for_onemsr)
        } else {
            (evt_tick.msr, tt)
        };
        //println!("@@@>Note Event at {}/{}", nmsr, ntick);
        (nmsr, ntick)
    }
}
