//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

#[derive(Copy, Clone, PartialEq)]
pub struct Beat(pub i32, pub i32); // 分子/分母

pub const DEFAULT_TICK_FOR_QUARTER: i32 = 480;
pub const DEFAULT_TICK_FOR_ONE_MEASURE: i32 = 1920; // 480 * 4

pub const END_OF_DATA: i32 = -1;
pub const NO_DATA: i32 = -1;
pub const FULL: i32 = 10000;
pub const _ALL_PART: i16 = -1;
pub const KEEP: i32 = 0;
pub const LAST: i32 = 10000;

pub const NO_ROOT: i16 = 0; // root = 1:Ib,2:I,3:I# ...
pub const NO_TABLE: i16 = 10000;
pub const _CANCEL: i16 = -1;
pub const NOTHING: i16 = -1;

pub const MAX_INDICATOR: usize = 8;

//*******************************************************************
//          part count
//*******************************************************************
// Comp 2(L)+2(R), Phrase 2(L)+2(R), Pedal 1
pub const LEFT1: usize = 0;
pub const LEFT2: usize = 1;
pub const RIGHT1: usize = 2;
pub const RIGHT2: usize = 3;
pub const MAX_LEFT_PART: usize = 2;
pub const MAX_RIGHT_PART: usize = 2;
pub const MAX_KBD_PART: usize = MAX_LEFT_PART + MAX_RIGHT_PART;
pub const MAX_PHRASE: usize = 10; // normal + vari(1-9)
pub const DAMPER_PEDAL_PART: usize = MAX_KBD_PART;

//*******************************************************************
//          default value
//*******************************************************************
pub const DEFAULT_BPM: i16 = 100;
pub const DEFAULT_NOTE_NUMBER: u8 = 60; // C4
pub const MAX_NOTE_NUMBER: u8 = 108; // C8
pub const MIN_NOTE_NUMBER: u8 = 21; // A0
pub const NO_NOTE: u8 = 255;
pub const REST: u8 = 254;
pub const RPT_HEAD: u8 = 253; // Head of Repeat
pub const NO_MIDI_VALUE: u8 = 128;
pub const DEFAULT_TURNNOTE: i16 = 5;

