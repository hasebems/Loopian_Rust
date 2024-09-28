//  Created by Hasebe Masahiko on 2024/09/28.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use std::fs;
//use std::fs::File;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowSize {
    pub window_x_default: f32,
    pub window_y_default: f32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MIDI {
    pub midi_out: String,
    pub midi_ext_out: String,
    pub midi_device: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub window_size: WindowSize,
    pub midi: MIDI,
}

impl Settings {
    const SETTINGS_FILE: &'static str = "settings.toml";
    pub fn load_settings() -> Settings {
        let fs: String = fs::read_to_string(Self::SETTINGS_FILE).unwrap();
        let sts: Result<Settings, toml::de::Error> = toml::from_str(&fs);
        match sts {
            Ok(s) => s,
            Err(e) => panic!("Filed to parse TOML: {}", e),
        }
    }
}
