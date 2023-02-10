//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

#[derive(Copy, Clone, PartialEq)]
pub struct Beat(pub u32, pub u32); // 分子/分母

pub const DEFAULT_TICK_FOR_QUARTER: u32 = 480;
pub const DEFAULT_TICK_FOR_ONE_MEASURE: u32 = 1920;  // 480 * 4

//=====================
// part count
//=====================
pub const MAX_LEFT_PART: usize = 2;
pub const MAX_RIGHT_PART: usize = 2;
pub const MAX_PART: usize = MAX_LEFT_PART+MAX_RIGHT_PART;

pub const FIRST_COMPOSITION_PART: usize = 0;
pub const MAX_COMPOSITION_PART: usize = MAX_PART; // Normal と対応する同数のパート

pub const FIRST_PHRASE_PART: usize = MAX_COMPOSITION_PART;
pub const MAX_PHRASE_PART: usize = MAX_PART;       //Composition と対応

pub const DAMPER_PEDAL_PART: usize = MAX_COMPOSITION_PART+MAX_PHRASE_PART;
pub const MAX_PART_COUNT: usize = MAX_COMPOSITION_PART+MAX_PHRASE_PART+1;

//=====================
// default value
//=====================
pub const DEFAULT_BPM: u32 = 100;
pub const DEFAULT_NOTE_NUMBER: u8 = 60;    // C4
pub const NO_NOTE: u8 = 255;
pub const DEFAULT_VEL: u8 = 100;

//=====================
// UI->ELPS Message
//=====================
pub const MSG_QUIT: u16 = 0xffff;
pub const MSG_START: u16 = 0xfff0;
pub const MSG_STOP: u16 = 0xfff8;
pub const MSG_SET:  u16 = 0xfff9;
pub const MSG2_BPM:  u16 = 0x0001;