//*******************************************************************
//          UI->ELPS Message
//              []: meaning, < >: index, (a/b/c): selection
//*******************************************************************
// MSG_PHR
/// for mtype
pub const TYPE_NONE: i16 = 0; // 共用
pub const _TYPE_ID: i16 = 1000; // for TYPE
pub const TYPE_NOTE: i16 = 1001; // for index TYPE
pub const TYPE_INFO: i16 = 1020; // タイミングを持つ演奏以外の情報
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct PhrEvt {
    pub mtype: i16, // message type
    pub tick: i16,
    pub dur: i16,  // duration
    pub note: i16, // note number / TYPE_INFO > RPT_HEAD
    pub vel: i16,  // velocity
    pub trns: i16, // translation
}
impl PhrEvt {
    pub fn gen_repeat(tick: i16) -> Self {
        Self {
            mtype: TYPE_INFO,
            tick: tick as i16,
            dur: 0,
            note: RPT_HEAD as i16,
            vel: 0,
            trns: TRNS_NONE,
        }
    }
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct PhrData {
    pub whole_tick: i16,
    pub auftakt: i16, // 0:no auftakt, 1-:beat begin auftakt
    pub do_loop: bool,
    pub evts: Vec<PhrEvt>,
}
impl PhrData {
    pub fn empty() -> Self {
        Self {
            whole_tick: 0,
            auftakt: 0,
            do_loop: true,
            evts: Vec::new(),
        }
    }
}
//-------------------------------------------------------------------
// MSG_CMP
/// for mtype
pub const TYPE_CHORD: i16 = 1002;
pub const TYPE_VARI: i16 = 1004;
pub const TYPE_CONTROL: i16 = 1007;
/// for tbl
pub const UPPER: i16 = 1000;
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ChordEvt {
    pub mtype: i16, // message type
    pub tick: i16,
    pub root: i16, // root note / TYPE_VARI: vari number
    pub tbl: i16,
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ChordData {
    pub whole_tick: i16,
    pub do_loop: bool,
    pub evts: Vec<ChordEvt>,
}
impl ChordData {
    pub fn empty() -> Self {
        Self {
            whole_tick: 0,
            do_loop: true,
            evts: Vec::new(),
        }
    }
}
//-------------------------------------------------------------------
// MSG_ANA
/// for mtype
pub const TYPE_BEAT: i16 = 1006; // for message TYPE
pub const TYPE_EXP: i16 = 1010; // for message TYPE
pub const _TYPE_DUR: i16 = 1012; // for message TYPE
/// mtype: TYPE_EXP のとき
/// atype
pub const NOPED: i16 = 10; // TYPE_BEAT の Note情報より先に置く
pub const PARA_ROOT: i16 = 12; // note に並行移動の基本rootの値を書く(0-11)
pub const ARTIC: i16 = 14; // cnt に Staccato/legato の長さを書く(1-200%)
/// mtype: TYPE_BEAT のとき
///   note: highest note,
///   cnt: same timing note number
/// atype、PhrEvt.trns, Arpeggio
pub const TRNS_COM: i16 = 0; // Common 変換
pub const TRNS_PARA: i16 = 10000; // Parallel 変換
pub const TRNS_NONE: i16 = 10001; // 変換しない
                                  //  -n .. +n  : ARP のときの Note 差分
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct AnaEvt {
    pub mtype: i16, // message type
    pub tick: i16,
    pub dur: i16,   // duration
    pub note: i16,  // note
    pub cnt: i16,   // value for something
    pub atype: i16, // type for something
}
impl AnaEvt {
    pub fn new() -> Self {
        Self {
            mtype: TYPE_NONE,
            tick: 0,
            dur: 0,
            note: 0,
            cnt: 0,
            atype: 0,
        }
    }
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct AnaData {
    pub evts: Vec<AnaEvt>,
}
impl AnaData {
    pub fn empty() -> Self {
        Self {
            evts: vec![AnaEvt::new()],
        }
    }
}
//-------------------------------------------------------------------
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct DmprEvt {
    pub mtype: i16, // message type
    pub tick: i16,
    pub dur: i16,      // duration
    pub position: i16, // damper position
}
impl DmprEvt {
    pub fn _new() -> Self {
        Self {
            mtype: TYPE_NONE,
            tick: 0,
            dur: 0,
            position: 0,
        }
    }
}
pub const TYPE_DAMPER: i16 = 1003;
//-------------------------------------------------------------------
#[derive(Clone, Debug)]
pub enum ElpsMsg {
    Ctrl(i16),
    Sync(i16),
    Rit([i16; 2]),
    Set([i16; 2]),
    //    SetBpm([i16; 3]),
    SetBeat([i16; 2]),
    //    SetKey([i16; 3]),
    Phr(i16, i16, PhrData), //  Phr : part, vari, (whole_tick,evts)
    Cmp(i16, ChordData),    //  Cmp : part, (whole_tick,evts)
    Ana(i16, i16, AnaData), //  Ana : part, vari, (evts)
    PhrX(i16, i16),         //  PhrX : part, vari
    CmpX(i16),              //  CmpX : part
    AnaX(i16, i16),         //  AnaX : part, vari
}
//  Ctrl
pub const MSG_CTRL_QUIT: i16 = -1;
pub const MSG_CTRL_START: i16 = -16; //  1byte msg
pub const MSG_CTRL_STOP: i16 = -15;
//pub const MSG_CTRL_FERMATA: i16  = -14;
pub const MSG_CTRL_PANIC: i16 = -13;
pub const MSG_CTRL_RESUME: i16 = -12;
pub const MSG_CTRL_FLOW: i16 = 100; // 100-104
pub const MSG_CTRL_ENDFLOW: i16 = 110;
//  Sync
// 0-4 : Part0-4
pub const MSG_SYNC_LFT: i16 = 5;
pub const MSG_SYNC_RGT: i16 = 6;
pub const MSG_SYNC_ALL: i16 = 7;
//  Rit : rit.を１小節以上かける場合、1byte目に [小節数*10] を足す
pub const MSG_RIT_NRM: i16 = 1;
pub const MSG_RIT_POCO: i16 = 2;
pub const MSG_RIT_MLT: i16 = 3;
pub const MSG2_RIT_ATMP: i16 = 9999;
pub const MSG2_RIT_FERMATA: i16 = 10000;
//  Set
pub const MSG_SET_BPM: i16 = 1;
pub const MSG_SET_KEY: i16 = 2;
pub const MSG_SET_TURN: i16 = 3;
//  Set BEAT  : numerator, denomirator

//*******************************************************************
//          Graphic
//*******************************************************************
// Graphic Message
pub const NO_MSG: i16 = -1;
pub const LIGHT_MODE: i16 = 1;
pub const DARK_MODE: i16 = 2;

//-------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InputMode {
    Fixed,  // 階名のオクターブ位置は固定。絶対位置を指定
    Closer, // 次の階名は近い方のオクターブを選択。遠い方を指示する場合、+/-を使う。
}

//*******************************************************************
//          Func
//*******************************************************************
pub fn convert_exp2vel(vel_text: &str) -> i32 {
    match vel_text {
        "ff" => 127,
        "f" => 114,
        "mf" => 100,
        "mp" => 84,
        "p" => 64,
        "pp" => 48,
        "ppp" => 24,
        "pppp" => 12,
        "ppppp" => 1,
        _ => END_OF_DATA,
    }
}
pub fn split_by(splitter: char, txt: String) -> Vec<String> {
    let mut splited: Vec<String> = Vec::new();
    let mut old_locate: usize = 0;
    for (i, ltr) in txt.chars().enumerate() {
        if ltr == splitter {
            splited.push((&txt[old_locate..i]).to_string());
            old_locate = i + 1;
        }
    }
    splited.push((&txt[old_locate..txt.len()]).to_string());
    splited
}
pub fn split_by_by(sp1: char, sp2: char, txt: String) -> Vec<String> {
    let mut splited: Vec<String> = Vec::new();
    let mut old_locate: usize = 0;
    for (i, ltr) in txt.chars().enumerate() {
        if ltr == sp1 || ltr == sp2 {
            splited.push((&txt[old_locate..i]).to_string());
            old_locate = i + 1;
        }
    }
    splited.push((&txt[old_locate..txt.len()]).to_string());
    splited
}
pub fn doremi_number(ltr: char, mut base_note: i32) -> i32 {
    match ltr {
        'd' => base_note += 0,
        'r' => base_note += 2,
        'm' => base_note += 4,
        'f' => base_note += 5,
        's' => base_note += 7,
        'l' => base_note += 9,
        't' => base_note += 11,
        _ => base_note = NO_NOTE as i32,
    }
    base_note
}
pub fn doremi_semi_number(ltr: char, mut base_note: i32) -> i32 {
    match ltr {
        'i' => base_note += 1,
        'a' => base_note -= 1,
        _ => (),
    }
    base_note
}
//*******************************************************************
//          extract_xxx_from_parentheses
//*******************************************************************
pub fn extract_number_from_parentheses(ne: &str) -> usize {
    if let Some(i) = ne.find('(') {
        if let Some(e) = ne.find(')') {
            if i < e {
                let numtxt = if i + 1 < e {
                    ne[(i + 1)..e].to_string()
                } else {
                    '1'.to_string()
                };
                return numtxt.parse().unwrap_or(0);
            } else {
                return 1;
            }
        }
    }
    0
}
pub fn extract_texts_from_parentheses(ne: &str) -> &str {
    if let Some(i) = ne.find('(') {
        if let Some(e) = ne.find(')') {
            if i <= e {
                return &ne[(i + 1)..e];
            }
        }
    }
    ""
}
