//  Created by Hasebe Masahiko on 2024/05/25.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::graphic::Resize;
use super::viewobj::{NormalView, NoteObj};
use crate::lpnlib::*;

pub struct StaticViewForVoice4 {
    font: nannou::text::Font,
}

impl StaticViewForVoice4 {
    pub fn new(font: nannou::text::Font) -> Self {
        Self {
            font,
        }
    }
}
impl NormalView for StaticViewForVoice4 {
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) {}
    fn disp(&self, draw: Draw, _tm: f32, rs: Resize) {
        let x = rs.get_full_size_x() / 5.0;
        let y = rs.get_full_size_y() / 5.0;
        let part_name = ["L1", "L2", "R1", "R2"];
        for i in 0..4 {
            let d = format!("{}", part_name[i]);
            draw.ellipse()
                //.w_h(x * 0.5, y * 0.5)
                .x_y(if i / 2 == 1 { x } else { -x }, 
                    if i % 2 == 0 { -y-5.0 } else { y-5.0 }
                )
                .color(MAGENTA)
                .radius(30.0);
            draw.text(&d)
                .font(self.font.clone())
                .font_size(32)
                .color(BLACK)
                .center_justify()
                .x_y(
                    if i / 2 == 1 { x } else { -x },
                    if i % 2 == 0 { -y } else { y },
                );
        }
    }
}

pub struct Voice4 {
    note: f32, // 0.0 - 1.0
    vel: f32,
    part: i32,
    time: f32,
    mode: GraphMode,
}

impl Voice4 {
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
impl NoteObj for Voice4 {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        let elapsed_time = crnt_time - self.time;
        if elapsed_time > Voice4::DISAPPEAR_TIME {
            false
        } else {
            true
        }
    }
    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        let elapsed_time = (crnt_time - self.time) * 4.0;
        if elapsed_time > Voice4::DISAPPEAR_TIME {
            return;
        }
        let rest = Voice4::DISAPPEAR_TIME - elapsed_time;
        let gray_scl = rest / Voice4::DISAPPEAR_TIME;
        let upper = if self.part % 2 == 1 { 1.0 } else { -1.0 }; // 0,2:upper, 1,3:lower
        let left = if self.part / 2 == 0 { -1.0 } else { 1.0 }; // 0,1:left, 2,3:right
        let rotary_phase = self.note * 3840.0; // noteによる回転phase
        let ntx = rotary_phase.cos() * (self.note / 2.0 + 0.5); // 相対x座標
        let nty = rotary_phase.sin() * (self.note / 2.0 + 0.5); // 相対y座標
        for i in 1..(Voice4::THICKNESS as usize) {
            let i_f32 = i as f32;
            let scale = if i_f32 > Voice4::THICKNESS / 2.0 {
                (Voice4::THICKNESS - i_f32) / (Voice4::THICKNESS / 2.0)
            } else {
                i_f32 / (Voice4::THICKNESS / 2.0)
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
