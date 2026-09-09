//  Created by Hasebe Masahiko on 2023/01/20.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;

use super::input_txt::CmndRtn;
use super::send_msg::*;
use super::seq_stock::*;
use super::txt2seq_ana::*;
use super::txt2seq_cmps::*;
use crate::common::lpnlib::*;
use crate::common::txt_common::*;
use crate::graphic::generative_view::{GraphicMsg, generate_graphic_msg};

#[derive(Debug, Clone)]
pub struct CmdReply {
    pub text: String,
    pub graphic: GraphicMsg,
}

impl CmdReply {
    fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            graphic: GraphicMsg::NoMsg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdError {
    InvalidInput,
    UnknownCommand,
    BadNumber,
    BadChannel,
    InvalidPart,
    Phrase(PhraseCmdError),
    Composition(CompositionCmdError),
}

pub type CmdResult = Result<CmdReply, CmdError>;

pub fn cmd_error_to_text(error: &CmdError) -> String {
    match error {
        CmdError::InvalidInput => "Invalid input.".to_string(),
        CmdError::UnknownCommand => "what?".to_string(),
        CmdError::BadNumber => "Number is wrong.".to_string(),
        CmdError::BadChannel => "Channel number is wrong.".to_string(),
        CmdError::InvalidPart => "what?".to_string(),
        CmdError::Phrase(PhraseCmdError::PendingMismatch) => {
            "Unfinished phrase input! Close it with a plain [...] first.".to_string()
        }
        CmdError::Phrase(_) => "what?".to_string(),
        CmdError::Composition(_) => "what?".to_string(),
    }
}

//*******************************************************************
//      Tokenizer / Classifier (pure, no state)
//*******************************************************************
/// 入力テキストの先頭文字によるコマンド種別分類
pub enum CmdKind {
    TogglePlay,                  // 単体の "."
    Slash,                       // "/" で始まる
    At,                          // "@" で始まる
    Bracket,                     // "[" で始まる
    Brace,                       // "{" で始まる
    PartSelect,                  // part 名単体による current part 切替
    PartWithPayload(Vec<usize>), // part 指定 + payload の形式
    OneWord,                     // トークン数 1
    MultiWord,                   // トークン数 2 以上
    Unknown,                     // 不明な形式
}

/// パート文字列を部番号に変換する純関数
pub fn detect_part(part_str: &str) -> Option<usize> {
    match part_str {
        "left1" | "L1" => Some(LEFT1),
        "left2" | "L2" => Some(LEFT2),
        "right1" | "R1" => Some(RIGHT1),
        "right2" | "R2" => Some(RIGHT2),
        "V1" => Some(VIOLIN1),
        "V2" => Some(VIOLIN2),
        _ => None,
    }
}
fn get_part_num(part_str: &str) -> Result<Vec<usize>, CmdError> {
    let part_num: Vec<usize> = match part_str {
        "L1" => vec![LEFT1],
        "L2" => vec![LEFT2],
        "L" => vec![LEFT1, LEFT2],
        "L1!" => vec![LEFT2, RIGHT1, RIGHT2],
        "L2!" => vec![LEFT1, RIGHT1, RIGHT2],
        "R1" => vec![RIGHT1],
        "R2" => vec![RIGHT2],
        "R" => vec![RIGHT1, RIGHT2],
        "R1!" => vec![LEFT1, LEFT2, RIGHT2],
        "R2!" => vec![LEFT1, LEFT2, RIGHT1],
        "FLOW" => vec![FLOW_PART],
        "D" | "DAMPER" => vec![DAMPER_PART],
        "SO" | "SOSTENUTO" => vec![SOSTENUTO_PART],
        "SH" | "SHIFT" => vec![SHIFT_PART],
        "V" => vec![VIOLIN1, VIOLIN2],
        "V1" => vec![VIOLIN1],
        "V2" => vec![VIOLIN2],
        "ALL" => vec![LEFT1, LEFT2, RIGHT1, RIGHT2, VIOLIN1, VIOLIN2],
        _ => return Err(CmdError::UnknownCommand),
    };
    Ok(part_num)
}

/// 入力テキストと token 列からコマンド種別を判定する純関数
pub fn classify_cmd(tokens: &[String]) -> CmdKind {
    if tokens.is_empty() {
        return CmdKind::Unknown;
    }
    let first = tokens
        .first()
        .and_then(|token| token.chars().next())
        .unwrap_or(' ');
    let token_count = tokens.len();
    let first_token = tokens.first().map(|token| token.as_str()).unwrap_or("");
    match first {
        '.' => CmdKind::TogglePlay,
        '/' => CmdKind::Slash,
        '@' => CmdKind::At,
        '[' => CmdKind::Bracket,
        '{' => CmdKind::Brace,
        _ if token_count >= 2 && matches!(first, 'L' | 'R' | 'F' | 'A' | 'D' | 'S' | 'V') => {
            if let Ok(part_num) = get_part_num(first_token) {
                CmdKind::PartWithPayload(part_num)
            } else {
                CmdKind::Unknown // 仮の値、実際の値は後で決定
            }
        }
        _ if detect_part(first_token).is_some() => CmdKind::PartSelect,
        _ if token_count == 1 => CmdKind::OneWord,
        _ => CmdKind::MultiWord,
    }
}

pub struct LoopianCmd {
    during_play: bool,
    indicator_key_stock: String,
    input_part: usize,
    path: Option<String>,
    pub dtstk: SeqDataStock,
    pub sndr: MessageSender,
}
impl LoopianCmd {
    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>) -> Self {
        Self {
            during_play: false,
            indicator_key_stock: "C".to_string(),
            input_part: RIGHT1,
            path: None,
            dtstk: SeqDataStock::new(),
            sndr: MessageSender::new(msg_hndr),
        }
    }
    pub fn get_indicator_key_stock(&self) -> String {
        self.indicator_key_stock.clone()
    }
    pub fn indicator_key_stock(&mut self, kstk: String) {
        self.indicator_key_stock = kstk;
    }
    pub fn get_input_part(&self) -> usize {
        self.input_part
    }
    pub fn get_path(&self) -> Option<String> {
        self.path.clone()
    }
    pub fn path(&mut self, path: String) {
        self.path = Some(path);
    }
    pub fn send_quit(&self) {
        self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Quit));
    }
    pub fn set_measure(&mut self, msr: i16) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Set(MsgSet::CurrentMsr(msr)));
    }
    pub fn send_clear(&self) {
        self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Clear));
        println!("*** All data has been erased at Elapse thread! ***");
    }
    pub fn set_riten(&mut self, percent: i16) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Rit(MsgRit::Riten(percent)));
    }
    #[cfg(feature = "raspi")]
    pub fn send_reconnect(&self) {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::MidiReconnect));
    }
    //*************************************************************************
    pub fn put_and_get_responce(&mut self, input_text: &str) -> Option<CmndRtn> {
        match self.execute_command(input_text) {
            Ok(reply) => Some(CmndRtn(reply.text, reply.graphic)),
            Err(CmdError::InvalidInput) => None,
            Err(error) => Some(CmndRtn(cmd_error_to_text(&error), GraphicMsg::NoMsg)),
        }
    }
    fn execute_command(&mut self, input_text: &str) -> CmdResult {
        if input_text.is_empty() || !input_text.is_ascii() {
            return Err(CmdError::InvalidInput);
        }
        let tokens = tokenize_cmd(input_text);
        println!("Analyzed Commands: {:?}", tokens);
        match classify_cmd(&tokens) {
            CmdKind::TogglePlay => Ok(CmdReply::text(self.cmd_dot())),
            CmdKind::Slash => Ok(CmdReply::text(self.letter_slash(input_text))),
            CmdKind::At => Ok(CmdReply::text(self.letter_at(input_text))),
            CmdKind::Bracket => self
                .dispatch_phrase(
                    false,
                    PhraseDest::Part(vec![self.input_part], PhraseAs::Normal),
                    tokens,
                )
                .map(CmdReply::text),
            CmdKind::Brace => self
                .apply_composition_to_part(self.input_part, tokens)
                .map(CmdReply::text),
            CmdKind::PartSelect => self.change_current_part(&tokens[0]).map(CmdReply::text),
            CmdKind::PartWithPayload(pn) => self.com_with_part(pn, tokens).map(CmdReply::text),
            CmdKind::OneWord => Ok(CmdReply::text(self.one_word_command(&tokens[0].clone()))),
            CmdKind::MultiWord => match tokens[0].as_str() {
                "set" => self
                    .parse_set_command_result(&tokens[1])
                    .map(CmdReply::text),
                "sync" => Ok(CmdReply::text(self.cmd_sync(&tokens[1]))),
                "clear" => Ok(CmdReply::text(self.cmd_clear(&tokens[1]))),
                "fine" => Ok(CmdReply::text(self.cmd_fine(&tokens[1]))),
                "help" => Ok(CmdReply::text(self.cmd_help(&tokens[1]))),
                "graph" => {
                    let rtn = generate_graphic_msg(tokens);
                    Ok(CmdReply {
                        text: rtn.0,
                        graphic: rtn.1,
                    })
                }
                "effect" => Ok(CmdReply::text(self.cmd_effect(&tokens[1]))),
                "rit" => Ok(CmdReply::text(self.cmd_rit(tokens[1..].to_vec()))),
                _ => Err(CmdError::UnknownCommand),
            },
            CmdKind::Unknown => Err(CmdError::UnknownCommand),
        }
    }
    fn one_word_command(&mut self, input_text: &str) -> String {
        match input_text {
            "hello" => self.cmd_hello(),
            "play" | "p" => self.cmd_play(),
            "panic" => self.cmd_panic(),
            "stop" | "end" => self.cmd_stop(),
            "sync" => self.cmd_sync(""),
            "clear" => self.cmd_clear(""),
            "fermata" => self.cmd_fermata(),
            "fine" => self.cmd_fine(""),
            "resume" => self.cmd_resume(),
            "reconnect" => self.cmd_reconnect(),
            "help" => self.cmd_help(""),
            "right1" => {
                self.input_part = RIGHT1;
                "Changed current part to right1.".to_string()
            }
            "right2" => {
                self.input_part = RIGHT2;
                "Changed current part to right2.".to_string()
            }
            "left1" => {
                self.input_part = LEFT1;
                "Changed current part to left1.".to_string()
            }
            "left2" => {
                self.input_part = LEFT2;
                "Changed current part to left2.".to_string()
            }
            "rit" | "rit." => self.cmd_rit(vec![]),
            _ => "what?".to_string(),
        }
    }
    fn cmd_clear(&mut self, input_part: &str) -> String {
        if input_part.is_empty() {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Stop));
            self.during_play = false;
            // clear
            for i in 0..MAX_KBD_PART {
                self.clear_part(i);
            }
            self.dtstk.clear_common_variation();
            self.send_clear();
            "all data erased!".to_string()
        } else if let Some(pnum) = detect_part(input_part) {
            println!("clear>>{input_part}");
            self.clear_part(pnum);
            match pnum {
                LEFT1 => "part L1 data erased!".to_string(),
                LEFT2 => "part L2 data erased!".to_string(),
                RIGHT1 => "part R1 data erased!".to_string(),
                RIGHT2 => "part R2 data erased!".to_string(),
                _ => "some part data erased!".to_string(),
            }
        } else if input_part == "env" {
            self.change_key("C");
            self.change_bpm(100);
            self.change_meter(4, 4);
            for i in 0..MAX_KBD_PART {
                self.change_oct("0", i);
            }
            "Environment data erased!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn cmd_effect(&mut self, parameter: &str) -> String {
        if parameter.contains("dmp(") {
            if let Some(dmp) = extract_number_from_parentheses(parameter) {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Efct(MsgEfct::Dmp(dmp as i16)));
                format!("Set Damper Value: {}", dmp)
            } else {
                "No Value!".to_string()
            }
        } else if parameter.contains("cc70(") {
            if let Some(cc70) = extract_number_from_parentheses(parameter) {
                self.sndr
                    .send_msg_to_elapse(ElpsMsg::Efct(MsgEfct::Cc70(cc70 as i16)));
                format!("Set CC70 Value: {}", cc70)
            } else {
                "No Value!".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    fn cmd_fermata(&mut self) -> String {
        self.sndr.send_msg_to_elapse(ElpsMsg::Rit(MsgRit::Fermata));
        "Will stop!".to_string()
    }
    fn cmd_fine(&mut self, input_next: &str) -> String {
        if input_next.is_empty() {
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Fine));
        } else if input_next == "next" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::FineNext2Bar));
        } else if input_next == "beat(2)" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::FineNext2Beat));
        } else if input_next == "beat(3)" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::FineNext3Beat));
        } else if input_next == "beat(4)" {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::FineNext4Beat));
        }
        "Fine.".to_string()
    }
    fn cmd_play(&mut self) -> String {
        if !self.during_play {
            // play
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Start));
            self.during_play = true;
            "Phrase has started!".to_string()
        } else {
            "Playing now!".to_string()
        }
    }
    fn cmd_panic(&mut self) -> String {
        // panic
        self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Panic));
        "All Sound Off!".to_string()
    }
    fn cmd_resume(&mut self) -> String {
        if !self.during_play {
            // resume
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Resume));
            self.during_play = true;
            "Resume.".to_string()
        } else {
            "Playing now!".to_string()
        }
    }
    fn cmd_reconnect(&mut self) -> String {
        self.sndr
            .send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::MidiReconnect));
        "Send reconnect".to_string()
    }
    fn cmd_stop(&mut self) -> String {
        if self.during_play {
            // stop
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Stop));
            self.during_play = false;
            "Stopped!".to_string()
        } else {
            "Settle down!".to_string()
        }
    }
    fn cmd_sync(&mut self, part_text: &str) -> String {
        if part_text.is_empty() {
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Sync(MsgSync::Part(self.input_part as i16)));
            "Synchronized!".to_string()
        } else if part_text == "right" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MsgSync::Right));
            "Right Part Synchronized!".to_string()
        } else if part_text == "left" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MsgSync::Left));
            "Left Part Synchronized!".to_string()
        } else if part_text == "all" {
            self.sndr.send_msg_to_elapse(ElpsMsg::Sync(MsgSync::All));
            "All Part Synchronized!".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn cmd_hello(&mut self) -> String {
        "Hi, hello!".to_string()
    }
    fn cmd_help(&mut self, input_text: &str) -> String {
        if input_text.is_empty() {
            "How can I assist you?".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_slash(&mut self, input_text: &str) -> String {
        let len = input_text.chars().count();
        if len >= 2 && &input_text[0..2] == "//" {
            "....Gotcha.".to_string()
        } else {
            "what?".to_string()
        }
    }
    fn letter_at(&mut self, input_text: &str) -> String {
        let split_txt = split_by('=', input_text.to_string());
        if split_txt.len() == 2 {
            let len = split_txt[0].chars().count();
            if len >= 4 && &input_text[0..4] == "@msr" {
                let msr;
                if let Some(m) = extract_number_from_parentheses(&split_txt[0]) {
                    msr = m;
                } else {
                    msr = 1;
                }
                match self.dispatch_phrase(
                    true,
                    PhraseDest::Part(vec![self.input_part], PhraseAs::Measure(msr)),
                    tokenize_cmd(&split_txt[1]),
                ) {
                    Ok(text) => text,
                    Err(error) => cmd_error_to_text(&error),
                }
            } else if len == 2 {
                let ltr = split_txt[0].chars().nth(1).unwrap_or('x');
                let vari = ltr.to_digit(10).unwrap_or(0);
                if ltr == 'c' {
                    self.dtstk.set_cluster_memory(split_txt[1].to_string());
                    "Set a cluster memory!".to_string()
                } else if vari > 0 {
                    match self.dispatch_phrase(
                        true,
                        PhraseDest::CommonVariation(vari as usize),
                        tokenize_cmd(&split_txt[1]),
                    ) {
                        Ok(text) => text,
                        Err(error) => cmd_error_to_text(&error),
                    }
                } else {
                    "what?".to_string()
                }
            } else {
                "what?".to_string()
            }
        } else {
            "what?".to_string()
        }
    }
    fn cmd_dot(&mut self) -> String {
        if self.during_play {
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Stop));
            self.during_play = false;
            "Stopped!".to_string()
        } else {
            self.sndr.send_msg_to_elapse(ElpsMsg::Ctrl(MsgCtrl::Start));
            self.during_play = true;
            "Phrase has started!".to_string()
        }
    }
    fn change_current_part(&mut self, part_str: &str) -> Result<String, CmdError> {
        if let Some(pnum) = detect_part(part_str) {
            self.input_part = pnum;
            Ok(format!("Changed current part to {}.", get_part_txt(pnum)))
        } else {
            Err(CmdError::InvalidPart)
        }
    }
    fn com_with_part(
        &mut self,
        part_num: Vec<usize>,
        msg_vec: Vec<String>,
    ) -> Result<String, CmdError> {
        let rest_vec = msg_vec[1..].to_vec();
        let first_letter = rest_vec[0].chars().next().unwrap_or('~');

        // FLOW パートは、ショートカットではなく、専用のコマンド形式で処理する
        if part_num.len() == 1 && part_num[0] == FLOW_PART {
            return Ok(self.flow_part_command(rest_vec));
        }

        let mut rtn = Err(CmdError::UnknownCommand);
        match first_letter {
            '[' => {
                // FLOW パートが含まれている場合は、FLOW パートを除いたパートに対してフレーズコマンドを適用する
                let proper_part: Vec<usize> = if part_num.contains(&FLOW_PART) {
                    part_num
                        .iter()
                        .filter(|&&p| p != FLOW_PART)
                        .cloned()
                        .collect()
                } else {
                    part_num
                };
                // 複数パート一括指定でも []+ の解決処理を1回だけ通す
                // (パート数だけループすると、同じ入力が多重にバッファへ
                // 連結されてしまうため)
                rtn = self.dispatch_phrase(
                    true,
                    PhraseDest::Part(proper_part, PhraseAs::Normal),
                    rest_vec,
                );
            }
            '{' => {
                for &pnum in &part_num {
                    rtn = self.apply_composition_to_part(pnum, rest_vec.clone());
                }
            }
            _ => {
                if rest_vec[0].contains("dyn") {
                    for &pnum in &part_num {
                        let mut dt = PhrData::empty();
                        dt.ana = put_amp_data(&rest_vec);
                        let msg = ElpsMsg::Phr(pnum as i16, dt);
                        self.sndr.send_msg_to_elapse(msg);
                    }
                    return Ok("Set Amp Analysis!".to_string());
                } else if rest_vec[0].contains("stacc") || rest_vec[0].contains("legato") {
                    for &pnum in &part_num {
                        let mut dt = PhrData::empty();
                        dt.ana = crispy_tick(&rest_vec);
                        let msg = ElpsMsg::Phr(pnum as i16, dt);
                        self.sndr.send_msg_to_elapse(msg);
                    }
                    return Ok("Set Articulation Analysis!".to_string());
                } else {
                    return Err(CmdError::UnknownCommand);
                }
            }
        }
        rtn
    }
    fn flow_part_command(&mut self, msg_vec: Vec<String>) -> String {
        if &msg_vec[0][0..1] == "{" {
            self.apply_composition_to_part(FLOW_PART, msg_vec)
                .unwrap_or_else(|error| cmd_error_to_text(&error))
        } else if msg_vec[0].contains("dyn") {
            let dyntxt = extract_texts_from_parentheses(&msg_vec[0]);
            let vel = if dyntxt.is_empty() {
                0
            } else {
                convert_expstr2amp(dyntxt)
            };
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set(MsgSet::FlowVelocity(vel)));
            "Flow Velocity Changed!".to_string()
        } else if msg_vec[0].contains("static") {
            let chord_txt = extract_texts_from_parentheses(&msg_vec[0]);
            let (_root, table) = convert_chord_to_num(chord_txt.to_string());
            self.sndr
                .send_msg_to_elapse(ElpsMsg::Set(MsgSet::FlowStaticScale(table)));
            "Flow Static Scale Changed!".to_string()
        } else {
            "what?".to_string()
        }
    }
    /// []+ の保留/継続/差し替え/エラー判定を経て、確定した入力を宛先ごとに
    /// ディスパッチする。素の `[...]`、`Part.[...]`、`@n=[...]`、
    /// `@msr(M)=[...]` のいずれもこの関数を通す(`explicit` はどの経路から
    /// 来たかを表す。素の `[...]` のみ false)。
    fn dispatch_phrase(
        &mut self,
        explicit: bool,
        dest: PhraseDest,
        tokens: Vec<String>,
    ) -> Result<String, CmdError> {
        match self.dtstk.resolve_or_buffer_phrase(explicit, dest, tokens) {
            PendingResult::Buffered => Ok("Keep Phrase as being unified phrase!".to_string()),
            PendingResult::Replaced => {
                Ok("Discarded unfinished phrase input, started a new one!".to_string())
            }
            PendingResult::Mismatch => Err(CmdError::Phrase(PhraseCmdError::PendingMismatch)),
            PendingResult::Resolved(PhraseDest::Part(parts, vari), combined) => {
                for part in parts {
                    self.apply_resolved_phrase(part, vari.clone(), combined.clone())?;
                }
                Ok("Set Phrase!".to_string())
            }
            PendingResult::Resolved(PhraseDest::CommonVariation(n), combined) => {
                self.dtstk.set_common_variation(n, combined);
                Ok("Set Variation Phrase!".to_string())
            }
        }
    }
    /// 解決済みのフレーズテキストを1パート分だけ格納し、elapse へ送信する。
    fn apply_resolved_phrase(
        &mut self,
        part_num: usize,
        vari: PhraseAs,
        msg_vec: Vec<String>,
    ) -> Result<(), CmdError> {
        self.dtstk
            .apply_phrase_data(part_num, vari.clone(), msg_vec)
            .map_err(CmdError::Phrase)?;
        if part_num < MAX_KBD_PART {
            self.sndr.send_phrase_to_elapse(part_num, vari, &self.dtstk);
        } else if (DAMPER_PART..=SHIFT_PART).contains(&part_num) {
            self.sndr.send_pedal_to_elapse(part_num, &self.dtstk);
        } else if (VIOLIN1..=VIOLIN2).contains(&part_num) {
            self.sndr.send_phrase_to_elapse(part_num, vari, &self.dtstk);
        }
        Ok(())
    }
    fn apply_composition_to_part(
        &mut self,
        part_num: usize,
        msg_vec: Vec<String>,
    ) -> Result<String, CmdError> {
        let vari_refs = self
            .dtstk
            .set_raw_composition(part_num, msg_vec)
            .map_err(CmdError::Composition)?;
        self.sndr.send_composition_to_elapse(part_num, &self.dtstk);
        // Composition が @n を参照していれば、共有 Variation ストアの内容を
        // このパート向けに同期し、このタイミングで初めて elapse へ送信する。
        for vari in vari_refs {
            if self.dtstk.sync_common_variation_to_part(part_num, vari) {
                self.sndr
                    .send_phrase_to_elapse(part_num, PhraseAs::Variation(vari), &self.dtstk);
            }
        }
        Ok("Set Composition!".to_string())
    }
    fn clear_part(&mut self, part_num: usize) {
        // seq stock のデータを消去
        self.dtstk.del_raw_phrase(part_num);

        // Phrase を消去する message を送る
        self.sndr.clear_phrase_to_elapse(part_num);

        if self
            .dtstk
            .set_raw_composition(part_num, vec!["{}".to_string()])
            .is_ok()
        {
            self.sndr.send_composition_to_elapse(part_num, &self.dtstk);
        }
        self.dtstk.change_oct(0, true, part_num);
    }
    fn cmd_rit(&mut self, mut input_text: Vec<String>) -> String {
        let mut aft_rit: RitAfter = RitAfter::Atmp;
        let mut strength: RitStrength = RitStrength::Nrm;
        let mut bar_num: i16 = 0;

        while !input_text.is_empty() {
            if input_text[0].chars().any(|x| x == '(') {
                if let Some((cmd, prm)) = separate_cmnd_and_str(&input_text[0]) {
                    if cmd == "bar" {
                        bar_num = prm.parse::<i16>().unwrap_or(0);
                        if bar_num >= 1 {
                            // 入力値は、内部値より1大きい
                            bar_num -= 1;
                        }
                    } else if cmd == "bpm" {
                        if let Ok(tmp) = prm.parse::<i16>() {
                            aft_rit = RitAfter::Bpm(tmp);
                        } else {
                            return "Number is wrong.".to_string();
                        }
                    }
                }
            } else if input_text[0] == "molto" {
                strength = RitStrength::Mlt;
            } else if input_text[0] == "poco" {
                strength = RitStrength::Poco;
            } else if input_text[0] == "fermata" {
                aft_rit = RitAfter::Fermata;
            } else if input_text[0] == "fine" {
                aft_rit = RitAfter::Fine;
            }
            input_text.remove(0);
        }

        println!(
            "Rit,strength:{:?}, bar:{}, after:{:?}",
            strength, bar_num, aft_rit
        );
        self.sndr.send_msg_to_elapse(ElpsMsg::Rit(MsgRit::Strength {
            strength,
            bar: bar_num,
            after: aft_rit,
        }));

        if aft_rit == RitAfter::Fine {
            "rit. has started and will be ended!".to_string()
        } else {
            "rit. has started!".to_string()
        }
    }
    /// ElapseStack が実際に stop したタイミングで、UiMsg::TickUi 経由の
    /// 通知を受けて during_play を同期させる
    pub fn set_during_play(&mut self, playing: bool) {
        self.during_play = playing;
    }
}
