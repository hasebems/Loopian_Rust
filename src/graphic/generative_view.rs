//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;

use super::draw_graph::*;
use super::guiev::*;
use super::view_beatlissa::*;
use super::view_circlethds::*;
use super::view_fish::*;
use super::view_jumping::*;
use super::view_lissajous::*;
use super::view_noteroll::*;
use super::view_raineffect::*;
use super::view_sinewave::*;
use super::view_voice4::*;
use super::view_waterripple::*;
use super::view_wavestick::*;
use crate::cmd::txt_common::*;

//*******************************************************************
//      Enum, Table
//*******************************************************************
// Graphic Message
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GraphicMsg {
    What,
    NoMsg,
    LightMode,
    DarkMode,
    TextVisibleCtrl,
    Title(String, String),
    RipplePattern,
    VoicePattern,
    LissajousPattern,
    BeatLissaPattern(i32),
    SineWavePattern,
    RainEffectPattern,
    FishPattern,
    JumpingPattern,
    WaveStickPattern,
    CircleThdsPattern,
    NoteRollPattern(String),
}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GraphMode {
    Dark,
    Light,
}
pub fn generate_graphic_msg(input_msgs: Vec<String>) -> (String, GraphicMsg) {
    if input_msgs[1] == "light" {
        ("Changed Graphic!".to_string(), GraphicMsg::LightMode)
    } else if input_msgs[1] == "dark" {
        ("Changed Graphic!".to_string(), GraphicMsg::DarkMode)
    } else if input_msgs[1] == "title" {
        let txt = extract_texts_from_parentheses(&input_msgs[1]);
        let txts = txt.split(',').collect::<Vec<&str>>();
        let title_txt = txts.first().unwrap_or(&"");
        let subtitle_txt = txts.get(1).unwrap_or(&"");
        (
            format!("Set Title: {}", title_txt),
            GraphicMsg::Title(title_txt.to_string(), subtitle_txt.to_string()),
        )
    } else if input_msgs[1].contains("ripple") {
        ("Changed Graphic!".to_string(), GraphicMsg::RipplePattern)
    } else if input_msgs[1].contains("voice") {
        ("Changed Graphic!".to_string(), GraphicMsg::VoicePattern)
    } else if input_msgs[1].contains("lissa") {
        ("Changed Graphic!".to_string(), GraphicMsg::LissajousPattern)
    } else if input_msgs[1].contains("beatlissa") {
        let prm = extract_texts_from_parentheses(&input_msgs[1]);
        let num = prm.parse::<i32>().unwrap_or(0);
        (
            "Changed Graphic!".to_string(),
            GraphicMsg::BeatLissaPattern(num),
        )
    } else if input_msgs[1].contains("sinewave") {
        ("Changed Graphic!".to_string(), GraphicMsg::SineWavePattern)
    } else if input_msgs[1].contains("rain") {
        (
            "Changed Graphic!".to_string(),
            GraphicMsg::RainEffectPattern,
        )
    } else if input_msgs[1].contains("fish") {
        ("Changed Graphic!".to_string(), GraphicMsg::FishPattern)
    } else if input_msgs[1].contains("jumping") {
        ("Changed Graphic!".to_string(), GraphicMsg::JumpingPattern)
    } else if input_msgs[1].contains("wavestick") {
        ("Changed Graphic!".to_string(), GraphicMsg::WaveStickPattern)
    } else if input_msgs[1].contains("circlethreads") {
        (
            "Changed Graphic!".to_string(),
            GraphicMsg::CircleThdsPattern,
        )
    } else if input_msgs[1].contains("noteroll") {
        let prm = extract_texts_from_parentheses(&input_msgs[1]);
        (
            "Changed Graphic!".to_string(),
            GraphicMsg::NoteRollPattern(prm.to_string()),
        )
    } else {
        ("what?".to_string(), GraphicMsg::What)
    }
}

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
    gmsg: &GraphicMsg,
    gmode: GraphMode,
    font_nrm: nannou::text::Font,
) -> Option<Box<dyn GenerativeView>> {
    match gmsg {
        // ◆◆◆ generative_view が追加されたらここに追加
        GraphicMsg::RipplePattern => Some(Box::new(WaterRipple::new(gmode))),
        GraphicMsg::VoicePattern => Some(Box::new(Voice4::new(font_nrm.clone()))),
        GraphicMsg::LissajousPattern => Some(Box::new(Lissajous::new(gmode))),
        GraphicMsg::BeatLissaPattern(md) => {
            let mt = guiev.get_indicator(INDC_METER).to_string();
            let num_str = split_by('/', mt);
            let num = num_str[0].parse::<i32>().unwrap_or(0);
            Some(Box::new(BeatLissa::new(num, crnt_time, *md, gmode)))
        }
        GraphicMsg::SineWavePattern => Some(Box::new(SineWave::new(gmode))),
        GraphicMsg::RainEffectPattern => Some(Box::new(RainEffect::new(gmode))),
        GraphicMsg::FishPattern => Some(Box::new(SchoolOfFish::new())),
        GraphicMsg::JumpingPattern => Some(Box::new(Jumping::new())),
        GraphicMsg::WaveStickPattern => Some(Box::new(WaveStick::new())),
        GraphicMsg::CircleThdsPattern => Some(Box::new(CircleThread::new())),
        GraphicMsg::NoteRollPattern(tp) => Some(Box::new(NoteRoll::new(tp, gmode))),
        _ => None,
    }
}
