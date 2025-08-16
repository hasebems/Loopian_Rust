//  Created by Hasebe Masahiko on 2024/11/02.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use nannou::prelude::*;
use std::sync::mpsc;

use super::cnv_file;
use super::history::History;
use crate::cmd::cmdparse::*;
use crate::cmd::txt_common::*;
use crate::elapse::tickgen::CrntMsrTick;
use crate::graphic::guiev::GuiEv;
use crate::lpnlib::*;

//*******************************************************************
//      Input Text
//*******************************************************************
pub struct InputText {
    input_text: String,
    input_locate: usize,
    visible_locate: usize,
    history_cnt: usize,
    file_name_stock: String,
    next_msr_tick: Option<CrntMsrTick>,
    scroll_lines: Vec<(TextAttribute, String, String)>,
    history: History,
    cmd: LoopianCmd,
    shift_pressed: bool,
    ctrl_pressed: bool,
    just_after_hokan: bool,
}
impl InputText {
    const CURSOR_MAX_VISIBLE_LOCATE: usize = 65;
    const COMMAND_INPUT_REST_TICK: i32 = 240;

    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>) -> Self {
        Self {
            input_text: "".to_string(),
            input_locate: 0,
            visible_locate: 0,
            history_cnt: 0,
            file_name_stock: String::new(),
            next_msr_tick: None,
            scroll_lines: vec![],
            history: History::new(),
            cmd: LoopianCmd::new(msg_hndr),
            shift_pressed: false,
            ctrl_pressed: false,
            just_after_hokan: false,
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
    pub fn get_input_text(&self) -> String {
        self.input_text[self.visible_locate..].to_string()
    }
    pub fn get_scroll_lines(&self) -> &Vec<(TextAttribute, String, String)> {
        &self.scroll_lines
    }
    #[cfg(feature = "raspi")]
    pub fn send_reconnect(&self) {
        self.cmd.send_reconnect();
    }
    pub fn window_event(&mut self, event: Event, graphmsg: &mut Vec<GraphicMsg>) {
        match event {
            Event::WindowEvent {
                simple: Some(WindowEvent::ReceivedCharacter(c)),
                ..
            } => {
                // 制御文字（例: バックスペース）を除外
                if !c.is_control() && ((c != ' ') || !self.shift_pressed) {
                    self.input_letter(&c);
                }
            }
            Event::WindowEvent {
                simple: Some(WindowEvent::KeyPressed(key)),
                ..
            } => {
                self.key_pressed(&key, graphmsg);
                //println!("Key pressed: {:?}", key);
            }
            Event::WindowEvent {
                simple: Some(WindowEvent::KeyReleased(key)),
                ..
            } => {
                self.key_released(&key);
                //println!("Key released: {:?}", key);
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
                    self.update_visible_locate();
                    if self.just_after_hokan {
                        self.input_text.remove(self.input_locate);
                        self.update_visible_locate();
                    }
                }
            }
            &Key::Left => {
                if self.shift_pressed {
                    self.input_locate = 0;
                } else if self.input_locate > 0 {
                    self.input_locate -= 1;
                }
                self.update_visible_locate();
            }
            &Key::Right => {
                let maxlen = self.input_text.chars().count();
                if self.shift_pressed {
                    self.input_locate = maxlen;
                } else {
                    self.input_locate += 1;
                }
                self.update_visible_locate();
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
                    self.visible_locate = 0;
                }
            }
            &Key::Down => {
                if self.input_locate == 0 {
                    if let Some(txt) = self.history.arrow_down() {
                        self.input_text = txt.0;
                        self.history_cnt = txt.1;
                    }
                    self.input_locate = 0;
                    self.visible_locate = 0;
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
        if key == &Key::LShift || key == &Key::RShift {
            self.shift_pressed = false;
        } else
        /*if key == &Key::LControl*/
        {
            // カーソルKeyに使うと Ctrl Released が反応しないため
            self.ctrl_pressed = false;
        }
    }
    fn input_letter(&mut self, ltr: &char) {
        self.input_text.insert(self.input_locate, *ltr);
        self.input_locate += 1;
        self.update_visible_locate();
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
    fn update_visible_locate(&mut self) {
        if self.input_locate >= Self::CURSOR_MAX_VISIBLE_LOCATE {
            self.visible_locate = self.input_locate - Self::CURSOR_MAX_VISIBLE_LOCATE;
        } else if self.input_locate < self.visible_locate {
            self.visible_locate = self.input_locate;
        }
    }
    pub fn get_cursor_locate(&self) -> usize {
        if self.input_locate > Self::CURSOR_MAX_VISIBLE_LOCATE {
            Self::CURSOR_MAX_VISIBLE_LOCATE
        } else {
            self.input_locate
        }
    }
    fn pressed_enter(&mut self, graphmsg: &mut Vec<GraphicMsg>) {
        let itxt = self.input_text.clone();
        if itxt.is_empty() {
            return;
        }
        self.input_text = "".to_string();
        self.input_locate = 0;
        self.visible_locate = 0;
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
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "All data cleared!".to_string(),
            ));
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
            self.history
                .get_loaded_blk(blk_name)
                .iter()
                .for_each(|txt| {
                    self.one_command(txt.clone(), graphmsg, false);
                });
        } else if len >= 5 && &itxt[0..5] == "!msr(" {
            // measure の読み込み
            if let Some(msr_num) = extract_number_from_parentheses(&itxt[0..]) {
                self.load_by_msr_command(msr_num, graphmsg);
            }
        } else if len >= 7 && &itxt[0..7] == "!cnv2tl" {
            // convert to timeline file
            self.convert_to_timeline_file(&itxt[0..]);
        }
    }
    fn clear_loaded_data(&mut self) {
        self.file_name_stock = String::new();
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
        if let Some(cmd) = self.history.read_line_from_lpn(
            self.file_name_stock.clone(),
            self.cmd.get_path().as_deref(),
            num,
        ) {
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
    /// ファイルをロードして、一行ずつ処理する
    fn load_file(&mut self, itxt: &str, graphmsg: &mut Vec<GraphicMsg>, playable: bool) {
        let fnx = split_by('.', itxt.to_string());
        if fnx.len() >= 2 {
            self.file_name_stock = fnx[1].clone();
        }

        // load_lpn() でファイルの中身を読み込み、
        // send_loaddata_to_elapse() で一行ずつ再生し Scroll Text に入れていく
        if self
            .history
            .load_lpn(self.file_name_stock.clone(), self.cmd.get_path().as_deref())
        {
            let loaded = self.history.get_from_mt_to_next(CrntMsrTick::default());
            self.next_msr_tick = self.send_loaddata_to_elapse(graphmsg, playable, loaded);
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
        // send_loaddata_to_elapse() で一行ずつ Scroll Text に入れていく
        let mt: CrntMsrTick = CrntMsrTick {
            msr: msr as i32,
            ..Default::default()
        };
        let loaded = self.history.get_from_0_to_mt(mt);
        if loaded.0.is_empty() {
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "No Data!".to_string(),
            ));
            return;
        }
        self.next_msr_tick = self.send_loaddata_to_elapse(graphmsg, true, loaded);
        let msr0ori = if msr > 0 { (msr as i16) - 1 } else { 0 };
        self.cmd.set_measure(msr0ori);
        if self.next_msr_tick.is_some() {
            //#[cfg(feature = "verbose")]
            println!(
                "@@@ Load by msr: {}, next_msr_tick: {:?}",
                msr,
                self.next_msr_tick.unwrap()
            );
        }
    }
    /// Auto Load  called from main::update()
    pub fn auto_load_command(&mut self, guiev: &GuiEv, graphmsg: &mut Vec<GraphicMsg>) {
        if let Some(mt) = self.next_msr_tick {
            let crnt: CrntMsrTick = guiev.get_msr_tick();
            if mt.msr != LAST
                && mt.msr > 0
                && mt.msr - 1 == crnt.msr
                && crnt.tick_for_onemsr - crnt.tick < Self::COMMAND_INPUT_REST_TICK
            {
                let loaded = self.history.get_from_mt_to_next(mt);
                self.next_msr_tick = self.send_loaddata_to_elapse(graphmsg, true, loaded);
            }
        }
    }
    /// ロードされたファイルの内容を再生する
    fn send_loaddata_to_elapse(
        &mut self,
        graphmsg: &mut Vec<GraphicMsg>,
        playable: bool,
        loaded: (Vec<String>, Option<CrntMsrTick>),
    ) -> Option<CrntMsrTick> {
        for (i, onecmd) in loaded.0.iter().enumerate() {
            if playable {
                self.one_command(onecmd.clone(), graphmsg, false);
            } else {
                let time = format!("  >> History: {:05} ", i);
                self.set_history(time, onecmd.clone(), None);
            }
        }
        self.scroll_lines.push((
            TextAttribute::Answer,
            "".to_string(),
            "Loaded from designated file".to_string(),
        ));
        loaded.1
    }
    /// 一行分のコマンド入力
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
