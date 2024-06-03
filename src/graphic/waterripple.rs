//  Created by Hasebe Masahiko on 2023/11/03.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use eframe::egui::*;

use super::noteobj::NoteObj;

pub struct WaterRipple {
    para1: f32, // 0.0 - 1.0
    para2: f32, // 0.0 - 1.0
    para3: f32,
    time: i32,
    mode: GraphMode,
}

impl WaterRipple {
    const DISAPPEAR_RATE: f32 = 500.0; // Bigger, Slower
    const RIPPLE_SIZE: i32 = 48; // Bigger, Thicker
    const BRIGHTNESS: f32 = 500.0; // 明るさ
    const RIPPLE_SIZE_F: f32 = (WaterRipple::RIPPLE_SIZE - 1) as f32;
    pub fn new(nt: f32, vel: f32, rnd: f32, tm: i32, mode: GraphMode) -> Self {
        Self {
            para1: nt / 128.0,
            para2: rnd,
            para3: (vel * vel / 16384.0), // velは小さい時に薄くするため二乗
            time: tm,
            mode,
        }
    }
    fn gray(&self, gray_scl: u8) -> Color32 {
        if self.mode == GraphMode::Light {
            Color32::from_black_alpha(gray_scl)
        } else if self.mode == GraphMode::Dark {
            Color32::from_white_alpha(gray_scl)
        } else {
            Color32::from_white_alpha(gray_scl)
        }
    }
}
impl NoteObj for WaterRipple {
    fn disp(&self, crnt_time: i32, ui: &mut Ui, fsz: Pos2) -> bool {
        let cnt = (crnt_time - self.time) * 4;
        if cnt as f32 > WaterRipple::DISAPPEAR_RATE {
            return false;
        }
        for i in 0..WaterRipple::RIPPLE_SIZE {
            let phase = std::f32::consts::PI * (i as f32) / 16.0; // 波の密度
            let gray = WaterRipple::BRIGHTNESS * (1.0 - phase.sin().abs()); // 波パターンの関数(sinの絶対値)
            let gray_scl = (
                gray*self.para3*
                ((WaterRipple::RIPPLE_SIZE_F-(i as f32))/WaterRipple::RIPPLE_SIZE_F)*     // 厚さと濃淡
                ((WaterRipple::DISAPPEAR_RATE-(cnt as f32))/WaterRipple::DISAPPEAR_RATE)
                // 消えゆく速さ
            ) as u8; // 白/Alpha値への変換
            if i < cnt {
                ui.painter().circle_stroke(
                    Pos2 {
                        x: ((self.para1 - 0.5) * 1.4 + 0.5) * fsz.x,
                        y: self.para2 * (fsz.y * 0.6) + (fsz.y * 0.2),
                    }, // location
                    (cnt - i) as f32, // radius
                    Stroke {
                        width: 1.0,
                        color: self.gray(gray_scl),
                    },
                );
            }
        }
        true
    }
}
