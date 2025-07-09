//  Created by Hasebe Masahiko on 2025/06/21
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::tickgen::CrntMsrTick;
use crate::lpnlib::*;

/// FloatingTick
/// 本クラスは、PhraseLoop の next_tick を入力すると、その値を時間方向に散らして
/// より生演奏に近い Tick を自動生成するクラスである。
/// 時間方向に散らすことを FloatingTick と呼ぶこととする。
/// 以下の二つの処理を行う。
/// 1. 同時発音する和音の Tick をずらす。
/// 2. 通常発音の Tick にランダム性を持たせる。
pub struct FloatingTick {
    just_crnt: CrntMsrTick,            //   現在の小節とTickの情報が保持される
    next_real_tick: CrntMsrTick,       //   次に呼ばれる小節とTickの情報が保持される
    next_notational_tick: CrntMsrTick, //   次に呼ばれる譜面上の小節とTickの情報が保持される
    floating: bool,                    //   FloatingTick が有効かどうか
    max_count: Option<i32>,            //   Tick を散らす回数
    disperse_count: i32,               //   Tick を散らす回数
}
impl FloatingTick {
    const MAX_FRONT_DISPERSE: i32 = 50; // Tick の前への散らし幅
    const EACH_DISPERSE: i32 = 50; // Tick の散らし幅の単位

    pub fn new(floating: bool) -> Self {
        Self {
            just_crnt: CrntMsrTick::default(),
            next_real_tick: CrntMsrTick::set(FULL), // 大きな数値にしておく
            next_notational_tick: CrntMsrTick::set(FULL),
            floating,
            max_count: None,
            disperse_count: 0,
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
    /// FloatingTick を使用して、同音発音を散らす指示を受ける
    /// max_num: 同音発音数
    /// start: 散らし開始位置
    pub fn set_disperse_count(&mut self, max_num: i32, start: i32) {
        self.max_count = Some(max_num);
        self.disperse_count = start;
    }
    /// 現実の Tick を Notational Tick に変換する。
    pub fn convert_to_notational(&mut self, crnt_: &CrntMsrTick) -> CrntMsrTick {
        self.just_crnt = *crnt_;
        if self.next_real_tick._is_older_than(crnt_) {
            println!(">> FloatingTick: next_real_tick: {:?}", self.next_real_tick);
            self.next_notational_tick
        } else if self.next_real_tick._is_same_as(crnt_) {
            self.next_notational_tick
        } else {
            *crnt_
        }
    }
    /// Notational Tick を現実に鳴るべき Tick に変換する。
    pub fn convert_to_real(&mut self, crnt_: &CrntMsrTick) -> CrntMsrTick {
        self.next_notational_tick = *crnt_;
        self.next_real_tick = *crnt_;
        if self.floating {
            // 時間の前方向への散らし幅
            let disperse_size = if let Some(max) = self.max_count {
                if self.disperse_count < max {
                    let count = self.disperse_count;
                    self.disperse_count += 1;
                    Self::MAX_FRONT_DISPERSE - (count * Self::EACH_DISPERSE)
                } else {
                    self.max_count = None;
                    self.disperse_count = 0;
                    Self::MAX_FRONT_DISPERSE
                }
            } else {
                self.disperse_count = 0;
                Self::MAX_FRONT_DISPERSE
            };
            let real_tick = self.next_real_tick.tick - disperse_size;
            if real_tick < 0 {
                self.next_real_tick.tick = self.next_real_tick.tick_for_onemsr + real_tick;
                self.next_real_tick.msr -= 1;
            } else if real_tick >= self.next_real_tick.tick_for_onemsr {
                self.next_real_tick.msr += 1;
                self.next_real_tick.tick = real_tick - self.next_real_tick.tick_for_onemsr;
            } else {
                self.next_real_tick.tick = real_tick;
            }
        }
        self.next_real_tick
    }
}
