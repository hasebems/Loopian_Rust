//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

#[derive(Copy, Clone, PartialEq)]
pub struct Beat(pub u32, pub u32); // 分子/分母

pub const DEFAULT_TICK_FOR_QUARTER: i32 = 480;
pub const DEFAULT_TICK_FOR_ONE_MEASURE: i32 = 1920;  // 480 * 4

pub const END_OF_DATA: i32 = -1;
pub const FULL: i32 = 10000;
pub const _ALL_PART: u16 = 0xffff;
pub const _KEEP: u16 = 0;
pub const _CANCEL: u16 = 0xffff;

//=====================
// part count
//=====================
// Comp 2(L)+2(R), Phrase 2(L)+2(R), Pedal 1 
pub const _LEFT1: usize = 0;
pub const _LEFT2: usize = 1;
pub const RIGHT1: usize = 2;
pub const _RIGHT2: usize = 3;
pub const MAX_LEFT_PART: usize = 2;
pub const MAX_RIGHT_PART: usize = 2;
pub const MAX_USER_PART: usize = MAX_LEFT_PART+MAX_RIGHT_PART;

pub const _FIRST_COMPOSITION_PART: usize = 0;
pub const MAX_COMPOSITION_PART: usize = MAX_USER_PART; // Normal と対応する同数のパート

pub const FIRST_PHRASE_PART: usize = MAX_COMPOSITION_PART;
pub const MAX_PHRASE_PART: usize = MAX_USER_PART;       //Composition と対応

pub const _DAMPER_PEDAL_PART: usize = MAX_COMPOSITION_PART+MAX_PHRASE_PART;
pub const ALL_PART_COUNT: usize = MAX_COMPOSITION_PART+MAX_PHRASE_PART+1;

//=====================
// default value
//=====================
pub const DEFAULT_BPM: u32 = 100;
pub const DEFAULT_NOTE_NUMBER: u8 = 60;    // C4
pub const NO_NOTE: u8 = 255;
pub const _DEFAULT_VEL: u8 = 100;

//=====================
// UI->ELPS Message
//=====================
pub const MSG_QUIT: u16     = 0xffff;
pub const MSG_START:u16     = 0xfff0;
pub const MSG_STOP: u16     = 0xfff8;
pub const MSG_SET:  u16     = 0xfff9;
pub const MSG2_BPM: u16     = 0x0001;
pub const MSG2_BEAT: u16    = 0x0002;

pub const MSG_PART_MASK: u16 = 0xfff0;
pub const MSG_PHR: u16      = 0x1000;   // 1桁目にパート番号
pub const _MSG_CMP: u16     = 0x2000;   // 1桁目にパート番号

pub const TYPE: usize       = 0;
pub const _TYPE_ID: u16     = 0xf000;
pub const TYPE_NOTE: u16    = 0xf001;
pub const TYPE_DAMPER: u16  = 0xf002;
pub const TICK: usize       = 1;
pub const DURATION: usize   = 2;
pub const NOTE: usize       = 3;
pub const VELOCITY: usize   = 4;

//=====================
// Enum
//=====================
#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum InputMode {
    _Fixed,  // 階名のオクターブ位置は固定。絶対位置を指定
    Closer, // 次の階名は近い方のオクターブを選択。遠い方を指示する場合、+/-を使う。
}