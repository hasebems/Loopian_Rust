//  Created by Hasebe Masahiko on 2025/04/05.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::graphic::draw_graph::*;
use crate::graphic::generative_view::{GenerativeView, NoteObj};
use crate::lpnlib::GraphMode;
use nannou::prelude::*;

//*******************************************************************
//      Screen Graphic
//*******************************************************************
pub struct SineWave {}
impl SineWave {
    pub fn new(_mode: GraphMode) -> Self {
        Self {}
    }
}
impl GenerativeView for SineWave {
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) {}
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {}
    fn set_mode(&mut self, _mode: GraphMode) {}
    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        _rs: Resize,
    ) {
        // 原点
        draw.line()
            .start(pt2(100.0, 0.0))
            .end(pt2(-100.0, 0.0))
            .color(RED);
        draw.line()
            .start(pt2(0.0, 100.0))
            .end(pt2(0.0, -100.0))
            .color(RED);
    }
}

//*******************************************************************
//      Note Graphic
//*******************************************************************
pub struct SineWaveNote {}
impl SineWaveNote {
    pub fn _new() -> Self {
        Self {}
    }
}
impl NoteObj for SineWaveNote {
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) -> bool {
        false
    }
    fn disp(
        &self,
        _draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        _rs: Resize,     //  ウィンドウサイズ
    ) {
    }
}
