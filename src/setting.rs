//  Created by Hasebe Masahiko on 2024/04/13
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use eframe::egui::*;

// Default Window Size
pub const WINDOW_X_DEFAULT: f32 = 1280.0; //  Main Window
pub const WINDOW_Y_DEFAULT: f32 = 860.0;

// MIDI Connection
#[cfg(not(feature = "raspi"))]
pub const MIDI_OUT: &str = "IACdriver";
pub const MIDI_EXT_OUT: &str = "USB MIDI";
#[cfg(feature = "raspi")]
pub const MIDI_OUT: &str = "Midi Through:Midi Through Port-0 14:0"; // for Raspi5
pub const MIDI_DEVICE: &str = "Pico";
//pub const MIDI_DEVICE: &str = "Loopian-ORBIT";
//pub const MIDI_DEVICE: &str = "Arduino Leonardo";         // Arduino によるチェック
//pub const MIDI_DEVICE: &str = "TouchMIDI32 MIDI OUT";     // for Mac
//pub const MIDI_DEVICE: &str = "IACdriver InternalBus1"; // MAX によるチェック for Mac

// Font Data File Name with path
pub fn add_myfont() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    #[cfg(not(feature = "raspi"))]
    fonts.font_data.insert(
        "profont".to_owned(),
        FontData::from_static(include_bytes!("../assets/newyork.ttf")), // for Mac
    );
    #[cfg(feature = "raspi")]
    fonts.font_data.insert(
        "profont".to_owned(),
        FontData::from_static(include_bytes!(
            "/home/pi/loopian/Loopian_Rust/assets/NewYork.ttf"
        )), // for linux
    );
    #[cfg(not(feature = "raspi"))]
    fonts.font_data.insert(
        "monofont".to_owned(),
        FontData::from_static(include_bytes!("../assets/courier.ttc")), // for Mac
    );
    #[cfg(feature = "raspi")]
    fonts.font_data.insert(
        "monofont".to_owned(),
        FontData::from_static(include_bytes!(
            "/home/pi/loopian/Loopian_Rust/assets/Courier.ttc"
        )), // for linux
    );
    fonts
}

// Max Pattern Number
pub const MAX_PATTERN_NUM: u8 = 16;
