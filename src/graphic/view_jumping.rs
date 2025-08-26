//  Created by Hasebe Masahiko on 2025/08/26.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::draw_graph::Resize;
use crate::graphic::generative_view::GenerativeView;
use crate::lpnlib::GraphMode;
use nannou::prelude::*;

//*******************************************************************
//      Screen Graphic
//*******************************************************************
pub struct Jumping {
    rs: Resize,
    beat_start_time: f32,
    crnt_bt: i32,
    x_normalized: f32,
    y_normalized: f32,
    obj: Vec<JumpingObject>,
}
impl Jumping {
    pub fn new() -> Self {
        Self {
            rs: Resize::default(),
            beat_start_time: 0.0,
            crnt_bt: 0,
            x_normalized: 0.0,
            y_normalized: 0.0,
            obj: Vec::new(),
        }
    }
}
impl GenerativeView for Jumping {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        self.rs = rs;
        let elapsed = crnt_time - self.beat_start_time;
        let sq = pow(elapsed - self.x_normalized, 2);
        let y_axis = 1.0 - sq / self.y_normalized;
        self.obj.iter_mut().for_each(|o| {
            o.update(crnt_time, y_axis);
        });
        self.obj.retain(|o| !o.over_bounds);
    }
    /// Beat 演奏情報を受け取る
    fn on_beat(&mut self, bt: i32, tm: f32, dt: f32) {
        if self.crnt_bt != bt {
            self.crnt_bt = bt;
            self.beat_start_time = tm;
            self.x_normalized = dt / 2.0;
            self.y_normalized = pow(dt / 2.0, 2);
            //println!("*** Jumping Beat: {bt}, time: {tm}, dt: {dt}");
            if bt % 2 == 0 {
                self.obj.push(JumpingObject::new(
                    tm,
                    self.rs.get_full_size_x() / 2.0,
                    self.rs.get_full_size_y() / 2.0,
                ));
            }
        }
    }
    fn set_mode(&mut self, _mode: GraphMode) {}
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        _rs: Resize,
    ) {
        self.obj.iter().for_each(|o| {
            o.disp(&draw, crnt_time);
        });
    }
}

//*******************************************************************
//      Beat Graphic
//*******************************************************************
pub struct JumpingObject {
    x_axis: f32,
    y_axis: f32,
    start_time: f32,
    x_limit: f32,
    y_limit: f32,
    x_multify: f32,
    y_multify: f32,
    radius: f32,
    obj_type: usize,
    pub over_bounds: bool,
}
impl JumpingObject {
    const BASE_Y_AXIS: f32 = -200.0;

    pub fn new(crnt_time: f32, x_limit: f32, y_limit: f32) -> Self {
        Self {
            x_axis: 0.0,
            y_axis: 0.0,
            start_time: crnt_time,
            x_limit,
            y_limit,
            x_multify: random_range(200.0, 500.0),
            y_multify: random_range(100.0, 400.0),
            radius: random_range(50.0, 150.0),
            obj_type: random_range(0, 3),
            over_bounds: false,
        }
    }
    fn update(&mut self, crnt_time: f32, y_axis: f32) {
        self.y_axis = Self::BASE_Y_AXIS + y_axis * self.y_multify;
        self.x_axis = -self.x_limit + (crnt_time - self.start_time) * self.x_multify;
        self.over_bounds = self.x_axis.abs() > self.x_limit || y_axis.abs() > self.y_limit;
    }
    fn disp(
        &self,
        draw: &Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
                         //rs: Resize,     //  ウィンドウサイズ
    ) {
        let y_rad = if self.y_axis < Self::BASE_Y_AXIS + 50.0 {
            self.radius * (Self::BASE_Y_AXIS - 50.0 - self.y_axis) / 100.0
        } else {
            self.radius
        };
        if self.obj_type == 0 {
            draw.ellipse()
                .x_y(self.x_axis, self.y_axis)
                .w_h(self.radius, y_rad.abs())
                .no_fill()
                .stroke_weight(2.0)
                .stroke(WHITE);
        } else if self.obj_type == 1 {
            draw.rect()
                .x_y(self.x_axis, self.y_axis)
                .w_h(self.radius, y_rad.abs())
                .no_fill()
                .stroke_weight(2.0)
                .stroke(WHITE);
        } else if self.obj_type == 2 {
            draw.tri()
                .points(
                    pt2(self.x_axis - self.radius / 2.0, self.y_axis - self.radius / 2.0),
                    pt2(self.x_axis + self.radius / 2.0, self.y_axis - self.radius / 2.0),
                    pt2(self.x_axis, self.y_axis + self.radius / 2.0),
            )
            .no_fill()
            .stroke_weight(2.0)
            .stroke(WHITE);
        }
    }
}
