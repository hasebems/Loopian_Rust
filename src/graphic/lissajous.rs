//  Created by Hasebe Masahiko on 2024/11/27.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::graphic::Resize;
use super::viewobj::{NormalView, NoteObj};

pub struct Lissajous {
    crnt_time: f32,
}

impl Lissajous {
    pub fn new() -> Self {
        Self { crnt_time: 0.0 }
    }
}

impl NormalView for Lissajous {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) {
        self.crnt_time = crnt_time;
    }
    fn disp(&self, draw: Draw, _tm: f32, _rs: Resize) {
        draw.ellipse().x_y(0.0, 0.0).radius(30.0).color(WHITE);
    }
}

impl NoteObj for Lissajous {
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) -> bool {
        true
    }
    fn disp(&self, _draw: Draw, _crnt_time: f32, _rs: Resize) {}
}
