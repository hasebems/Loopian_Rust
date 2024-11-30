//  Created by Hasebe Masahiko on 2024/11/27.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use super::viewobj::NormalView;

pub struct Lissajous {
    crnt_time: f32,
    track: Vec<Vec2>,
    range: f32,
    phase_real: f32,
    phase_target: f32,
}

impl Lissajous {
    const SPEED: f32 = 8.0;
    pub fn new() -> Self {
        Self {
            crnt_time: 0.0,
            track: Vec::new(),
            range: 0.0,
            phase_real: 0.0,
            phase_target: 0.0,
        }
    }
}

impl NormalView for Lissajous {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) {
        let past_time = self.crnt_time;
        self.crnt_time = crnt_time*Lissajous::SPEED;
        let x1 = (past_time*1.0+self.phase_real).sin()*(1.0+self.range)*100.0;
        let y1 = (past_time*2.0).sin()*(1.0+self.range)*100.0;
        let v = Vec2::new(x1, y1);
        self.track.push(v);
        if self.track.len() > 50 {
            self.track.remove(0);
        }
        self.range *= 0.95;
        self.phase_real += (self.phase_target - self.phase_real)*0.1;
    }
    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, _tm: f32) {
        self.range += vel as f32 / 127.0;
        if self.range > 1.0 {
            self.range = 1.0;
        }
        self.phase_target += nt as f32 / 127.0;
    }
    fn disp(&self, draw: Draw, _tm: f32, _rs: Resize) {
        let num = self.track.len();
        for i in 0..num-1 {
            let stg: f32 = ((i+1) as f32)/(num as f32);
            draw.line()
                .start(self.track[i+1])
                .end(self.track[i])
                .weight(5.0)
                .color(rgb(stg,stg,stg));
        }
    }
}