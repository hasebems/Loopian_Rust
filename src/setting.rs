//  Created by Hasebe Masahiko on 2024/04/13
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use eframe::egui::*;

// Default Window Size
pub const WINDOW_X_DEFAULT: f32 = 1000.0; //  Main Window
pub const WINDOW_Y_DEFAULT: f32 = 860.0;

// MIDI Connection
pub const MIDI_OUT: &str = "IACdriver";
//pub const MIDI_OUT: &str = "Midi Through:Midi Through Port-0 14:0"; // for Raspi5
pub const MIDI_DEVICE: &str = "Pico";
//pub const MIDI_DEVICE: &str = "Loopian-ORBIT";
//pub const MIDI_DEVICE: &str = "Arduino Leonardo";         // Arduino によるチェック
//pub const MIDI_DEVICE: &str = "TouchMIDI32 MIDI OUT";     // for Mac
//pub const MIDI_DEVICE: &str = "IACdriver InternalBus1"; // MAX によるチェック for Mac

// Font Data File Name with path
pub fn add_myfont() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    fonts.font_data.insert(
        "profont".to_owned(),
        FontData::from_static(include_bytes!("../assets/newyork.ttf")),// for Mac
        //FontData::from_static(include_bytes!("/home/pi/loopian/Loopian_Rust/assets/NewYork.ttf")),// for linux
    );
    fonts.font_data.insert(
        "monofont".to_owned(),
        FontData::from_static(include_bytes!("../assets/courier.ttc")),// for Mac
        //FontData::from_static(include_bytes!("/home/pi/loopian/Loopian_Rust/assets/Courier.ttc")),// for linux
    );
    fonts
}