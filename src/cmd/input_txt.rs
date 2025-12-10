//  Created by Hasebe Masahiko on 2024/11/02.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use nannou::prelude::*;
use std::sync::mpsc;

use super::cmdparse::*;
use super::txt_common::*;
use crate::elapse::tickgen::CrntMsrTick;
use crate::file::cnv_file;
use crate::file::history::*;
use crate::file::load::*;
use crate::graphic::generative_view::GraphicMsg;
use crate::graphic::guiev::GuiEv;
use crate::lpnlib::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputTextType {
    Phrase,   // [] で囲まれたデータ
    Realtime, // set, play, rit などのコマンド
    Any,      // 全て
}
#[derive(PartialEq, Eq)]
enum AutoLoadState {
    BeforeLoading,
    Reached,
    PhraseLoaded,
}
// return msg from command receiving job
pub struct CmndRtn(pub String, pub GraphicMsg);

//*******************************************************************
//      Input Text
//*******************************************************************
pub struct InputText {
    input_text: String,
    input_locate: usize,
    history_cnt: usize,
    next_msr_tick: Option<CrntMsrTick>,
    auto_load_buffer: (Vec<String>, Option<CrntMsrTick>),
    auto_load_state: AutoLoadState,
    scroll_lines: Vec<(TextAttribute, String, String)>,
    history: History, // 履歴管理モジュール
    load_buffer: LoadBuffer,
    cmd: LoopianCmd, // コマンド処理モジュール
    shift_pressed: bool,
    ctrl_pressed: bool,
    just_after_hokan: bool,
    riten_sent: bool,
}
impl InputText {
    //const CURSOR_MAX_VISIBLE_LOCATE: usize = 65;
    const COMMAND_INPUT_REST_TICK: i32 = 240;

    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>) -> Self {
        Self {
            input_text: "".to_string(),
            input_locate: 0,
            history_cnt: 0,
            next_msr_tick: None,
            auto_load_buffer: (vec![], None),
            auto_load_state: AutoLoadState::BeforeLoading,
            scroll_lines: vec![],
            history: History::new(),
            load_buffer: LoadBuffer::new(),
            cmd: LoopianCmd::new(msg_hndr),
            shift_pressed: false,
            ctrl_pressed: false,
            just_after_hokan: false,
            riten_sent: false,
        }
    }
    pub fn get_history_locate(&self) -> usize {
        self.history_cnt
    }
    pub fn gen_log(&mut self, num: usize, fname: String) {
        self.history.gen_log(num, fname);
    }
    pub fn get_input_part(&self) -> usize {
        self.cmd.get_input_part()
    }
    pub fn get_indicator_key_stock(&self) -> String {
        self.cmd.get_indicator_key_stock()
    }
    pub fn put_and_get_responce(&mut self, input_text: &str) -> Option<CmndRtn> {
        self.cmd.put_and_get_responce(input_text)
    }
    pub fn get_scroll_lines(&self) -> &Vec<(TextAttribute, String, String)> {
        &self.scroll_lines
    }
    pub fn get_input_text(
        &self,
        max_width_px: f32,
        char_width_px: f32,
    ) -> (Vec<String>, f32, usize) {
        // 折り返し優先文字
        const PREF_BREAK: &[char] = &[',', '.', ')'];

        let mut lines = Vec::new();
        let mut current = String::new();
        let mut last_break_idx: Option<usize> = None;
        let mut width = 0.0;

        for ch in self.input_text.chars() {
            let w = char_width_px;
            // 折り返し候補を記録
            if ch.is_whitespace() || PREF_BREAK.contains(&ch) {
                last_break_idx = Some(current.len());
            }
            // この文字を入れるとオーバー？
            if width + w > max_width_px && !current.is_empty() {
                let cut = last_break_idx.unwrap_or(current.len());
                let (push_part, remain_part) = current.split_at(cut);
                lines.push(push_part.trim_end().to_string());

                // 残りを次行に回す（必要ならリーダーを付与）
                let mut next = String::new();
                next.push_str(remain_part.trim_start());

                current = next;
                width = current.len() as f32 * char_width_px;
                last_break_idx = None;
            }

            current.push(ch);
            width += w;
        }

        if !current.is_empty() {
            lines.push(current);
        }

        let each_len = lines.iter().map(|line| line.len()).collect::<Vec<_>>();
        let mut cursor_locate = self.input_locate;
        let mut cursor_line = 0;
        while cursor_line < each_len.len() && cursor_locate > each_len[cursor_line] {
            cursor_locate -= each_len[cursor_line];
            cursor_line += 1;
        }
        (lines, cursor_locate as f32, cursor_line)
    }
    #[cfg(feature = "raspi")]
    pub fn send_reconnect(&self) {
        self.cmd.send_reconnect();
    }
    //*******************************************************************
    //          Window/Key Event
    //*******************************************************************
    pub fn window_event(&mut self, event: Event, graphmsg: &mut Vec<GraphicMsg>) {
        match event {
            Event::WindowEvent {
                simple: Some(WindowEvent::ReceivedCharacter(c)),
                ..
            } => {
                // 制御文字（例: バックスペース）を除外
                if self.exclusion_condition(&c) {
                    self.input_letter(&c);
                }
            }
            Event::WindowEvent {
                simple: Some(WindowEvent::KeyPressed(key)),
                ..
            } => {
                self.key_pressed(&key, graphmsg);
            }
            Event::WindowEvent {
                simple: Some(WindowEvent::KeyReleased(key)),
                ..
            } => {
                self.key_released(&key);
            }
            _ => {}
        }
    }
    fn key_pressed(&mut self, key: &Key, graphmsg: &mut Vec<GraphicMsg>) {
        match key {
            &Key::LShift | &Key::RShift => {
                self.shift_pressed = true;
            }
            &Key::LControl => {
                self.ctrl_pressed = true;
            }
            &Key::Return => {
                self.pressed_enter(graphmsg);
            }
            &Key::V => {
                // for ctrl+V
                if self.ctrl_pressed {
                    let mut ctx = ClipboardContext::new().unwrap();
                    let clip_text = ctx.get_contents().unwrap();
                    self.input_text += &clip_text;
                }
            }
            &Key::Back => {
                if self.input_locate > 0 {
                    self.input_locate -= 1;
                    self.input_text.remove(self.input_locate);
                    if self.just_after_hokan {
                        self.input_text.remove(self.input_locate);
                    }
                }
            }
            &Key::Left => {
                if self.shift_pressed {
                    self.input_locate = 0;
                } else if self.input_locate > 0 {
                    self.input_locate -= 1;
                }
            }
            &Key::Right => {
                let maxlen = self.input_text.chars().count();
                if self.shift_pressed {
                    self.input_locate = maxlen;
                } else {
                    self.input_locate += 1;
                }
                if self.input_locate > maxlen {
                    self.input_locate = maxlen;
                }
            }
            &Key::Up => {
                if self.input_locate == 0 {
                    if let Some(txt) = self.history.arrow_up() {
                        self.input_text = txt.0;
                        self.history_cnt = txt.1;
                    }
                    self.input_locate = 0;
                }
            }
            &Key::Down => {
                if self.input_locate == 0 {
                    if let Some(txt) = self.history.arrow_down() {
                        self.input_text = txt.0;
                        self.history_cnt = txt.1;
                    }
                    self.input_locate = 0;
                }
            }
            &Key::Key1
            | &Key::Key2
            | &Key::Key3
            | &Key::Key4
            | &Key::Key5
            | &Key::Key6
            | &Key::Key7
            | &Key::Key8
            | &Key::Key9 => {
                if self.ctrl_pressed {
                    let num = match key {
                        Key::Key1 => 1,
                        Key::Key2 => 2,
                        Key::Key3 => 3,
                        Key::Key4 => 4,
                        Key::Key5 => 5,
                        Key::Key6 => 6,
                        Key::Key7 => 7,
                        Key::Key8 => 8,
                        Key::Key9 => 9,
                        _ => 0,
                    };
                    if num > 0 {
                        self.cmd.set_riten(num * -10);
                        self.riten_sent = true;
                    }
                }
            }
            &Key::RControl => {}
            &Key::LAlt => {}
            &Key::RAlt => {}
            &Key::LWin => {}
            &Key::RWin => {}
            &Key::Space => {
                if self.shift_pressed {
                    graphmsg.push(GraphicMsg::TextVisibleCtrl);
                }
            }
            _ => {}
        }
        self.just_after_hokan = false;
    }
    fn key_released(&mut self, key: &Key) {
        match key {
            &Key::LShift | &Key::RShift => {
                self.shift_pressed = false;
            }
            &Key::Key1
            | &Key::Key2
            | &Key::Key3
            | &Key::Key4
            | &Key::Key5
            | &Key::Key6
            | &Key::Key7
            | &Key::Key8
            | &Key::Key9 => {
                if self.riten_sent {
                    self.cmd.set_riten(0); // Normal
                    self.riten_sent = false;
                }
            }
            _ => {
                // カーソルKeyに使うと Ctrl Released が反応しないため
                self.ctrl_pressed = false;
            }
        }
    }
    fn exclusion_condition(&self, c: &char) -> bool {
        !c.is_control() && // 制御文字を除外
        ((*c != ' ') || !self.shift_pressed) && // Shift+Space を除外
        !self.ctrl_pressed // Ctrl 押下中は除外
    }
    fn input_letter(&mut self, ltr: &char) {
        self.input_text.insert(self.input_locate, *ltr);
        self.input_locate += 1;
        // 括弧の補完
        if *ltr == '(' {
            self.input_text.insert(self.input_locate, ')');
            self.just_after_hokan = true;
        } else if *ltr == '[' {
            self.input_text.insert(self.input_locate, ']');
            self.just_after_hokan = true;
        } else if *ltr == '{' {
            self.input_text.insert(self.input_locate, '}');
            self.just_after_hokan = true;
        } else {
            self.just_after_hokan = false;
        }
        // space を . に変換
        if self.input_text.chars().any(|x| x == ' ') {
            let itx = self.input_text.clone();
            self.input_text = itx.replacen(' ', ".", 100); // egui とぶつかり replace が使えない
        }
    }
    /// 手入力で Enter キーが押された
    fn pressed_enter(&mut self, graphmsg: &mut Vec<GraphicMsg>) {
        let itxt = self.input_text.clone();
        if itxt.is_empty() {
            return;
        }
        self.input_text = "".to_string();
        self.input_locate = 0;

        self.set_command(itxt, graphmsg);
    }
    pub fn set_command(&mut self, itxt: String, graphmsg: &mut Vec<GraphicMsg>) {
        let chr = itxt.chars().nth(0).unwrap_or(' ');
        if chr != '!' {
            self.one_command(itxt, graphmsg, true);
        } else {
            self.non_logged_command(itxt, graphmsg);
        }
    }
    //*******************************************************************
    //          File 関連操作
    //*******************************************************************
    /// Log に記録しないコマンドを処理 (例: !q, !load, !clear)
    fn non_logged_command(&mut self, itxt: String, graphmsg: &mut Vec<GraphicMsg>) {
        let len = itxt.chars().count();
        if (len == 2 && &itxt[0..2] == "!q") || (len >= 5 && &itxt[0..5] == "!quit") {
            // The end of the App
            self.cmd.send_quit();
            self.gen_log(0, "".to_string());
            println!("That's all. Thank you!");
            std::process::exit(0);
        } else if (len >= 2 && &itxt[0..2] == "!l") || (len >= 5 && &itxt[0..5] == "!load") {
            // Load Command
            self.load_file(&itxt[0..], graphmsg, true);
        } else if (len >= 2 && &itxt[0..2] == "!h") || (len >= 8 && &itxt[0..8] == "!history") {
            // Load to history Command
            self.load_file(&itxt[0..], graphmsg, false);
        } else if (len == 6 && &itxt[0..6] == "!clear") // no parameter
            || (len == 4 && &itxt[0..4] == "!clr")
            || (len == 2 && &itxt[0..2] == "!c")
        {
            // clear loaded file data
            self.clear_loaded_data();
            self.cmd.send_clear();
            self.set_answer_line("All data cleared!".to_string());
        } else if (len >= 2 && &itxt[0..2] == "!s") || (len >= 5 && &itxt[0..5] == "!save") {
            // Save Command
            self.save_command(itxt);
        } else if (len >= 2 && &itxt[0..2] == "!r")
            || (len >= 3 && &itxt[0..3] == "!rd")
            || (len >= 5 && &itxt[0..5] == "!read")
        {
            // 一行のコマンドを読み込む
            self.read_oneline_command(&itxt[0..]);
        } else if len >= 5 && &itxt[0..5] == "!blk(" {
            // ブロックの読み込み
            let blk_name = extract_texts_from_parentheses(&itxt[0..]);
            let loaded_blk = self.load_buffer.get_loaded_blk(blk_name);
            if !loaded_blk.is_empty() {
                loaded_blk.iter().for_each(|txt| {
                    self.one_command(txt.clone(), graphmsg, false);
                });
            } else {
                self.set_answer_line("No such block.".to_string());
            }
        } else if len >= 5 && &itxt[0..5] == "!msr(" {
            // measure の読み込み
            if let Some(msr_num) = extract_number_from_parentheses(&itxt[0..]) {
                self.load_by_msr_command(msr_num, graphmsg);
            } else {
                self.set_answer_line("No such measure.".to_string());
            }
        } else if len >= 7 && &itxt[0..7] == "!cnv2tl" {
            // convert to timeline file
            self.convert_to_timeline_file(&itxt[0..]);
        } else if (len == 5 && &itxt[0..5] == "!play") || (len == 2 && &itxt[0..2] == "!p") {
            if self.load_buffer.get_file_name().is_some() {
                // ファイルを読み込んでいる場合、そのデータの冒頭から再生するようセッティングする
                self.auto_load_buffer = (vec![], None);
                self.auto_load_state = AutoLoadState::BeforeLoading;
                let loaded = self
                    .load_buffer
                    .get_from_msr_to_next(CrntMsrTick::default());
                self.send_loaddata_to_elapse(graphmsg, InputTextType::Any, true, loaded.0, Some(1));
                self.next_msr_tick = loaded.1;
            } else {
                self.set_answer_line("No file loaded".to_string());
            }
        } else {
            self.set_answer_line("Unknown command".to_string());
        }
    }
    /// Answer に出力する文字列をセットする
    fn set_answer_line(&mut self, answer: String) {
        self.scroll_lines
            .push((TextAttribute::Answer, "".to_string(), answer)); // for display answer
    }
    fn clear_loaded_data(&mut self) {
        self.auto_load_buffer = (vec![], None);
        self.auto_load_state = AutoLoadState::BeforeLoading;
        self.load_buffer.clear_file_name();
        self.next_msr_tick = None;
    }
    fn save_command(&mut self, itxt: String) {
        let itxts = split_by('.', itxt);
        let fname = if itxts.len() >= 2 {
            itxts[1].clone()
        } else {
            "".to_string()
        };
        let num;
        if let Some(n) = extract_number_from_parentheses(&itxts[0]) {
            num = n;
        } else {
            num = 0;
        }
        self.gen_log(num, fname);
        self.scroll_lines.push((
            TextAttribute::Answer,
            "".to_string(),
            "log saved!".to_string(),
        ));
    }
    fn read_oneline_command(&mut self, itxt: &str) {
        let num;
        if let Some(n) = extract_number_from_parentheses(itxt) {
            num = n;
        } else {
            num = 0;
        }
        if let Some(cmd) = self
            .load_buffer
            .read_line_from_lpn(self.cmd.get_path().as_deref(), num)
        {
            self.input_text = cmd;
        }
    }
    fn convert_to_timeline_file(&mut self, itxt: &str) {
        println!("Convert to Timeline File");
        let itxts = split_by('.', itxt.to_string());
        if itxts.len() >= 2 {
            cnv_file::convert_to_timeline(itxts[1].clone(), self.cmd.get_path().as_deref());
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "Converted to Timeline File!".to_string(),
            ));
        }
    }
    /// ファイルをロードする
    fn load_file(&mut self, itxt: &str, graphmsg: &mut Vec<GraphicMsg>, playable: bool) {
        let fnx = split_by('.', itxt.to_string());
        if fnx.len() >= 2 {
            self.load_buffer.set_file_name(fnx[1].clone());
        }

        // load_lpn() でファイルの中身を読み込む
        if let Some(file_name) = self.load_buffer.get_file_name() {
            if self.load_buffer.load_lpn(self.cmd.get_path().as_deref()) {
                if !playable {
                    // 履歴にのみロードする場合
                    self.clear_loaded_data();
                    let loaded = self
                        .load_buffer
                        .get_from_msr_to_next(CrntMsrTick::default());
                    self.send_loaddata_to_elapse(
                        graphmsg,
                        InputTextType::Any,
                        false,
                        loaded.0,
                        None,
                    );
                    self.next_msr_tick = loaded.1;
                }

                let answer_word = format!("Loaded from file: {}.lpn", file_name);
                self.scroll_lines
                    .push((TextAttribute::Answer, "".to_string(), answer_word));
            }
        } else {
            // 適切なファイルや中身がなかった場合
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "No File.".to_string(),
            ));
        }
    }
    /// !msr() で指定された小節までのデータをロード
    /// ここでは、指定された小節の直前までのデータをロード
    fn load_by_msr_command(&mut self, msr: usize, graphmsg: &mut Vec<GraphicMsg>) {
        let mt: CrntMsrTick = CrntMsrTick {
            msr: msr as i32,
            ..Default::default()
        };
        let loaded = self.load_buffer.get_from_0_to_mt(mt);
        if !loaded.0.is_empty() {
            self.send_loaddata_to_elapse(graphmsg, InputTextType::Realtime, true, loaded.0, None);
        } else {
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "No Data!".to_string(),
            ));
            return;
        }
        self.next_msr_tick = loaded.1;
        let loaded = self.load_buffer.get_from_msr(mt);
        if !loaded.0.is_empty() {
            self.send_loaddata_to_elapse(
                graphmsg,
                InputTextType::Any,
                true,
                loaded.0,
                Some(msr as i32),
            );
        }

        // 小節番号を設定する
        let msr0ori = if msr > 0 { (msr as i16) - 1 } else { 0 };
        self.cmd.set_measure(msr0ori);
        self.auto_load_buffer = (vec![], None);
        self.auto_load_state = AutoLoadState::BeforeLoading;
    }
    /// Auto Load  called from main::update()
    pub fn auto_load_command(&mut self, guiev: &GuiEv, graphmsg: &mut Vec<GraphicMsg>) {
        if let Some(next_mt) = self.next_msr_tick {
            let crnt: CrntMsrTick = guiev.get_msr_tick();
            if next_mt.msr != LAST && next_mt.msr > 0 && next_mt.msr - 1 == crnt.msr {
                // 指定された小節の１小節前まで来た場合
                if self.auto_load_state == AutoLoadState::BeforeLoading {
                    self.auto_load_buffer = self.load_buffer.get_from_msr_to_next(next_mt);
                    self.auto_load_state = AutoLoadState::Reached;
                } else if self.auto_load_state == AutoLoadState::Reached
                    && crnt.tick > Self::COMMAND_INPUT_REST_TICK
                {
                    // 1拍目の COMMAND_INPUT_REST_TICK 後に、フレーズを再生
                    let autoload = self.auto_load_buffer.clone();
                    self.send_loaddata_to_elapse(
                        graphmsg,
                        InputTextType::Phrase,
                        true,
                        autoload.0,
                        Some(next_mt.msr),
                    );
                    self.auto_load_state = AutoLoadState::PhraseLoaded;
                } else if self.auto_load_state == AutoLoadState::PhraseLoaded
                    && crnt.tick_for_onemsr - crnt.tick < Self::COMMAND_INPUT_REST_TICK
                {
                    // 小節終わりの COMMAND_INPUT_REST_TICK 前に、リアルタイムメッセージを再生
                    let autoload = self.auto_load_buffer.clone();
                    self.send_loaddata_to_elapse(
                        graphmsg,
                        InputTextType::Realtime,
                        true,
                        autoload.0,
                        None,
                    );
                    self.next_msr_tick = autoload.1;
                    self.auto_load_state = AutoLoadState::BeforeLoading;
                }
            }
        }
    }
    /// ロードされたファイルの内容を Elapse Engine に送る
    fn is_fitting_command(ttype: InputTextType, onecmd: &str) -> bool {
        let cnd = onecmd.chars().any(|c| c == '[')
            || onecmd.chars().any(|c| c == ']')
            || onecmd.contains("L1")
            || onecmd.contains("L2")
            || onecmd.contains("R1")
            || onecmd.contains("R2");
        match ttype {
            InputTextType::Phrase => cnd,
            InputTextType::Realtime => !cnd,
            _ => true,
        }
    }
    fn send_loaddata_to_elapse(
        &mut self,
        graphmsg: &mut Vec<GraphicMsg>,
        txt_type: InputTextType,
        playable: bool,
        loaded: Vec<String>,
        next_msr: Option<i32>,
    ) {
        let mut answer: bool = false;
        for (i, onecmd) in loaded.iter().enumerate() {
            if Self::is_fitting_command(txt_type, onecmd) {
                if playable {
                    self.one_command(onecmd.clone(), graphmsg, false);
                    answer = true;
                } else {
                    let time = format!(" >History: {:05} ", i);
                    self.set_history(time, onecmd.clone(), None);
                }
            }
        }
        if answer && next_msr.is_some() {
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                format!("Transferred to play measure {}", next_msr.unwrap_or(0)),
            ));
        }
    }
    //*******************************************************************
    //          General Task
    //*******************************************************************
    /// 一行分のコマンド入力（手入力＆ファイル入力）
    fn one_command(&mut self, itxt: String, graphmsg: &mut Vec<GraphicMsg>, verbose: bool) {
        let time = get_crnt_date_txt();
        let msg = if let Some(answer) = self.cmd.put_and_get_responce(&itxt) {
            let answer0 = if verbose { Some(&(answer.0)) } else { None };
            self.set_history(time, itxt, answer0);
            answer.1
        } else {
            GraphicMsg::NoMsg
        };
        graphmsg.push(msg);
    }
    /// 入力したコマンドを履歴に追加
    fn set_history(&mut self, time: String, itxt: String, answer: Option<&String>) {
        self.history_cnt = self.history.set_scroll_text(time.clone(), itxt.clone()); // input history
        self.scroll_lines.push((TextAttribute::Common, time, itxt)); // for display text
        if let Some(a) = answer {
            self.scroll_lines
                .push((TextAttribute::Answer, "".to_string(), a.to_string())); // for display answer
        }
    }
}
