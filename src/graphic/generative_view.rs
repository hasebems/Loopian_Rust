//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::Resize;
use crate::lpnlib::*;

pub struct GraphicPatternName(pub GraphicPattern, pub GraphicMsg, pub &'static str);
pub const GRAPHIC_PATTERN_NAME: [GraphicPatternName; 6] = [
    GraphicPatternName(GraphicPattern::Ripple, GraphicMsg::RipplePattern, "ripple"),
    GraphicPatternName(GraphicPattern::Voice4, GraphicMsg::VoicePattern, "voice"),
    GraphicPatternName(
        GraphicPattern::Lissajous,
        GraphicMsg::LissajousPattern,
        "lissa",
    ),
    GraphicPatternName(
        GraphicPattern::BeatLissa,
        GraphicMsg::BeatLissaPattern(0),
        "beatlissa(0)",
    ),
    GraphicPatternName(
        GraphicPattern::BeatLissa,
        GraphicMsg::BeatLissaPattern(1),
        "beatlissa(1)",
    ),
    GraphicPatternName(
        GraphicPattern::SineWave,
        GraphicMsg::SineWavePattern,
        "sinewave",
    ),
];

pub trait GenerativeView {
    /// 画面全体の Model の更新
    fn update_model(&mut self, crnt_time: f32, rs: Resize);
    /// Note 演奏情報を受け取る
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {}
    /// Beat 演奏情報を受け取る
    fn on_beat(&mut self, _bt: i32, _ct: f32, _dt: f32) {}
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
