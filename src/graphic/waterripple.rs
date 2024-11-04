//  Created by Hasebe Masahiko on 2023/11/03.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::{prelude::*, wgpu::Color};
use rand::{rngs, thread_rng, Rng};
//use eframe::egui::*;

use crate::lpnlib::*;
use crate::Resize;
use super::noteobj::NoteObj;

pub struct WaterRipple {
    para1: f32, // 0.0 - 1.0
    para2: f32, // 0.0 - 1.0
    para3: f32,
    time: f32,
    mode: GraphMode,
    cnt: f32,
}

impl WaterRipple {
    const SPREAD_SPEED: f32 = 50.0; // Bigger, Faster
    const DISAPPEAR_TIME: f32 = 5.0; // Bigger, Slower
    const RIPPLE_SIZE: i32 = 48; // Bigger, Thicker
    const BRIGHTNESS: f32 = 4.0; // 明るさ
    const RIPPLE_SIZE_F: f32 = (WaterRipple::RIPPLE_SIZE - 1) as f32;
    pub fn new(nt: f32, vel: f32, time: f32/* , mode: GraphMode*/) -> Self {
        Self {
            para1: nt / 128.0,
            para2: thread_rng().gen(),
            para3: (vel * vel / 16384.0), // velは小さい時に薄くするため二乗
            time,
            mode: GraphMode::Dark,
            cnt: 1.0, // 1.0..DISAPPEAR_TIME+1.0
        }
    }
    /*fn gray(&self, gray_scl: u8) -> Color {
        if self.mode == GraphMode::Light {
            GRAY//Color32::from_black_alpha(gray_scl)
        } else if self.mode == GraphMode::Dark {
            GRAY//Color32::from_white_alpha(gray_scl)
        } else {
            GRAY//Color32::from_white_alpha(gray_scl)
        }
    }*/
}
impl NoteObj for WaterRipple {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool {
        self.cnt = crnt_time - self.time + 1.0;
        if self.cnt > WaterRipple::DISAPPEAR_TIME+1.0 {
            false
        } else {
            true
        }
    }
    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        const THICKNESS: f32 = 2.0;
        for i in 0..WaterRipple::RIPPLE_SIZE {
            let phase = std::f32::consts::PI * (i as f32) / 5.0; // 波の密度
            let gray = WaterRipple::BRIGHTNESS * (0.5 + phase.sin()/2.0); // 波パターンの関数(sinの絶対値)
            //let gray = 1.0 - phase.sin().abs();
            let alpha_level = 
                gray*self.para3*
                //((WaterRipple::RIPPLE_SIZE_F-(i as f32))/WaterRipple::RIPPLE_SIZE_F)*     // 厚さと濃淡
                ((WaterRipple::DISAPPEAR_TIME+1.0-self.cnt)/WaterRipple::DISAPPEAR_TIME);
                // 消えゆく速さ // 白/Alpha値への変換
            let gray_scal = rgba(1.0, 1.0, 1.0, alpha_level);
            if i < self.cnt as i32 {
                draw.ellipse()
                    .x_y( 
                        ((self.para1 - 0.5) * 2.0 * 1.4) * rs.full_size_x,
                        (self.para2 - 0.5) * (rs.full_size_y * 0.6) - (rs.full_size_y * 0.2),
                    )
                    .no_fill()
                    .stroke_weight(1.0)
                    .stroke(gray_scal)
                    .radius(self.cnt * WaterRipple::SPREAD_SPEED - (i as f32)*THICKNESS);
            }
        }
    }
}
