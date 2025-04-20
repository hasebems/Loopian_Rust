//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::*;
use super::guiev::*;
use super::view_beatlissa::*;
use super::view_lissajous::*;
use super::view_raineffect::*;
use super::view_sinewave::*;
use super::view_voice4::*;
use super::view_waterripple::*;
use crate::cmd::txt_common::*;
use crate::lpnlib::*;

//*******************************************************************
//      Enum, Table
//*******************************************************************
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GraphicPattern {
    Ripple,
    Voice4,
    Lissajous,
    BeatLissa,
    SineWave,
    RainEffect,
}
pub struct GraphicPatternName(pub GraphicPattern, pub GraphicMsg, pub &'static str);
pub const GRAPHIC_PATTERN_NAME: [GraphicPatternName; 7] = [
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
    GraphicPatternName(
        GraphicPattern::RainEffect,
        GraphicMsg::RainEffectPattern,
        "rain",
    ),
];

//*******************************************************************
//      struct GenerativeView
//*******************************************************************
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

//*******************************************************************
//      Get GenerativeView Instance
//*******************************************************************
pub fn get_view_instance(
    guiev: &mut GuiEv,
    crnt_time: f32,
    gmsg: GraphicMsg,
    gmode: GraphMode,
    font_nrm: nannou::text::Font,
) -> (Option<GraphicPattern>, Option<Box<dyn GenerativeView>>) {
    let gptn;
    let view: Option<Box<dyn GenerativeView>>;
    match gmsg {
        // ◆◆◆ generative_view が追加されたらここに追加
        GraphicMsg::RipplePattern => {
            gptn = Some(GRAPHIC_PATTERN_NAME[0].0); //GraphicPattern::Ripple;
            view = Some(Box::new(WaterRipple::new(gmode)));
        }
        GraphicMsg::VoicePattern => {
            gptn = Some(GRAPHIC_PATTERN_NAME[1].0);
            view = Some(Box::new(Voice4::new(font_nrm.clone())));
        }
        GraphicMsg::LissajousPattern => {
            gptn = Some(GRAPHIC_PATTERN_NAME[2].0);
            view = Some(Box::new(Lissajous::new(gmode)));
        }
        GraphicMsg::BeatLissaPattern(md) => {
            let mt = guiev.get_indicator(INDC_METER).to_string();
            let num_str = split_by('/', mt);
            let num = num_str[0].parse::<i32>().unwrap_or(0);
            gptn = Some(GRAPHIC_PATTERN_NAME[3].0);
            view = Some(Box::new(BeatLissa::new(num, crnt_time, md, gmode)));
        }
        GraphicMsg::SineWavePattern => {
            gptn = Some(GRAPHIC_PATTERN_NAME[5].0);
            view = Some(Box::new(SineWave::new(gmode)));
        }
        GraphicMsg::RainEffectPattern => {
            gptn = Some(GRAPHIC_PATTERN_NAME[6].0);
            view = Some(Box::new(RainEffect::new(gmode)));
        }
        _ => {
            gptn = None;
            view = None;
        }
    }
    (gptn, view)
}
