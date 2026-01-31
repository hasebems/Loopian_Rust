//  Created by Hasebe Masahiko on 2025/01/08.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;
use super::generative_view::*;
use super::draw_graph::*;

//*******************************************************************
//     メモ： View 作成のテンプレート
// 必ず GenerativeView を実装すること
// 常に動作を伴う表示を行う場合： GenerativeView の disp() を実装する
// ノートに同期した表示を行う場合：
//  - GenerativeView note_on() で、NoteObj を生成し Vec<NoteObj>に追加
//  - GenerativeView disp() 内で NoteObj の disp() を呼び出す
//  - NoteObj を実装する
// ビートに同期した表示を行う場合：
//  - GenerativeView on_beat() で、BeatObj を生成し Vec<BeatObj>に追加
//  - GenerativeView disp() 内で BeatObj の disp() を呼び出す
//  - BeatObj を実装する
//*******************************************************************
//      Screen Graphic
//*******************************************************************
pub struct UpDownRoll {
    mode: GraphMode,
    nobj: Vec<Box<dyn NoteObj>>, // Note Object
}
impl UpDownRoll {
    pub fn new(mode: GraphMode) -> Self {
        Self {
            mode,
            nobj: Vec::new(),
        }
    }
}
impl GenerativeView for UpDownRoll {
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
    fn on_beat(&mut self, _bt: i32, _tm: f32, _dt: f32) {
    }
    /// Note Event
    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, tm: f32) {
        self.nobj.push(Box::new(UpDownRollNote::new(
            nt as f32, vel as f32, tm, self.mode,
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
pub struct UpDownRollNote {
    nt: f32,
    vel: f32,
    start_time: f32,
    elapsed_time: f32,
    mode: GraphMode,
}
impl UpDownRollNote {
    const CIRCLE_SIZE: f32 = 16.0;
    const CIRCLE_USIZE: usize = Self::CIRCLE_SIZE as usize;

    pub fn new(midi_nt: f32, vel: f32, time: f32, mode: GraphMode) -> Self {
        let nt = ((midi_nt - 60.0) / 30.0).clamp(-1.0, 1.0);
        Self {
            nt,
            vel: vel / 127.0,
            start_time: time,
            elapsed_time: 0.0,
            mode,
        }
    }
}
impl NoteObj for UpDownRollNote {
    fn update_model(&mut self, crnt_time: f32, _rs: Resize) -> bool {
        self.elapsed_time = (crnt_time - self.start_time) / 2.0;
        self.elapsed_time < 1.0
    }
    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,     //  ウィンドウサイズ
    ) {
        //let disappear_time_inv = 1.0 / Self::DISAPPEAR_TIME;
        let half_size_x = rs.get_full_size_x() / 2.0;
        let half_size_y = rs.get_full_size_y() / 2.0;
        let x_offset = 0.0 + self.nt * half_size_x;
        let y_offset = 0.0 + self.elapsed_time * half_size_y;

        for i in 0..Self::CIRCLE_USIZE {
            let alpha_level = (1.0 - ((i as f32) / Self::CIRCLE_SIZE))
                * self.vel
                * (1.0 - self.elapsed_time);            
            if alpha_level <= 0.0 {
                return;
            }
            let gray_scal = if self.mode == GraphMode::Dark {
                rgba(1.0, 1.0, 1.0, alpha_level)
            } else {
                rgba(0.0, 0.0, 0.0, alpha_level*2.0) // 少し濃いめ
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
                        .x_y(x_offset, -y_offset)
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
                    .x_y(x_offset, -y_offset)
                    .no_fill()
                    .stroke_weight(3.0)
                    .stroke(gray_scal)
                    .radius(radius_sz);
            }
        }
    }
}
