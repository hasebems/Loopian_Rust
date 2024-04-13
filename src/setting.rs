//  Created by Hasebe Masahiko on 2024/04/13
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

// Default Window Size
pub const WINDOW_X_DEFAULT: f32 = 1000.0; //  Main Window
pub const WINDOW_Y_DEFAULT: f32 = 860.0;

// Font Data File Name with path


// MIDI Connection
pub const MIDI_OUT: &str = "IACdriver";
//pub const MIDI_OUT: &str = "Midi Through:Midi Through Port-0 14:0"; // for Raspi5
pub const MIDI_DEVICE: &str = "Pico";
//pub const MIDI_DEVICE: &str = "Loopian-ORBIT";
//pub const MIDI_DEVICE: &str = "Arduino Leonardo";         // Arduino によるチェック
//pub const MIDI_DEVICE: &str = "TouchMIDI32 MIDI OUT";     // for Mac
//pub const MIDI_DEVICE: &str = "IACdriver InternalBus1"; // MAX によるチェック for Mac
