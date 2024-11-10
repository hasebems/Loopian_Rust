//  Created by Hasebe Masahiko on 2024/09/28.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use std::fs;
use std::env;
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
        match fs::read_to_string(Self::SETTINGS_FILE) {
            Ok(fs) => {
                let sts: Result<Settings, toml::de::Error> = toml::from_str(&fs);
                match sts {
                    Ok(s) => s,
                    Err(e) => panic!("Filed to parse TOML: {}", e),
                }
            }
            Err(e) => {
                panic!("Failed to read settings file: {}", e);
            }
        }
    }
    pub fn find_setting_file() -> bool {
        if fs::metadata(Self::SETTINGS_FILE).is_err() {
            // もし設定ファイルが存在しない場合は、実行ファイルのディレクトリに移動

            // 現在の実行ファイルのパスを取得
            let exe_path = env::current_exe().expect("Failed to get current exe path");
        
            // 実行ファイルのディレクトリパスを取得
            let exe_dir = exe_path.parent().expect("Failed to get exe directory");

            if exe_path != *exe_dir {
                // ディレクトリを移動
                env::set_current_dir(exe_dir).expect("Failed to change directory");
                if fs::metadata(Self::SETTINGS_FILE).is_err() {
                    println!("Settings file not found.");
                    return false;
                } else {
                    println!("*** Settings file found.");
                }
            } else {
                println!("Settings file not found.");
                return false;
            }
        } else {
            println!("*** Settings file found.");
        }
        true
    }
}
