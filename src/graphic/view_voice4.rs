//  Created by Hasebe Masahiko on 2024/05/25.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use super::generative_view::*;
use crate::lpnlib::GraphMode;

pub struct Voice4 {
    font: nannou::text::Font,
    nobj: Vec<Box<dyn NoteObj>>, // Note Object
    mode: GraphMode,
}

impl Voice4 {
    pub fn new(font: nannou::text::Font) -> Self {
        Self {
            font,
            nobj: Vec::new(),
            mode: GraphMode::Dark,
        }
    }
}
impl GenerativeView for Voice4 {
    fn update_model(&mut self, tm: f32, rs: Resize) {
        // Note Object の更新と削除
        let mut retain: Vec<bool> = Vec::new();
        for obj in self.nobj.iter_mut() {
            retain.push(obj.update_model(tm, rs.clone()));
        }
        for (j, rt) in retain.iter().enumerate() {
            if !rt {
                self.nobj.remove(j);
                break;
            }
        }
    }
    /// Note 演奏情報を受け取る
    fn note_on(&mut self, nt: i32, vel: i32, pt: i32, tm: f32) {
        self.nobj.push(Box::new(Voice4Note::new(
            nt as f32, vel as f32, pt, tm, self.mode,
        )));
    }
    /// Mode 情報を受け取る
    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }
    fn disp(&self, draw: Draw, tm: f32, rs: Resize) {
        let x = rs.get_full_size_x() / 5.0;
        let y = rs.get_full_size_y() / 5.0;
        let part_name = ["L1", "L2", "R1", "R2"];
        for (i, &pt) in part_name.iter().enumerate() {
            let d = pt.to_string();
            draw.text(&d)
                .font(self.font.clone())
                .font_size(32)
                .color(MAGENTA)
                .center_justify()
                .x_y(
                    if i / 2 == 1 { x } else { -x },
                    if i % 2 == 0 { -y } else { y },
                );
            draw.rect()
                .x_y(
                    if i / 2 == 1 { x } else { -x },
                    if i % 2 == 0 { -y - 5.0 } else { y - 5.0 },
                )
                .w_h(50.0, 30.0)
                .no_fill()
                .stroke_weight(2.0)
                .stroke(MAGENTA);
        }
        //  Note Object の描画
        for obj in self.nobj.iter() {
            obj.disp(draw.clone(), tm, rs.clone());
        }
    }
}

pub struct Voice4Note {
    note: f32, // 0.0 - 1.0
    vel: f32,
    part: i32,
    time: f32,
    mode: GraphMode,
}

impl Voice4Note {
    const DISAPPEAR_TIME: f32 = 5.0; // Bigger, Slower
    const THICKNESS: f32 = 20.0;
    pub fn new(nt: f32, vel: f32, pt: i32, tm: f32, mode: GraphMode) -> Self {
        Self {
            note: nt / 127.0,
            vel: (vel * vel / 16384.0), // velは小さい時に薄くするため二乗
            part: pt,
            time: tm,
            mode,
        }
    }
}
impl NoteObj for Voice4Note {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        let elapsed_time = crnt_time - self.time;
        elapsed_time <= Self::DISAPPEAR_TIME
    }
    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        let elapsed_time = (crnt_time - self.time) * 4.0;
        if elapsed_time > Self::DISAPPEAR_TIME {
            return;
        }
        let rest = Self::DISAPPEAR_TIME - elapsed_time;
        let gray_scl = rest / Self::DISAPPEAR_TIME;
        let upper = if self.part % 2 == 1 { 1.0 } else { -1.0 }; // 0,2:upper, 1,3:lower
        let left = if self.part / 2 == 0 { -1.0 } else { 1.0 }; // 0,1:left, 2,3:right
        let rotary_phase = self.note * 3840.0; // noteによる回転phase
        let ntx = rotary_phase.cos() * (self.note / 2.0 + 0.5); // 相対x座標
        let nty = rotary_phase.sin() * (self.note / 2.0 + 0.5); // 相対y座標
        for i in 1..(Self::THICKNESS as usize) {
            let i_f32 = i as f32;
            let scale = if i_f32 > Self::THICKNESS / 2.0 {
                (Self::THICKNESS - i_f32) / (Self::THICKNESS / 2.0)
            } else {
                i_f32 / (Self::THICKNESS / 2.0)
            };
            let alpha_level = gray_scl * scale;
            let gray = if self.mode == GraphMode::Dark {
                rgba(1.0, 1.0, 1.0, alpha_level)
            } else {
                rgba(0.0, 0.0, 0.0, alpha_level)
            };
            draw.ellipse()
                .x_y(
                    ((ntx * 0.1) + (0.2 * left)) * rs.get_full_size_x(),
                    ((nty * 0.1) + (0.2 * upper)) * rs.get_full_size_y(),
                )
                .no_fill()
                .stroke_weight(2.0)
                .radius(200.0 * self.vel - i_f32)
                .stroke(gray);
        }
    }
}
