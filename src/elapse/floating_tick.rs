//  Created by Hasebe Masahiko on 2025/06/21
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use super::tickgen::CrntMsrTick;

/// FloatingTick
/// 本クラスは、PhraseLoop の next_tick を入力すると、その値を散らして
/// より音楽的な Tick を生成するクラスである。
/// 以下の二つの処理を行う。
/// 1. 同時発音する和音の Tick をずらす。
/// 2. 通常発音の Tick にランダム性を持たせる。
pub struct FloatingTick {
    last_real_crnt: CrntMsrTick, //   次に呼ばれる小節とTickの情報が保持される
    last_notational_crnt: CrntMsrTick, //   次に呼ばれる小節とTickの情報が保持される
}
impl FloatingTick {
    const TICK_DISPERSE: i32 = 20; // Tick の散らし幅

    pub fn new() -> Self {
        Self {
            last_real_crnt: CrntMsrTick::new(),
            last_notational_crnt: CrntMsrTick::new(),
        }
    }
    pub fn convert_to_notational(&mut self, crnt_: &CrntMsrTick) -> CrntMsrTick {
        if self.last_real_crnt == *crnt_ {
            self.last_notational_crnt
        } else {
            *crnt_
        }
    }
    pub fn convert_to_real(&mut self, crnt_: &CrntMsrTick) -> CrntMsrTick {
        self.last_notational_crnt = *crnt_;
        self.last_real_crnt = *crnt_;
        if Self::TICK_DISPERSE < self.last_real_crnt.tick {
            self.last_real_crnt.tick -= Self::TICK_DISPERSE;
        } else {
            self.last_real_crnt.tick = crnt_.tick_for_onemsr - Self::TICK_DISPERSE;
            self.last_real_crnt.msr -= 1;
        }
        self.last_real_crnt
    }
}
