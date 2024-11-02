//  Created by Hasebe Masahiko on 2024/11/02.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui::*;
use std::sync::mpsc;

use super::history::History;
use crate::cmd::cmdparse::*;
use crate::cmd::txt_common::*;
use crate::elapse::tickgen::CrntMsrTick;
use crate::graphic::graphic::Graphic;
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
        }
    }
    pub fn get_history_cnt(&self) -> usize {
        self.history_cnt
    }
    pub fn gen_log(&mut self) {
        self.history.gen_log();
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
    pub fn input_letter(&mut self, letters: Vec<&String>) {
        letters.iter().for_each(|ltr| {
            self.input_text.insert_str(self.input_locate, ltr);
            self.input_locate += 1;
            self.update_visible_locate();
        });
        // autofill
        if let Some(&ltr) = letters.last() {
            if ltr == "(" {
                self.input_text.insert_str(self.input_locate, ")");
            } else if ltr == "[" {
                self.input_text.insert_str(self.input_locate, "]");
            } else if ltr == "{" {
                self.input_text.insert_str(self.input_locate, "}");
            }
        }
        // space を . に変換
        if self.input_text.chars().any(|x| x == ' ') {
            let itx = self.input_text.clone();
            self.input_text = itx.replacen(' ', ".", 100); // egui とぶつかり replace が使えない
        }
    }
    pub fn pressed_key(&mut self, key: &Key, modifiers: &Modifiers, graph: &mut Graphic) {
        let itxt: String = self.input_text.clone();
        if key == &Key::Enter {
            self.pressed_enter(itxt, graph);
        } else if key == &Key::V {
            // for ctrl+V
            if modifiers.ctrl {
                let mut ctx = ClipboardContext::new().unwrap();
                let clip_text = ctx.get_contents().unwrap();
                self.input_text += &clip_text;
            }
        } else if key == &Key::Backspace {
            if self.input_locate > 0 {
                self.input_locate -= 1;
                self.input_text.remove(self.input_locate);
                self.update_visible_locate();
            }
        } else if key == &Key::ArrowLeft {
            if modifiers.shift {
                self.input_locate = 0;
            } else if self.input_locate > 0 {
                self.input_locate -= 1;
            }
            self.update_visible_locate();
        } else if key == &Key::ArrowRight {
            let maxlen = self.input_text.chars().count();
            if modifiers.shift {
                self.input_locate = maxlen;
            } else {
                self.input_locate += 1;
            }
            self.update_visible_locate();
            if self.input_locate > maxlen {
                self.input_locate = maxlen;
            }
        } else if key == &Key::ArrowUp && self.input_locate == 0 {
            if let Some(txt) = self.history.arrow_up() {
                self.input_text = txt.0;
                self.history_cnt = txt.1;
            }
            self.input_locate = 0;
            self.visible_locate = 0;
        } else if key == &Key::ArrowDown && self.input_locate == 0 {
            if let Some(txt) = self.history.arrow_down() {
                self.input_text = txt.0;
                self.history_cnt = txt.1;
            }
            self.input_locate = 0;
            self.visible_locate = 0;
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
    fn pressed_enter(&mut self, itxt: String, graph: &mut Graphic) {
        if itxt.len() == 0 {
            return;
        }
        self.input_text = "".to_string();
        self.input_locate = 0;
        self.visible_locate = 0;
        let len = itxt.chars().count();
        if (len == 2 && &itxt[0..2] == "!q") || (len >= 5 && &itxt[0..5] == "!quit") {
            // The end of the App
            self.cmd.send_quit();
            self.gen_log();
            println!("That's all. Thank you!");
            std::process::exit(0);
        } else {
            if (len >= 5 && &itxt[0..5] == "!load") || (len >= 2 && &itxt[0..2] == "!l") {
                // Load File
                self.load_file(&itxt[0..], graph);
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
            } else {
                // Normal Input
                let msg = self.one_command(get_crnt_date_txt(), itxt, true);
                self.set_graphic_msg(msg, graph);
            }
        }
    }
    fn load_file(&mut self, itxt: &str, graph: &mut Graphic) {
        let blk_exists = |fnm: String| -> (Option<String>, Option<usize>) {
            let mut ltr = None;
            let mut num = None;
            if fnm.contains("blk") {
                ltr = Some(extract_texts_from_parentheses(fnm.as_str()).to_string());
            } else if fnm.contains("msr") {
                num = Some(extract_number_from_parentheses(fnm.as_str()));
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
            self.next_msr_tick = self.get_loaded_text(mt, graph);
        } else {
            // 適切なファイルや中身がなかった場合
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "No history".to_string(),
            ));
        }
    }
    pub fn auto_load_command(&mut self, guiev: &GuiEv, graph: &mut Graphic) {
        // from main loop
        if let Some(nmt) = self.next_msr_tick {
            let crnt: CrntMsrTick = guiev.get_msr_tick();
            if nmt.msr != LAST
                && nmt.msr > 0
                && nmt.msr - 1 == crnt.msr  // 一つ前の小節(両方とも1origin)
                && crnt.tick_for_onemsr - crnt.tick < 240
            {
                self.next_msr_tick = self.get_loaded_text(nmt, graph);
            }
        }
    }
    fn get_loaded_text(&mut self, mt: CrntMsrTick, graph: &mut Graphic) -> Option<CrntMsrTick> {
        let loaded = self.history.get_loaded_text(mt);
        for onecmd in loaded.0.iter() {
            let msg = self.one_command(get_crnt_date_txt(), onecmd.clone(), false);
            self.set_graphic_msg(msg, graph);
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
    fn set_graphic_msg(&mut self, msg: i16, graph: &mut Graphic) {
        match msg {
            LIGHT_MODE => graph.set_mode(GraphMode::Light),
            DARK_MODE => graph.set_mode(GraphMode::Dark),
            RIPPLE_PATTERN => graph.set_noteptn(GraphNote::Ripple),
            VOICE_PATTERN => graph.set_noteptn(GraphNote::Voice),
            _ => {}
        }
    }
}
