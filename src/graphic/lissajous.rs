//  Created by Hasebe Masahiko on 2024/11/27.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;
use std::f32::consts::PI;

use super::draw_graph::Resize;
use super::viewobj::*;
use crate::lpnlib::*;

pub struct Lissajous {
    crnt_time: f32,
    mode: GraphMode,
    track: Vec<[Vec2; 2]>,
    range_real: f32,
    range_target: f32,
    phase_real: f32,
    phase_target: f32,
}

impl Lissajous {
    const SPEED: f32 = 0.5;
    const MAX_TRACK: usize = 50;
    const X_MAX: f32 = 200.0;
    const Y_MAX: f32 = 150.0;
    pub fn new(mode: GraphMode) -> Self {
        Self {
            crnt_time: 0.0,
            mode,
            track: Vec::new(),
            range_real: 1.0,
            range_target: 1.0,
            phase_real: 0.0,
            phase_target: 0.0,
        }
    }
}

impl NormalView for Lissajous {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) {
        let past_time = self.crnt_time;
        self.crnt_time = crnt_time * Lissajous::SPEED;
        let x1 = (past_time * 1.0 + self.phase_real).sin() * self.range_real * Lissajous::X_MAX;
        let y1 = (past_time * 2.0).sin() * self.range_real * Lissajous::Y_MAX;
        let x2 = (past_time * 2.5 + self.phase_real + PI / 1.5).sin()
            * self.range_real
            * Lissajous::X_MAX;
        let y2 = (past_time * 1.5 - PI / 2.5).sin() * self.range_real * Lissajous::Y_MAX;
        let v1 = Vec2::new(x1, y1);
        let v2 = Vec2::new(x2, y2);
        self.track.push([v1, v2]);
        if self.track.len() > Lissajous::MAX_TRACK {
            self.track.remove(0);
        }
        // range, phase の補間
        self.range_target *= 0.98;
        if self.range_real < self.range_target {
            self.range_real += (self.range_target - self.range_real) * 0.5;
        } else if self.range_real > self.range_target {
            self.range_real -= (self.range_real - self.range_target) * 0.5;
        }
        if self.range_real < 1.0 {
            self.range_real = 1.0;
        }
        self.phase_real += (self.phase_target - self.phase_real) * 0.01;
    }
    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, _tm: f32) {
        self.range_target += vel as f32 / 127.0;
        if self.range_target > 3.0 {
            self.range_target = 3.0;
        }
        // どの音程でも一定程度の位相差を持たせる
        let pnt = (nt as u8).clamp(MIN_NOTE_NUMBER, MAX_NOTE_NUMBER) as f32;
        if pnt > 60.0 {
            // C4 以上 -> 68..100
            self.phase_target += PI * (pnt - (MAX_NOTE_NUMBER as f32 - 100.0)) / 100.0;
        } else {
            // C4 未満 -> -61..-100
            self.phase_target += PI * (pnt - (MIN_NOTE_NUMBER as f32 + 100.0)) / 100.0;
        }
    }
    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }
    fn disp(&self, draw: Draw, _tm: f32, _rs: Resize) {
        let num = self.track.len();
        let light = self.mode == GraphMode::Light;
        for i in 0..num - 1 {
            let mut stg: f32 = ((i + 1) as f32) / (num as f32);
            if light {
                stg = 1.0 - stg;
            }
            draw.line()
                .start(self.track[i + 1][0])
                .end(self.track[i][1])
                .weight(2.0)
                .color(rgb(stg, stg, stg));
        }
    }
}
