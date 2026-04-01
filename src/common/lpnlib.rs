//  Created by Hasebe Masahiko on 2023/01/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

#[derive(Copy, Clone, PartialEq)]
pub struct Meter(pub i32, pub i32); // 分子(numerator)/分母(denominator)

pub const DEFAULT_TICK_FOR_QUARTER: i32 = 480;
pub const DEFAULT_TICK_FOR_ONE_MEASURE: i32 = 1920; // 480 * 4
pub const TICK_4_4: f32 = (DEFAULT_TICK_FOR_QUARTER * 4) as f32;
pub const TICK_3_4: f32 = (DEFAULT_TICK_FOR_QUARTER * 3) as f32;

pub const END_OF_DATA: i32 = -1;
pub const NO_DATA: i32 = -1;
pub const FULL: i32 = 10000;
pub const _ALL_PART: i16 = -1;
pub const _KEEP: i32 = 0;
pub const LAST: i32 = 10000;

pub const NO_ROOT: i16 = 0; // root = 1:Ib,2:I,3:I# ...
pub const NO_TABLE: i16 = 10000;
pub const NO_PED_TBL_NUM: i16 = 0; // 'X'
pub const _CANCEL: i16 = -1;
pub const NOTHING: i16 = -1;

pub const MAX_PATTERN_NUM: u8 = 16; // Max Pattern Number

//*******************************************************************
//          part count
//*******************************************************************
pub enum PtName {
    Left1 = 0,
    Left2 = 1,
    Right1 = 2,
    Right2 = 3,
    Flow = 4,
    Damper = 5,
    Sostenuto = 6,
    Shift = 7,
}
// Comp 2(L)+2(R), Phrase 2(L)+2(R), Flow 1, Pedal 3(Damper,Sostenuto,Shift)
pub const LEFT1: usize = PtName::Left1 as usize;
pub const LEFT2: usize = PtName::Left2 as usize;
pub const RIGHT1: usize = PtName::Right1 as usize;
pub const RIGHT2: usize = PtName::Right2 as usize;
pub const MAX_LEFT_PART: usize = LEFT2 + 1;
pub const MAX_RIGHT_PART: usize = RIGHT2 - LEFT2;
pub const MAX_KBD_PART: usize = MAX_LEFT_PART + MAX_RIGHT_PART;
pub const MAX_COMPOSITION_PART: usize = MAX_KBD_PART + 1;
pub const MAX_VARIATION: usize = 10; // normal + vari(1-9) + 1(for measure)
pub const FLOW_PART: usize = PtName::Flow as usize; // Flow part
pub const MAX_ALL_KBD_PART: usize = PtName::Flow as usize + 1; // Flow part included
pub const DAMPER_PART: usize = PtName::Damper as usize; // Elapse inside Part
pub const SOSTENUTO_PART: usize = PtName::Sostenuto as usize; // Elapse inside Part
pub const SHIFT_PART: usize = PtName::Shift as usize; // Elapse inside Part
pub const MAX_PEDAL_PART: usize = PtName::Shift as usize + 1 - PtName::Damper as usize; // Damper, Sostenuto, Shift
pub const NONE_NUM: usize = 255;

//*******************************************************************
//          default value
//*******************************************************************
pub const DEFAULT_BPM: i16 = 100;
pub const DEFAULT_NOTE_NUMBER: u8 = 60; // C4
pub const MAX_NOTE_NUMBER: u8 = 108; // C8
pub const MIN_NOTE_NUMBER: u8 = 21; // A0
pub const NO_NOTE: u8 = 255;
pub const INVALID: u8 = 255;
pub const REST: u8 = 254;
pub const RPT_HEAD: u8 = 253; // Head of Repeat
pub const NO_MIDI_VALUE: u8 = 128;
pub const DEFAULT_TURNNOTE: i16 = 5;
pub const DEFAULT_ARTIC: i16 = 100;

