//  Created by Hasebe Masahiko on 2023/11/03.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use super::generative_view::*;
//use crate::lpnlib::*;

pub struct WaterRipple {
    mode: GraphMode,
    nobj: Vec<Box<dyn NoteObj>>, // Note Object
}

impl WaterRipple {
    pub fn new(mode: GraphMode) -> Self {
        Self {
            mode,
            nobj: Vec::new(),
        }
    }
}

impl GenerativeView for WaterRipple {
    /// 画面全体の Model の更新
    fn update_model(&mut self, tm: f32, rs: Resize) {
        // Note Object の更新と削除
        let mut retain: Vec<bool> = Vec::new();
        for obj in self.nobj.iter_mut() {
            retain.push(obj.update_model(tm, rs.clone()));
        }
        for (j, rt) in retain.iter().enumerate() {
            if !rt {
                self.nobj.remove(j);
                break;
            }
        }
    }
    /// Note 演奏情報を受け取る
    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, tm: f32) {
        self.nobj.push(Box::new(WaterRippleNote::new(
            nt as f32, vel as f32, tm, self.mode,
        )));
    }
    /// Mode 情報を受け取る
    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }
    /// 画面全体の描画
    fn disp(&self, draw: Draw, tm: f32, rs: Resize) {
        //  Note Object の描画
        for obj in self.nobj.iter() {
            obj.disp(draw.clone(), tm, rs.clone());
        }
    }
}

pub struct WaterRippleNote {
    para1: f32, // 0.0 - 1.0
    para2: f32, // 0.0 - 1.0
    para3: f32,
    start_time: f32,
    mode: GraphMode,
    elapsed_time: f32,
}

impl WaterRippleNote {
    const SPREAD_SPEED: f32 = 50.0; // Bigger, Faster
    const DISAPPEAR_TIME: f32 = 5.0; // Bigger, Slower
    const RIPPLE_SIZE: i32 = 24; // Bigger, Thicker
    const BRIGHTNESS: f32 = 2.0; // 明るさ
    const LENGTH: f32 = 4.0; // 波の長さ 大きいほど波が短い
    const DENSITY: f32 = 2.5; // 波の密度 小さいほど波が細かい
    const RIPPLE_FSIZE: f32 = WaterRippleNote::RIPPLE_SIZE as f32;
    pub fn new(nt: f32, vel: f32, time: f32, mode: GraphMode) -> Self {
        Self {
            para1: nt / 128.0,
            para2: random(),
            para3: (vel * vel / 16384.0), // velは小さい時に薄くするため二乗
            start_time: time,
            mode,
            elapsed_time: 0.0, // 1.0..DISAPPEAR_TIME+1.0
        }
    }
}
impl NoteObj for WaterRippleNote {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        self.elapsed_time = crnt_time - self.start_time;
        self.elapsed_time <= Self::DISAPPEAR_TIME
    }
    fn disp(&self, draw: Draw, _crnt_time: f32, rs: Resize) {
        const THICKNESS: f32 = 3.0;
        for i in 0..Self::RIPPLE_SIZE {
            let phase = std::f32::consts::PI * (i as f32) / Self::DENSITY; // 波の密度
            let gray = Self::BRIGHTNESS * (0.5 + phase.sin() / 2.0); // 波パターンの関数(sinの絶対値)
            let alpha_level = gray
                * self.para3
                * ((Self::RIPPLE_FSIZE - (i as f32)) / Self::RIPPLE_FSIZE).powf(Self::LENGTH)
                * ((Self::DISAPPEAR_TIME - self.elapsed_time) / Self::DISAPPEAR_TIME); // 消えゆく速さ
            let gray_scal = if self.mode == GraphMode::Dark {
                rgba(1.0, 1.0, 1.0, alpha_level)
            } else {
                rgba(0.0, 0.0, 0.0, alpha_level)
            };
            let radius_sz = self.elapsed_time * Self::SPREAD_SPEED - (i as f32) * THICKNESS;
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
