//  Created by Hasebe Masahiko on 2023/11/03.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use eframe::egui::*;
//use rand::{thread_rng, Rng, rngs};

pub struct WaterRipple {
    para1: f32,
    para2: f32,
    para3: f32,
    time: i32,
}

impl WaterRipple {
    const DISAPPEAR_RATE: f32 = 300.0;
    const RIPPLE_SIZE: i32 = 32;
    const BRIGHTNESS: f32 = 255.0;  // Max 255
    const RIPPLE_SIZE_F: f32 = (WaterRipple::RIPPLE_SIZE-1) as f32;
    pub fn new(nt: f32, vel: f32, tm: i32) -> Self {
        Self {
            para1: nt, para2: 50.0, para3: vel, time: tm,
        }
    }
    pub fn disp(&self, crnt_time: i32, ui: &mut Ui) -> bool {
        let cnt = (crnt_time - self.time)*4;
        if cnt as f32 > WaterRipple::DISAPPEAR_RATE {return false;}
        for i in 0..WaterRipple::RIPPLE_SIZE {
            let phase = std::f32::consts::PI*(i as f32)/16.0;  // 波の密度
            let gray = WaterRipple::BRIGHTNESS*(1.0-phase.sin().abs());          // 波パターンの関数(sinの絶対値)
            let gray_scl = (gray*
                (self.para3/100.0)*
                ((WaterRipple::RIPPLE_SIZE_F-(i as f32))/WaterRipple::RIPPLE_SIZE_F)*     // 厚さと濃淡
                ((WaterRipple::DISAPPEAR_RATE-(cnt as f32))/WaterRipple::DISAPPEAR_RATE)  // 消えゆく速さ
            ) as u8;  // 白/Alpha値への変換
            if i < cnt {
                ui.painter().circle_stroke(
                    Pos2 {x:self.para1*5.0, y:self.para2*5.0},  // location
                    (cnt-i) as f32,                                   // radius
                    Stroke {width:1.0, color:Color32::from_white_alpha(gray_scl)}
                );
            }
        }
        true
    }
}