//  Created by Hasebe Masahiko on 2024/05/25.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use eframe::egui::*; //{egui::*, epaint::WHITE_UV};

use super::noteobj::NoteObj;

pub struct Voice4 {
    note: f32, // 0.0 - 1.0
    vel: f32,
    part: i32,
    time: i32,
    mode: GraphMode,
}

impl Voice4 {
    const DISAPPEAR_RATE: f32 = 100.0; // Bigger, Slower
    const THICKNESS: f32 = 20.0;
    pub fn new(nt: f32, vel: f32, pt: i32, tm: i32, mode: GraphMode) -> Self {
        Self {
            note: nt / 127.0,
            vel: (vel * vel / 16384.0), // velは小さい時に薄くするため二乗
            part: pt,
            time: tm,
            mode,
        }
    }
    fn gray(&self, gray_scl: u8) -> Color32 {
        if self.mode == GraphMode::Light {
            Color32::from_black_alpha(gray_scl)
        } else if self.mode == GraphMode::Dark {
            Color32::from_white_alpha(gray_scl)
        } else {
            Color32::from_white_alpha(gray_scl)
        }
    }
}
impl NoteObj for Voice4 {
    fn disp(&self, crnt_time: i32, ui: &mut Ui, fsz: Pos2) -> bool {
        let cnt = (crnt_time - self.time) as f32 * 4.0;
        if cnt > Voice4::DISAPPEAR_RATE {
            return false;
        }
        let rate = Voice4::DISAPPEAR_RATE - cnt;
        let gray_scl = rate * 255.0 / Voice4::DISAPPEAR_RATE;
        let upper = if self.part % 2 == 1 { 1.0 } else { 0.0 }; // 0,2:upper, 1,3:lower
        let left = if self.part / 2 == 0 { 0.0 } else { 1.0 }; // 0,1:left, 2,3:right
        let rotary_phase = self.note * 3840.0; // noteによる回転phase
        let ntx = rotary_phase.cos() * (2.0 - self.note) / 2.0; // 相対x座標
        let nty = rotary_phase.sin() * (2.0 - self.note) / 2.0; // 相対y座標
        for i in 1..(Voice4::THICKNESS as usize) {
            let i_f32 = i as f32;
            let scale = if i_f32 > Voice4::THICKNESS / 2.0 {
                (Voice4::THICKNESS - i_f32) / (Voice4::THICKNESS / 2.0)
            } else {
                i_f32 / (Voice4::THICKNESS / 2.0)
            };
            let gray = (gray_scl * scale) as u8;
            ui.painter().circle_stroke(
                Pos2 {
                    x: ((ntx * 0.1) + (0.4 * left) + 0.3) * fsz.x,
                    y: ((nty * 0.1) + (0.4 * upper) + 0.3) * fsz.y,
                }, // location
                200.0 * self.vel - i_f32, // radius
                Stroke {
                    width: 1.0,
                    color: self.gray(gray),
                },
            );
        }
        true
    }
}
