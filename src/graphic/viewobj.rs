//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::graphic::Resize;

pub trait NormalView {
    fn update_model(&mut self, crnt_time: f32, rs: Resize);
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    );
}

pub trait NoteObj {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool; //  false: 消去可能
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,     //  ウィンドウサイズ
    );
}