//*******************************************************************
//          Parameter Type
//*******************************************************************
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TrnsType {
    #[default]
    Com, // TRNS_COM: Common 変換
    Para,     // TRNS_PARA: Parallel 変換
    Arp(i16), // ARP: Arpeggio 変換, -n .. +n  : Note 差分
    NoTrns,   // 変換しない
}
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpType {
    // TYPE_EXP のときの atype
    #[default]
    Noped, // TYPE_BEAT の Note情報より先に置く
    ParaRoot, // note に並行移動の基本rootの値を書く(0-11)
    Artic,    // value に Staccato/legato の長さを書く(1-200%)
    Amp,      // value に Auto Amp の値を書く(-16..0..+16)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FineType {
    NextBar,       // 次の小節の頭で終了
    NextBeat(i32), // 次の小節の頭からN拍目で終了
    Next2Bar,      // 次の次の小節の頭で終了
    WaitForBeat(i32),
    WaitFor2Bar,
}

//*******************************************************************
//          UI->ELPS Message
//              []: meaning, < >: index, (a/b/c): selection
//*******************************************************************
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amp {
    pub note_amp: i16, // note amplitude -16..0..+16
    // -16: %%%%, -12: %%%, -8: %%, -4: %, 0: --, +4: ^, +8: ^^, +12: ^^^, +16: ^^^^
    pub auto_amp: i16, // auto amplitude -16..0..+16
}
impl Amp {
    pub fn default() -> Self {
        Self {
            note_amp: 0,
            auto_amp: 0,
        }
    }
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct NoteListEvt {
    pub tick: i16,      // tick
    pub dur: i16,       // duration
    pub notes: Vec<u8>, // note number
    pub floating: bool, // true: floating tick, false: not floating
    pub amp: Amp,       // amplitude
    pub trns: TrnsType, // translation
    pub artic: i16,     // 0..100..200[%] staccato/legato
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct NoteEvt {
    pub tick: i16,      // tick
    pub dur: i16,       // duration
    pub note: u8,       // note number
    pub floating: bool, // true: floating tick, false: not floating
    pub amp: Amp,       // amplitude
    pub trns: TrnsType, // translation
    pub artic: i16,     // 0..100..200[%] staccato/legato
}
impl NoteEvt {
    pub fn from_note_list(list: &NoteListEvt, note: u8) -> Self {
        Self {
            tick: list.tick,
            dur: list.dur,
            note,
            floating: list.floating,
            amp: list.amp,
            trns: list.trns,
            artic: list.artic,
        }
    }
}
//-------------------------------------------------------------------
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct BrkPatternEvt {
    pub tick: i16,      // tick
    pub dur: i16,       // duration
    pub lowest: i16,    // lowest note number -7..0..7
    pub amp: Amp,       // amplitude
    pub max_count: i16, // max note count: 2-5
    pub figure: i16,    // figure of arpeggio: u/d/xu/xd(0-3)
    pub each_dur: i16,  // each note's duration
    pub artic: i16,     // 0..100..200[%] staccato/legato
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ClsPatternEvt {
    pub tick: i16,      // tick
    pub dur: i16,       // duration
    pub lowest: i16,    // lowest note number -7..0..7
    pub amp: Amp,       // amplitude
    pub max_count: i16, // max note count: 2-5
    pub arpeggio: i16,  // figure of arpeggio: 0,1-3
    pub each_dur: i16,  // each note's duration
    pub artic: i16,     // 0..100..200[%] staccato/legato
}
//-------------------------------------------------------------------
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct InfoEvt {
    pub tick: i16, // tick
    pub dur: i16,  // duration
    pub info: i16, // RPT_HEAD
}
impl InfoEvt {
    pub fn gen_repeat(tick: i16) -> Self {
        Self {
            tick,
            dur: 0,
            info: RPT_HEAD as i16, // default is repeat head
        }
    }
}
//-------------------------------------------------------------------
#[derive(Default, Clone, Debug, PartialEq, Eq, Copy)]
pub enum PedalPos {
    #[default]
    NoEvt, // no event
    Off,
    Half,
    Full,
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum PedalType {
    #[default]
    Damper,
    Sostenuto,
    Shift,
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct PedalEvt {
    pub msr: i16, // measure number
    pub beat: i16,
    pub front: bool,        // true: front of beat, false: back of beat
    pub position: PedalPos, // pedal position
}

//-------------------------------------------------------------------
//      Phrase Event
//-------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhrEvt {
    Note(NoteEvt),
    NoteList(NoteListEvt),
    BrkPtn(BrkPatternEvt),
    ClsPtn(ClsPatternEvt),
    Damper(PedalEvt),
    Sostenuto(PedalEvt),
    Shift(PedalEvt),
    Info(InfoEvt),
}
impl PhrEvt {
    pub fn dur(&self) -> i16 {
        match self {
            PhrEvt::Note(e) => e.dur,
            PhrEvt::NoteList(e) => e.dur,
            PhrEvt::BrkPtn(e) => e.dur,
            PhrEvt::ClsPtn(e) => e.dur,
            PhrEvt::Info(e) => e.dur,
            _ => 0,
        }
    }
    pub fn set_dur(&mut self, dur: i16) {
        match self {
            PhrEvt::Note(e) => e.dur = dur,
            PhrEvt::NoteList(e) => e.dur = dur,
            PhrEvt::BrkPtn(e) => e.dur = dur,
            PhrEvt::ClsPtn(e) => e.dur = dur,
            PhrEvt::Info(e) => e.dur = dur,
            _ => {}
        }
    }
    pub fn tick(&self) -> i16 {
        match self {
            PhrEvt::Note(e) => e.tick,
            PhrEvt::NoteList(e) => e.tick,
            PhrEvt::BrkPtn(e) => e.tick,
            PhrEvt::ClsPtn(e) => e.tick,
            PhrEvt::Info(e) => e.tick,
            _ => 0,
        }
    }
    pub fn set_tick(&mut self, tick: i16) {
        match self {
            PhrEvt::Note(e) => e.tick = tick,
            PhrEvt::NoteList(e) => e.tick = tick,
            PhrEvt::BrkPtn(e) => e.tick = tick,
            PhrEvt::ClsPtn(e) => e.tick = tick,
            PhrEvt::Info(e) => e.tick = tick,
            _ => {}
        }
    }
    pub fn set_artic(&mut self, artic: i16) {
        match self {
            PhrEvt::Note(e) => e.artic = artic,
            PhrEvt::NoteList(e) => e.artic = artic,
            PhrEvt::BrkPtn(e) => e.artic = artic,
            PhrEvt::ClsPtn(e) => e.artic = artic,
            _ => {}
        }
    }
    pub fn note(&self) -> u8 {
        match self {
            PhrEvt::Note(e) => e.note,
            PhrEvt::NoteList(e) => e.notes[e.notes.len() - 1], // last note
            //    PhrEvt::BrkPtn(e) => e.note,
            //    PhrEvt::ClsPtn(e) => e.note,
            //    PhrEvt::Info(e) => e.tick,
            _ => NO_NOTE,
        }
    }
    pub fn set_note(&mut self, note: u8) {
        if let PhrEvt::Note(e) = self {
            e.note = note;
            //    PhrEvt::NoteList(e) => e.set_note(note),
            //    PhrEvt::BrkPtn(e) => e.set_note(note),
            //    PhrEvt::ClsPtn(e) => e.set_note(note),
            //    PhrEvt::Info(e) => e.tick = tick,
        }
    }
}

//-------------------------------------------------------------------
//      MSG_ANA
//-------------------------------------------------------------------
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct AnaBeatEvt {
    pub tick: i16,
    pub dur: i16,  // duration
    pub note: i16, // highest note
    pub cnt: i16,  // same timing note number
    pub trns: TrnsType,
    // Com, Para, NoTrns,
    // Arp: -n .. +n ARP のときの Note 差分
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct AnaExpEvt {
    pub tick: i16,
    pub dur: i16,  // duration
    pub note: i16, // note
    pub value: i16,
    pub atype: ExpType,
    // Noped: TYPE_BEAT の Note情報より先に置く
    // ParaRoot: note に並行移動の基本rootの値を書く(0-11)
    // Artic: value に Staccato/legato の長さを書く(1-200%)
    // Amp: value に Auto Amp の値を書く(-16..0..+16)
    // -16: pppp, -12: ppp, -8: pp, -4: p, 0: mp, +4: mf, +8: f, +12: ff, +16: fff
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnaEvt {
    Beat(AnaBeatEvt),
    Exp(AnaExpEvt),
}
impl AnaEvt {
    pub fn tick(&self) -> i16 {
        match self {
            AnaEvt::Beat(e) => e.tick,
            AnaEvt::Exp(e) => e.tick,
        }
    }
    pub fn set_tick(&mut self, tick: i16) {
        match self {
            AnaEvt::Beat(e) => e.tick = tick,
            AnaEvt::Exp(e) => e.tick = tick,
        }
    }
}
pub fn get_noped(ana: &[AnaEvt]) -> bool {
    ana.iter().any(|x| {
        if let AnaEvt::Exp(e) = x {
            e.atype == ExpType::Noped
        } else {
            false
        }
    })
}
pub fn get_para(ana: &[AnaEvt]) -> bool {
    ana.iter().any(|x| {
        if let AnaEvt::Exp(e) = x {
            e.atype == ExpType::ParaRoot
        } else {
            false
        }
    })
}
pub fn get_para_root_base(ana: &[AnaEvt]) -> i16 {
    let mut para_root_base = 0;
    ana.iter().for_each(|x| match x {
        AnaEvt::Exp(e) if e.atype == ExpType::ParaRoot => {
            para_root_base = e.note;
        }
        _ => (),
    });
    para_root_base
}
pub fn get_artic_rate(ana: &[AnaEvt], shorter: bool) -> i32 {
    let mut artic_rate = if shorter { 90 } else { 100 };
    ana.iter().for_each(|x| match x {
        AnaEvt::Exp(e) if e.atype == ExpType::Artic => {
            artic_rate = e.value as i32;
        }
        _ => (),
    });
    artic_rate
}
pub fn get_amp(ana: &[AnaEvt]) -> i16 {
    let mut phrase_amp = 0;
    ana.iter().for_each(|x| match x {
        AnaEvt::Exp(e) if e.atype == ExpType::Amp => {
            phrase_amp = e.value;
        }
        _ => (),
    });
    phrase_amp
}

//-------------------------------------------------------------------
//      Phrase DATA
//-------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PhraseAs {
    #[default]
    Normal,
    Variation(usize), // 1..9:variation
    Measure(usize),   // 1..:measure number
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct PhrData {
    pub whole_tick: i16,
    pub do_loop: bool,
    pub evts: Vec<PhrEvt>,
    pub ana: Vec<AnaEvt>,
    pub vari: PhraseAs,
    pub auftakt: i16, // 0:no auftakt, 1..:auftakt(beat number)
}
impl PhrData {
    pub fn empty() -> Self {
        Self {
            whole_tick: 0,
            do_loop: true,
            evts: Vec::new(),
            ana: Vec::new(),
            vari: PhraseAs::Normal,
            auftakt: 0,
        }
    }
}
//-------------------------------------------------------------------
//      Composition DATA
//-------------------------------------------------------------------
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ChordEvt {
    pub tick: i16,
    pub root: i16, // root note
    pub tbl: i16,
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct VariEvt {
    pub tick: i16,
    pub vari: i16, // vari number
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ControlEvt {
    pub tick: i16,
    pub ctrl: i16, // control number
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CmpEvt {
    Chord(ChordEvt),
    Vari(VariEvt),
    Control(ControlEvt),
}
impl CmpEvt {
    pub fn tick(&self) -> i16 {
        match self {
            CmpEvt::Chord(e) => e.tick,
            CmpEvt::Vari(e) => e.tick,
            CmpEvt::Control(e) => e.tick,
        }
    }
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CmpData {
    pub whole_tick: i16,
    pub do_loop: bool,
    pub evts: Vec<CmpEvt>,
    // how to start
    pub measure: i16, // NOTHING: no effect, 1..:measure number
}
impl CmpData {
    pub fn empty() -> Self {
        Self {
            whole_tick: 0,
            do_loop: true,
            evts: Vec::new(),
            measure: NOTHING,
        }
    }
}

//*******************************************************************
//          Elapse Message Definition
//*******************************************************************
//  ElpsMsg <- Phr *- PhrData *- PhrEvt <- NoteEvt / PedalEvt / ...
//          <- Cmp *- CmpData *- CmpEvt <- Chord / Vari / Control
#[derive(Clone, Debug)]
pub enum ElpsMsg {
    Ctrl(i16),
    Sync(i16),
    Rit([i16; 2]),
    Set([i16; 2]),
    Efct([i16; 2]),
    SetMeter([i16; 2]),
    SetElasticity([i16; 2]),
    Phr(i16, PhrData),      //  Phr : part, (whole_tick,evts)
    PhrX(i16),              //  PhrX : part
    Cmp(i16, CmpData),      //  Cmp : part, (whole_tick,evts)
    CmpX(i16),              //  CmpX : part
    MIDIRx(u8, u8, u8, u8), //  status, dt1, dt2, extra
}
//  Ctrl
pub const MSG_CTRL_QUIT: i16 = -1;
pub const MSG_CTRL_START: i16 = -16; //  1byte msg
pub const MSG_CTRL_STOP: i16 = -15;
pub const MSG_CTRL_FINE: i16 = -14; // 次の小節の頭で終了
pub const MSG_CTRL_FINE_NEXT_2BAR: i16 = -21; // 次の小節の頭の次の小節の頭で終了
pub const MSG_CTRL_FINE_NEXT_2BEAT: i16 = -22; // 次の小節の頭から２拍目で終了
pub const MSG_CTRL_FINE_NEXT_3BEAT: i16 = -23; // 次の小節の頭から３拍目で終了
pub const MSG_CTRL_FINE_NEXT_4BEAT: i16 = -24; // 次の小節の頭から４拍目で終了
pub const MSG_CTRL_PANIC: i16 = -13;
pub const MSG_CTRL_RESUME: i16 = -12;
pub const MSG_CTRL_CLEAR: i16 = -11; // Elapse Objectの内容をクリア
pub const MSG_CTRL_MIDI_RECONNECT: i16 = -10;
pub const _MSG_CTRL_FLOW: i16 = 100; // 100-104
pub const _MSG_CTRL_ENDFLOW: i16 = 110;
//  Sync
// 0-4 : Part0-4
pub const MSG_SYNC_LFT: i16 = 5;
pub const MSG_SYNC_RGT: i16 = 6;
pub const MSG_SYNC_ALL: i16 = 7;
//  Rit : rit.を１小節以上かける場合、1byte目に [小節数*10] を足す（コマンドは10以下のみ）
pub const MSG_RIT_NRM: i16 = 1;
pub const MSG_RIT_POCO: i16 = 2;
pub const MSG_RIT_MLT: i16 = 3;
pub const MSG_RIT_RITEN: i16 = 8; // 2byte目: 現状のテンポへの倍率: 90..10..0..-10..-90[%]
pub const MSG2_RIT_ATMP: i16 = 9999;
pub const MSG2_RIT_FERMATA: i16 = 10000;
//  Set
pub const MSG_SET_BPM: i16 = 1;
pub const MSG_SET_KEY: i16 = 2;
pub const MSG_SET_TURN: i16 = 3;
pub const MSG_SET_CRNT_MSR: i16 = 4; // RESUME と一緒に使う
pub const MSG_SET_FLOW_TICK_RESOLUTION: i16 = 5; // Flow の Tick 解像度を設定
pub const MSG_SET_FLOW_VELOCITY: i16 = 6; // Flow の Velocity を設定
pub const MSG_SET_FLOW_STATIC_SCALE: i16 = 7; // Flow の Static Scale を設定
pub const MSG_SET_MIDI_INPUT_CH: i16 = 8; // Flow の MIDI Input Ch を設定
//  Set Meter  : numerator, denomirator
//  Set Elasticity : middle_rate, last_rate, middle_tick
//  Effect
pub const MSG_EFCT_DMP: i16 = 1;
pub const MSG_EFCT_CC70: i16 = 2;

//*******************************************************************
//          UI Message from Elapse thread
//*******************************************************************
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextAttribute {
    Common,
    Answer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteUiEv {
    pub key_num: u8,
    pub vel: u8,
    pub pt: u8,
}
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct PartUi {
    pub exist: bool,
    pub msr_in_loop: i32,
    pub all_msrs: i32,
    pub flow: bool,
    pub chord_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphicEv {
    NoteEv(NoteUiEv),
    BeatEv(i32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiMsg {
    NewMeasure,
    NewBeat(i32),
    BpmUi(i16, i16), // bpm, bpm_rate
    Meter(i32, i32),
    TickUi(bool, i32, i32, i32), //running, tick_in_beat, beat, msr
    PartUi(usize, PartUi),       // part_num
    NoteUi(NoteUiEv),
    ChangePtn(u8),
}
//*******************************************************************
//          Command Definition
//*******************************************************************
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InputMode {
    Fixed,    // 階名のオクターブ位置は固定。絶対位置を指定
    Closer,   // +/-がない時、次の階名は近い方のオクターブを選択。遠い方を指示する場合、+/-を使う。
    Upcloser, // +/-がない時、次の階名は前の音程から1オクターブ上を選択。遠い方を指示する場合、+/-を使う。
}
