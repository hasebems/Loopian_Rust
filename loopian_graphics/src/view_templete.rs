//  Created by Hasebe Masahiko on 2025/01/08.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;
use loopian_graphic_api::generative_view::{BeatObj, GenerativeView, GraphMode, NoteObj};
use loopian_graphic_api::Resize;

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
pub struct GraphicTemplete {
    pub rs: Resize,
}
impl GraphicTemplete {
    pub fn new() -> Self {
        Self {
            rs: Resize::default(),
        }
    }
}
impl GenerativeView for GraphicTemplete {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
    }
    /// Beat Event
    fn on_beat(&mut self, bt: i32, tm: f32, dt: f32) {
    }
    /// Note Event
    fn note_on(&mut self, _nt: i32, vel: i32, _pt: i32, _tm: f32) {
    }
    fn set_mode(&mut self, mode: GraphMode) {
    }
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    ) {
        // 原点
        draw.line()
            .start(pt2(100.0, 0.0))
            .end(pt2(-100.0, 0.0))
            .color(RED);
        draw.line()
            .start(pt2(0.0, 100.0))
            .end(pt2(0.0, -100.0))
            .color(RED);
    }
}

//*******************************************************************
//      Note Graphic
//*******************************************************************
pub struct GraphicNoteTemplete {

}
impl GraphicNoteTemplete {
    pub fn new() -> Self {
        Self {
        }
    }
}
impl NoteObj for GraphicNoteTemplete {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool {
        false
    }
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,     //  ウィンドウサイズ
    ) {
    }
}

//*******************************************************************
//      Beat Graphic
//*******************************************************************
pub struct GraphicBeatTemplete {

}
impl GraphicBeatTemplete {
    pub fn new() -> Self {
        Self {
        }
    }
}
impl BeatObj for GraphicBeatTemplete {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool {
        false
    }
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,     //  ウィンドウサイズ
    ) {
    }
}
