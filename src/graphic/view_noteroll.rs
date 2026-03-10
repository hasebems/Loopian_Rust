//  Created by Hasebe Masahiko on 2025/01/08.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::draw_graph::*;
use super::generative_view::*;
use nannou::prelude::*;

//*******************************************************************
//      Screen Graphic
//*******************************************************************
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RollMode {
    Vertical,
    Horizontal,
}
pub struct NoteRoll {
    mode: GraphMode,
    roll_mode: RollMode,
    nobj: Vec<Box<dyn NoteObj>>, // Note Object
}
impl NoteRoll {
    pub fn new(roll_mode: &str, mode: GraphMode) -> Self {
        let roll_mode = if roll_mode == "h" {
            RollMode::Horizontal
        } else {
            RollMode::Vertical
        };
        Self {
            mode,
            roll_mode,
            nobj: Vec::new(),
        }
    }
}
impl GenerativeView for NoteRoll {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        // Note Object の更新と削除
        let mut retain: Vec<bool> = Vec::new();
        for obj in self.nobj.iter_mut() {
            retain.push(obj.update_model(crnt_time, rs.clone()));
        }
        for (j, rt) in retain.iter().enumerate() {
            if !rt {
                self.nobj.remove(j);
                break;
            }
        }
    }
    /// Beat Event
    fn on_beat(&mut self, _bt: i32, _tm: f32, _dt: f32) {}
    /// Note Event
    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, tm: f32) {
        self.nobj.push(Box::new(NoteRollNote::new(
            nt as f32,
            vel as f32,
            tm,
            self.roll_mode,
            self.mode,
        )));
    }
    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    ) {
        //  Note Object の描画
        for obj in self.nobj.iter() {
            obj.disp(draw.clone(), crnt_time, rs.clone());
        }
    }
}

//*******************************************************************
//      Note Graphic
//*******************************************************************
pub struct NoteRollNote {
    nt: f32,
    vel: f32,
    start_time: f32,
    elapsed_time: f32,
    roll_mode: RollMode,
    mode: GraphMode,
}
impl NoteRollNote {
    const CIRCLE_SIZE: f32 = 16.0;
    const CIRCLE_USIZE: usize = Self::CIRCLE_SIZE as usize;

    pub fn new(midi_nt: f32, vel: f32, time: f32, roll_mode: RollMode, mode: GraphMode) -> Self {
        let nt = ((midi_nt - 60.0) / 40.0).clamp(-1.0, 1.0);
        Self {
            nt,
            vel: vel / 127.0,
            start_time: time,
            elapsed_time: 0.0,
            roll_mode,
            mode,
        }
    }
}
impl NoteObj for NoteRollNote {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        self.elapsed_time = (crnt_time - self.start_time) / 2.0;
        self.elapsed_time < 1.0
    }
    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,      //  ウィンドウサイズ
    ) {
        //let disappear_time_inv = 1.0 / Self::DISAPPEAR_TIME;
        let half_size_x = rs.get_full_size_x() / 2.0;
        let half_size_y = rs.get_full_size_y() / 2.0;
        let x_offset = if self.roll_mode == RollMode::Vertical {
            0.0 + self.nt * half_size_x
        } else {
            0.0 + self.elapsed_time * half_size_x
        };
        let y_offset = if self.roll_mode == RollMode::Vertical {
            0.0 + self.elapsed_time * half_size_y
        } else {
            0.0 + self.nt * half_size_y
        };
        let x_offset_mirror = if self.roll_mode == RollMode::Vertical {
            0.0 + self.nt * half_size_x
        } else {
            0.0 - self.elapsed_time * half_size_x
        };
        let y_offset_mirror = if self.roll_mode == RollMode::Vertical {
            0.0 - self.elapsed_time * half_size_y
        } else {
            0.0 + self.nt * half_size_y
        };

        for i in 0..Self::CIRCLE_USIZE {
            let alpha_level =
                (1.0 - ((i as f32) / Self::CIRCLE_SIZE)) * self.vel * (1.0 - self.elapsed_time);
            if alpha_level <= 0.0 {
                return;
            }
            let gray_scal = if self.mode == GraphMode::Dark {
                rgba(1.0, 1.0, 1.0, alpha_level)
            } else {
                rgba(0.0, 0.0, 0.0, alpha_level * 2.0) // 少し濃いめ
            };

            let radius_sz = ((i + 1) as f32) * 3.0 - self.elapsed_time * Self::CIRCLE_SIZE;
            if radius_sz > 0.0 {
                if i == 0 {
                    // 中心は塗りつぶし
                    draw.ellipse()
                        .x_y(x_offset, y_offset)
                        .color(gray_scal)
                        .radius(radius_sz + 1.0);
                    draw.ellipse()
                        .x_y(x_offset_mirror, y_offset_mirror)
                        .color(gray_scal)
                        .radius(radius_sz + 1.0);
                    continue;
                }
                draw.ellipse()
                    .x_y(x_offset, y_offset)
                    .no_fill()
                    .stroke_weight(3.0)
                    .stroke(gray_scal)
                    .radius(radius_sz);
                draw.ellipse()
                    .x_y(x_offset_mirror, y_offset_mirror)
                    .no_fill()
                    .stroke_weight(3.0)
                    .stroke(gray_scal)
                    .radius(radius_sz);
            }
        }
    }
}
