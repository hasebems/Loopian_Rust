//  Created by Hasebe Masahiko on 2024/11/27.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::viewobj::{NormalView, NoteObj};
use super::graphic::Resize;

pub struct Lissajous {

}

impl Lissajous {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl NormalView for Lissajous {
    fn disp(&self, draw: Draw, _tm: f32, _rs: Resize) {
        draw.ellipse()
            .x_y(0.0, 0.0)
            .radius(30.0)
            .color(WHITE);
    }
}

impl NoteObj for Lissajous {
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) -> bool {
        true
    }
    fn disp(&self, _draw: Draw, _crnt_time: f32, _rs: Resize) {
    }
}