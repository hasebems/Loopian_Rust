//  Created by Hasebe Masahiko on 2025/01/07.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::viewobj::*;
use super::draw_graph::Resize;

pub struct BeatView {
    beat: i32,
}

impl BeatView {
    pub fn new(_tm: f32, _mode: GraphMode) -> Self {
        Self {
            beat: 0,
        }
    }
}
impl NormalView for BeatView {
    /// 画面全体の Model の更新
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) {}
    /// Note 演奏情報を受け取る
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {}
    /// Beat 演奏情報を受け取る
    fn on_beat(&mut self, bt: i32, _tm: f32) {
        //println!("XXXX:{},{}",bt, tm);
        self.beat = bt;
    }
    /// Mode 情報を受け取る
    fn set_mode(&mut self, _mode: GraphMode) {}
    /// 画面全体の描画
    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        _rs: Resize,
    ) {
        draw.rect()
            .x_y(100.0*self.beat as f32 - 150.0, 0.0)
            .w_h(50.0, 50.0)
            .color(WHITE);
    }
}