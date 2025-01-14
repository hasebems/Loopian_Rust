//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GraphMode {
    Dark,
    Light,
}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GraphPattern {
    Ripple,
    Voice4,
    Lissajous,
    BeatLissa,
}

pub trait NormalView {
    /// 画面全体の Model の更新
    fn update_model(&mut self, crnt_time: f32, rs: Resize);
    /// Note 演奏情報を受け取る
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {}
    /// Beat 演奏情報を受け取る
    fn on_beat(&mut self, _bt: i32, _tm: f32) {}
    /// オブジェクトの位置を取得
    fn get_obj_position(&self, _otype: usize, _num: usize) -> Vec2 {
        (0.0, 0.0).into()
    }
    /// 現在のオブジェクト数を取得
    fn get_crnt_num(&self) -> usize {
        0
    }
    /// Mode 情報を受け取る
    fn set_mode(&mut self, _mode: GraphMode) {}
    /// 画面全体の描画
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    );
}

pub trait NoteObj {
    /// Note の Model の更新
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool; //  false: 消去可能
    /// Note の描画
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,     //  ウィンドウサイズ
    );
}

pub trait BeatObj {
    /// Beat の Model の更新
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool; //  false: 消去可能
    /// Beat の描画
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,     //  ウィンドウサイズ
    );
}
