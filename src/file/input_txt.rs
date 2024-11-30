//  Created by Hasebe Masahiko on 2024/11/02.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use nannou::prelude::*;
use std::sync::mpsc;

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
}
impl InputText {
    const CURSOR_MAX_VISIBLE_LOCATE: usize = 65;

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
    pub fn set_and_responce(&mut self, input_text: &str) -> Option<CmndRtn> {
        self.cmd.set_and_responce(input_text)
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
    pub fn window_event(&mut self, event: Event, graphmsg: &mut Vec<i16>) {
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
    fn key_pressed(&mut self, key: &Key, graphmsg: &mut Vec<i16>) {
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
                    self.set_graphic_msg(TEXT_VISIBLE_CTRL, graphmsg);
                }
            }
            _ => {}
        }
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
        } else if *ltr == '[' {
            self.input_text.insert(self.input_locate, ']');
        } else if *ltr == '{' {
            self.input_text.insert(self.input_locate, '}');
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
    fn pressed_enter(&mut self, graphmsg: &mut Vec<i16>) {
        let itxt = self.input_text.clone();
        if itxt.is_empty() {
            return;
        }
        self.input_text = "".to_string();
        self.input_locate = 0;
        self.visible_locate = 0;
        let len = itxt.chars().count();
        let chr = itxt.chars().nth(0).unwrap_or(' ');
        if chr != '!' {
            // Normal Input
            let msg = self.one_command(get_crnt_date_txt(), itxt, true);
            self.set_graphic_msg(msg, graphmsg);
        } else if (len == 2 && &itxt[0..2] == "!q") || (len >= 5 && &itxt[0..5] == "!quit") {
            // The end of the App
            self.cmd.send_quit();
            self.gen_log(0, "".to_string());
            println!("That's all. Thank you!");
            std::process::exit(0);
        } else if (len >= 2 && &itxt[0..2] == "!l") || (len >= 5 && &itxt[0..5] == "!load") {
            // Load File
            self.load_file(&itxt[0..], graphmsg);
        } else if (len >= 6 && &itxt[0..6] == "!clear")
            || (len >= 4 && &itxt[0..4] == "!clr")
            || (len >= 2 && &itxt[0..2] == "!c")
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
        } else if (len >= 2 && &itxt[0..2] == "!r")
            || (len >= 3 && &itxt[0..3] == "!rd")
            || (len >= 5 && &itxt[0..5] == "!read")
        {
            let num;
            if let Some(n) = extract_number_from_parentheses(itxt.as_str()) {
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
    }
    fn load_file(&mut self, itxt: &str, graphmsg: &mut Vec<i16>) {
        let blk_exists = |fnm: String| -> (Option<String>, Option<usize>) {
            let mut ltr = None;
            let mut num = None;
            if fnm.contains("blk") {
                ltr = Some(extract_texts_from_parentheses(fnm.as_str()).to_string());
            } else if fnm.contains("msr") {
                if let Some(e) = extract_number_from_parentheses(fnm.as_str()) {
                    num = Some(e);
                }
            }
            (ltr, num) // blk命令があるか調べ、あった場合は () 内の文字列取得
        };

        let mut blk: Option<String> = None;
        let mut msr: Option<usize> = None;
        let fname;
        let fnx = split_by('.', itxt.to_string());
        let fn_ele_num = fnx.len();
        if fn_ele_num >= 3 {
            (blk, msr) = blk_exists(fnx[2].clone());
            fname = fnx[1].clone();
            self.file_name_stock = fname.clone(); // file名を保存しておく
        } else if fn_ele_num == 2 {
            (blk, msr) = blk_exists(fnx[1].clone());
            if blk.is_none() && msr.is_none() {
                // !l.nnn はファイル名と考える
                fname = fnx[1].clone();
                self.file_name_stock = fname.clone(); // file名を保存しておく
            } else {
                // ファイル名は省略
                fname = self.file_name_stock.clone(); // 保存したfile名を使用する
            }
        } else {
            // "!l" だけ（ファイル名は省略）
            fname = self.file_name_stock.clone(); // 保存したfile名を使用する
        }

        if self
            .history
            .load_lpn(fname, self.cmd.get_path().as_deref(), blk)
        {
            // load_lpn() でファイルを読み込み、
            // get_loaded_text() で一行ずつ Scroll Text に入れていく
            let mut mt: CrntMsrTick = CrntMsrTick::default();
            if let Some(msr_num) = msr {
                // msr_num: 1origin
                let msr0ori = if msr_num > 0 { (msr_num as i16) - 1 } else { 0 };
                self.cmd.set_measure(msr0ori);
                mt.msr = msr_num as i32;
            }
            self.next_msr_tick = self.get_loaded_text(mt, graphmsg);
        } else {
            // 適切なファイルや中身がなかった場合
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "No history".to_string(),
            ));
        }
    }
    /// Auto Load  called from main::update()
    pub fn auto_load_command(&mut self, guiev: &GuiEv, graphmsg: &mut Vec<i16>) {
        if let Some(nmt) = self.next_msr_tick {
            let crnt: CrntMsrTick = guiev.get_msr_tick();
            if nmt.msr != LAST
                && nmt.msr > 0
                && nmt.msr - 1 == crnt.msr  // 一つ前の小節(両方とも1origin)
                && crnt.tick_for_onemsr - crnt.tick < 240
            {
                self.next_msr_tick = self.get_loaded_text(nmt, graphmsg);
            }
        }
    }
    fn get_loaded_text(&mut self, mt: CrntMsrTick, graphmsg: &mut Vec<i16>) -> Option<CrntMsrTick> {
        let loaded = self.history.get_loaded_text(mt);
        for onecmd in loaded.0.iter() {
            let msg = self.one_command(get_crnt_date_txt(), onecmd.clone(), false);
            self.set_graphic_msg(msg, graphmsg);
        }
        self.scroll_lines.push((
            TextAttribute::Answer,
            "".to_string(),
            "Loaded from designated file".to_string(),
        ));
        loaded.1
    }
    fn clear_loaded_data(&mut self) {
        self.file_name_stock = String::new();
        self.next_msr_tick = None;
    }
    fn one_command(&mut self, time: String, itxt: String, verbose: bool) -> i16 {
        // 通常のコマンド入力
        if let Some(answer) = self.cmd.set_and_responce(&itxt) {
            // normal command
            self.history_cnt = self
                .history
                .set_scroll_text(get_crnt_date_txt(), itxt.clone()); // input history
            self.scroll_lines
                .push((TextAttribute::Common, time.clone(), itxt.clone())); // for display text
            if verbose {
                self.scroll_lines
                    .push((TextAttribute::Answer, "".to_string(), answer.0));
            }
            return answer.1;
        }
        NO_MSG
    }
    fn set_graphic_msg(&mut self, msg: i16, graphmsg: &mut Vec<i16>) {
        graphmsg.push(msg);
    }
}
