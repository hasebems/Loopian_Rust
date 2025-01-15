//  Created by Hasebe Masahiko on 2025/01/07.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use super::generative_view::*;

//*******************************************************************
//      define struct BeatLissa
//*******************************************************************
pub struct BeatLissa {
    beat: i32,
    measure_position: i32,

    max_obj_inline: i32,
    max_obj: i32,
    start_x: f32,
    start_y: f32,
    obj_locate: Vec<Vec2>,
    bobj: Vec<Box<dyn BeatObj>>, // Beat Object
    mode: GraphMode,
}
//*******************************************************************
//      impl BeatLissa
//*******************************************************************
const SQUARE_SIZE: f32 = 100.0;
const GAP: f32 = 180.0;
const MAX_LINE: i32 = 2;
const START_Y: f32 = 150.0;

impl BeatLissa {
    pub fn new(_tm: f32, mode: GraphMode) -> Self {
        Self {
            beat: 0,
            measure_position: 0,
            max_obj_inline: 0, // 1行に表示するオブジェクト数
            max_obj: 0,        // 画面全体のオブジェクト数
            start_x: 0.0,
            start_y: 0.0,
            obj_locate: Vec::new(),
            bobj: Vec::new(),
            mode,
        }
    }
    /// beatlissa にビート数を設定
    /// 現在、生成時に設定されるのみで、拍子が変更された時に更新されない
    pub fn set_beat_inmsr(&mut self, beat_inmsr: i32) {
        self.max_obj_inline = beat_inmsr;
        self.max_obj = self.max_obj_inline * MAX_LINE;
        self.start_x = -(self.max_obj_inline as f32 * SQUARE_SIZE
            + (self.max_obj_inline - 1) as f32 * GAP)
            / 2.0
            + SQUARE_SIZE / 2.0;
        self.start_y = START_Y;
        for i in 0..self.max_obj {
            let xnum = i % self.max_obj_inline;
            let ynum = (i / self.max_obj_inline) as f32;
            let x = (SQUARE_SIZE + GAP) * xnum as f32 + self.start_x;
            let y = self.start_y - ynum * (SQUARE_SIZE + GAP);
            self.obj_locate.push(Vec2::new(x, y));
        }
        println!(
            "QQQ::max_obj_inline:{}, x:{}",
            self.max_obj_inline, self.start_x
        );
    }
}
//*******************************************************************
impl GenerativeView for BeatLissa {
    /// 画面全体の Model の更新
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        // Beat Object の更新と削除
        let mut retain: Vec<bool> = Vec::new();
        for obj in self.bobj.iter_mut() {
            retain.push(obj.update_model(crnt_time, rs.clone()));
        }
        for (j, rt) in retain.iter().enumerate() {
            if !rt {
                self.bobj.remove(j);
                break;
            }
        }
    }
    /// Beat 演奏情報を受け取る
    fn on_beat(&mut self, bt: i32, tm: f32, dt: f32) {
        self.beat = bt;
        if bt == 0 {
            self.measure_position += 1;
        }
        let max_obj =
            ((self.measure_position * self.max_obj_inline + self.beat) % self.max_obj + 1) as usize;

        if max_obj <= 1 && self.bobj.len() > max_obj {
            self.bobj.clear();
        }
        if self.bobj.len() < max_obj {
            let loc = self.obj_locate[self.bobj.len()];
            self.bobj
                .push(Box::new(BeatLissaObj::new(tm, dt, loc.x, loc.y, self.mode)));
        }
    }
    /// Mode 情報を受け取る
    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }
    /// 画面全体の描画
    fn disp(
        &self,
        draw: Draw,
        tm: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    ) {
        //  Beat Object の描画
        for obj in self.bobj.iter() {
            obj.disp(draw.clone(), tm, rs.clone());
        }
    }
}
//*******************************************************************
//      define struct  BeatObj
//*******************************************************************
pub struct BeatLissaObj {
    phase: f32,
    first_time: f32,
    last_time: f32,
    draw_time: f32,
    polyline: Vec<Vec2>,
    center: Vec2,
    mode: GraphMode,
}
//*******************************************************************
//      impl  BeatObj
//*******************************************************************
impl BeatLissaObj {
    pub fn new(crnt_time: f32, draw_time: f32, x: f32, y: f32, mode: GraphMode) -> Self {
        Self {
            phase: crnt_time * 2.0,
            first_time: crnt_time,
            last_time: 0.0,
            draw_time,
            polyline: Vec::new(),
            center: Vec2::new(x, y),
            mode,
        }
    }
}
//*******************************************************************
const DRAW_SPEED: f32 = 12.0;
impl BeatObj for BeatLissaObj {
    /// Beat の Model の更新
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        if crnt_time - self.last_time > 0.01 && crnt_time - self.first_time < self.draw_time {
            // 10msec ごとに更新、draw_time で終了
            let new_phase = crnt_time * DRAW_SPEED;
            let mut position: Vec2 = Default::default();
            position.x = ((new_phase + 1.4) * 0.78).sin() * SQUARE_SIZE + self.center.x;
            position.y = ((new_phase + 0.6) * 1.35).sin() * SQUARE_SIZE + self.center.y;
            self.polyline.push(position);
            self.phase = new_phase;
            self.last_time = crnt_time;
        }
        true
    }
    /// Beat の描画
    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        _rs: Resize,     //  ウィンドウサイズ
    ) {
        let num = self.polyline.len();
        if num < 2 {
            return;
        }
        let color = if self.mode == GraphMode::Dark {
            WHITE
        } else {
            BLACK
        };
        draw.line()
            .start(self.polyline[1])
            .end(self.polyline[0])
            .weight(3.0)
            .color(MAGENTA);
        for i in 1..num - 1 {
            draw.line()
                .start(self.polyline[i + 1])
                .end(self.polyline[i])
                .weight(3.0)
                .color(color);
        }
    }
}
