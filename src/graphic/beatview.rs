//  Created by Hasebe Masahiko on 2025/01/07.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use super::viewobj::*;

//*******************************************************************
//      define struct BeatView
//*******************************************************************
pub struct BeatView {
    beat: i32,
    measure_position: i32,
    numerator: i32,
    max_obj_inline: i32,
    max_msr_inline: i32,
    start_x: f32,
    start_y: f32,
    _mode: GraphMode,
}
//*******************************************************************
//      impl BeatView
//*******************************************************************
const SQUARE_SIZE: f32 = 100.0;
const GAP: f32 = 50.0;

impl BeatView {
    pub fn new(_tm: f32, _mode: GraphMode) -> Self {
        Self {
            beat: 0,
            measure_position: 0,
            numerator: 0,      // 拍子の分子
            max_obj_inline: 0, // 1行に表示するオブジェクト数
            max_msr_inline: 0, // 1行に表示する小節数
            start_x: 0.0,
            start_y: 0.0,
            _mode,
        }
    }
    pub fn set_beat_inmsr(&mut self, beat_inmsr: i32) {
        self.numerator = beat_inmsr;
        if beat_inmsr <= 4 {
            self.max_obj_inline = beat_inmsr * 2;
            self.max_msr_inline = 2;
        } else {
            self.max_obj_inline = beat_inmsr;
            self.max_msr_inline = 1;
        };
        self.start_x = -(self.max_obj_inline as f32 * SQUARE_SIZE
            + (self.max_obj_inline - 1) as f32 * GAP)
            / 2.0
            + SQUARE_SIZE / 2.0;
        self.start_y = 250.0;
        println!(
            "QQQ::max_obj_inline:{}, x:{}",
            self.max_obj_inline, self.start_x
        );
    }
}
//*******************************************************************
//      impl BeatView as NormalView
//*******************************************************************
impl NormalView for BeatView {
    /// 画面全体の Model の更新
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) {}
    /// Note 演奏情報を受け取る
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {}
    /// Beat 演奏情報を受け取る
    fn on_beat(&mut self, bt: i32, _tm: f32) {
        //println!("XXXX:{},{}",bt, tm);
        self.beat = bt;
        if bt == 0 {
            self.measure_position += 1;
        }
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
        let max_obj = self.max_obj_inline * 4;
        let disp_obj = (self.measure_position * self.numerator + self.beat) % max_obj;
        for i in 0..max_obj {
            let xnum = i % self.max_obj_inline;
            let ynum = (i / self.max_obj_inline) as f32;
            let x = (SQUARE_SIZE + GAP) * xnum as f32 + self.start_x;
            let y = self.start_y - ynum * (SQUARE_SIZE + GAP);
            if i <= disp_obj {
                draw.rect()
                    .x_y(x, y)
                    .w_h(SQUARE_SIZE, SQUARE_SIZE)
                    .color(GRAY);
            } else {
                draw.rect()
                    .x_y(x, y)
                    .w_h(SQUARE_SIZE, SQUARE_SIZE)
                    .no_fill()
                    .stroke_weight(2.0)
                    .stroke(GRAY);
            }
        }
    }
}
