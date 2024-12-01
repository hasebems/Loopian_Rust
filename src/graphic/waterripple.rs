//  Created by Hasebe Masahiko on 2023/11/03.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;
use rand::{thread_rng, Rng};

use super::draw_graph::Resize;
use super::viewobj::*;
//use crate::lpnlib::*;

pub struct WaterRipple {
    para1: f32, // 0.0 - 1.0
    para2: f32, // 0.0 - 1.0
    para3: f32,
    start_time: f32,
    mode: GraphMode,
    elapsed_time: f32,
}

impl WaterRipple {
    const SPREAD_SPEED: f32 = 50.0; // Bigger, Faster
    const DISAPPEAR_TIME: f32 = 5.0; // Bigger, Slower
    const RIPPLE_SIZE: i32 = 24; // Bigger, Thicker
    const BRIGHTNESS: f32 = 2.0; // 明るさ
    const LENGTH: f32 = 4.0; // 波の長さ 大きいほど波が短い
    const DENSITY: f32 = 2.5; // 波の密度 小さいほど波が細かい
    const RIPPLE_FSIZE: f32 = WaterRipple::RIPPLE_SIZE as f32;
    pub fn new(nt: f32, vel: f32, time: f32, mode: GraphMode) -> Self {
        Self {
            para1: nt / 128.0,
            para2: thread_rng().gen(),
            para3: (vel * vel / 16384.0), // velは小さい時に薄くするため二乗
            start_time: time,
            mode,
            elapsed_time: 0.0, // 1.0..DISAPPEAR_TIME+1.0
        }
    }
}
impl NoteObj for WaterRipple {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        self.elapsed_time = crnt_time - self.start_time;
        self.elapsed_time <= WaterRipple::DISAPPEAR_TIME
    }
    fn disp(&self, draw: Draw, _crnt_time: f32, rs: Resize) {
        const THICKNESS: f32 = 3.0;
        for i in 0..WaterRipple::RIPPLE_SIZE {
            let phase = std::f32::consts::PI * (i as f32) / WaterRipple::DENSITY; // 波の密度
            let gray = WaterRipple::BRIGHTNESS * (0.5 + phase.sin() / 2.0); // 波パターンの関数(sinの絶対値)
            let alpha_level = gray
                * self.para3
                * ((WaterRipple::RIPPLE_FSIZE - (i as f32)) / WaterRipple::RIPPLE_FSIZE)
                    .powf(WaterRipple::LENGTH)
                * ((WaterRipple::DISAPPEAR_TIME - self.elapsed_time) / WaterRipple::DISAPPEAR_TIME); // 消えゆく速さ
            let gray_scal = if self.mode == GraphMode::Dark {
                rgba(1.0, 1.0, 1.0, alpha_level)
            } else {
                rgba(0.0, 0.0, 0.0, alpha_level)
            };
            let radius_sz = self.elapsed_time * WaterRipple::SPREAD_SPEED - (i as f32) * THICKNESS;
            if radius_sz > 0.0 {
                draw.ellipse()
                    .x_y(
                        ((self.para1 - 0.5) * 2.0 * 1.4) * rs.get_full_size_x(),
                        (self.para2 - 0.5) * (rs.get_full_size_y() * 0.6),
                    )
                    .no_fill()
                    .stroke_weight(THICKNESS)
                    .stroke(gray_scal)
                    .radius(radius_sz);
            }
        }
    }
}
