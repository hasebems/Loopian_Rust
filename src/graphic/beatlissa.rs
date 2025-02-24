//  Created by Hasebe Masahiko on 2025/01/07.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use super::generative_view::*;

//*******************************************************************
//      struct BeatLissa
//*******************************************************************
pub struct BeatLissa {
    beat: i32,
    measure_position: i32,
    max_obj_inline: i32,
    max_obj: i32,
    obj_locate: Vec<Vec2>,       // Vec2: 位置
    bobj: Vec<Box<dyn BeatObj>>, // Beat Object
    mode: GraphMode,
}
//*******************************************************************
const SQUARE_BIG_SIZE: f32 = 230.0;
const SQUARE_SIZE: f32 = 100.0;
const GAP: f32 = 180.0;
const MAX_LINE: i32 = 2;
const START_Y: f32 = 150.0;
impl BeatLissa {
    pub fn new(beat_inmsr: i32, _tm: f32, blmd: i32, mode: GraphMode) -> Self {
        //self.max_obj_inline = beat_inmsr;
        let mut obj_locate: Vec<Vec2> = Vec::new();
        let max_obj: i32;
        if blmd == 0 {
            max_obj = 1;
        } else {
            max_obj = beat_inmsr * MAX_LINE;
            let start_x = -(beat_inmsr as f32 * SQUARE_SIZE + (beat_inmsr - 1) as f32 * GAP) / 2.0
                + SQUARE_SIZE / 2.0;
            let start_y = START_Y;

            for i in 0..max_obj {
                let xnum = i % beat_inmsr;
                let ynum = (i / beat_inmsr) as f32;
                let x = (SQUARE_SIZE + GAP) * xnum as f32 + start_x;
                let y = start_y - ynum * (SQUARE_SIZE + GAP);
                obj_locate.push(Vec2::new(x, y));
            }
        }
        Self {
            beat: 0,
            measure_position: 0,
            max_obj_inline: beat_inmsr, // 1行に表示するオブジェクト数
            max_obj,                    // 画面全体のオブジェクト数
            obj_locate,
            bobj: Vec::new(),
            mode,
        }
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
        if !self.obj_locate.is_empty() {
            let max_obj = ((self.measure_position * self.max_obj_inline + self.beat) % self.max_obj
                + 1) as usize;

            if max_obj <= 1 && self.bobj.len() > max_obj {
                self.bobj.clear();
            }
            if self.bobj.len() < max_obj {
                let loc = self.obj_locate[self.bobj.len()];
                self.bobj.push(Box::new(BeatLissaObj::new(
                    tm,
                    dt,
                    loc.x,
                    loc.y,
                    SQUARE_SIZE,
                    self.mode,
                )));
            }
        } else if bt == 0 {
            if !self.bobj.is_empty() {
                self.bobj.clear();
            };
            self.bobj.push(Box::new(BeatLissaObj::new(
                tm,
                dt * self.max_obj_inline as f32,
                0.0,
                0.0,
                SQUARE_BIG_SIZE,
                self.mode,
            )));
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
//      struct  BeatObj
//*******************************************************************
pub struct BeatLissaObj {
    phase: f32,
    first_time: f32,
    last_time: f32,
    draw_time: f32,
    polyline: Vec<Vec2>,
    center: Vec2,
    size: f32,
    thickness: f32,
    mode: GraphMode,

    phase_x: f32,
    phase_y: f32,
    ratio_x: f32,
    ratio_y: f32,
}
//*******************************************************************
const DRAW_SPEED: f32 = 12.0;
//const X_PHASE: f32 = 1.4;
//const Y_PHASE: f32 = 0.6;
//const X_RATIO: f32 = 0.78;
//const Y_RATIO: f32 = 1.35;
//*******************************************************************
impl BeatLissaObj {
    pub fn new(
        crnt_time: f32,
        draw_time: f32,
        center_x: f32,
        center_y: f32,
        size: f32,
        mode: GraphMode,
    ) -> Self {
        let thickness = if size == SQUARE_BIG_SIZE { 5.0 } else { 4.0 };
        Self {
            phase: crnt_time * DRAW_SPEED,
            first_time: crnt_time,
            last_time: 0.0,
            draw_time,
            polyline: Vec::new(),
            center: Vec2::new(center_x, center_y),
            size,
            thickness,
            mode,
            phase_x: random::<f32>() + 0.5,
            phase_y: random::<f32>() + 0.5,
            ratio_x: random::<f32>() + 0.5,
            ratio_y: random::<f32>() + 0.5,
        }
    }
}
impl BeatObj for BeatLissaObj {
    /// Beat の Model の更新
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        if crnt_time - self.last_time > 0.01 && crnt_time - self.first_time < self.draw_time {
            // 10msec ごとに更新、draw_time で終了
            let new_phase = crnt_time * DRAW_SPEED;
            let mut fine_phase = self.phase;
            while fine_phase < new_phase {
                let mut position: Vec2 = Default::default();
                position.x =
                    ((fine_phase + self.phase_x) * self.ratio_x).sin() * self.size + self.center.x;
                position.y =
                    ((fine_phase + self.phase_y) * self.ratio_y).sin() * self.size + self.center.y;
                self.polyline.push(position);
                fine_phase += 0.1;
            }
            self.phase = fine_phase;
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
        let gray = if self.mode == GraphMode::Dark {
            1.0
        } else {
            0.0
        };
        for i in 0..num - 1 {
            let alpha_level = (i as f32) / (num as f32);
            let color = rgba(gray, gray, gray, alpha_level);
            draw.line()
                .start(self.polyline[i + 1])
                .end(self.polyline[i])
                .weight(self.thickness)
                .color(color);
        }
    }
}
