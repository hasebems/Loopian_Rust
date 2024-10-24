//  Created by Hasebe Masahiko on 2022/10/30.
//  Copyright (c) 2022 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
mod cmd;
mod elapse;
mod file;
mod graphic;
mod lpnlib;
mod midi;
mod server;
mod test;

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use eframe::{egui, egui::*};
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use cmd::cmdparse;
use cmd::txt_common::*;
use elapse::stack_elapse::ElapseStack;
use elapse::tickgen::CrntMsrTick;
use file::history::History;
use file::settings::Settings;
use graphic::graphic::{Graphic, TextAttribute};
use graphic::guiev::GuiEv;
use lpnlib::*;
use server::server::cui_loop;

pub struct LoopianApp {
    input_locate: usize,   //  カーソルの位置
    visible_locate: usize, //  入力部に表示する最初の文字の位置
    input_text: String,
    file_name_stock: String, // 前回ロードしたファイル名を保持
    scroll_lines: Vec<(TextAttribute, String, String)>,
    history_cnt: usize,
    next_msr_tick: Option<CrntMsrTick>, //
    cmd: cmdparse::LoopianCmd,
    history: History,
    graph: Graphic,
    guiev: GuiEv,
}
impl LoopianApp {
    //*******************************************************************
    //      App Initialize / Log File /  App End
    //*******************************************************************
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (txmsg, rxui) = gen_elapse_thread();
        Self::init_font(cc);
        Self {
            input_locate: 0,
            visible_locate: 0,
            input_text: String::new(),
            file_name_stock: String::new(),
            scroll_lines: Vec::new(),
            history_cnt: 0,
            next_msr_tick: None,
            cmd: cmdparse::LoopianCmd::new(txmsg),
            history: History::new(),
            graph: Graphic::new(),
            guiev: GuiEv::new(rxui, true),
        }
    }
    fn init_font(cc: &eframe::CreationContext<'_>) {
        let mut fonts = Self::add_myfont();

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(FontFamily::Proportional) //  search value of this key
            .or_default() //  if not found
            .insert(0, "profont".to_owned());

        // Put my font first (highest priority) for monospace text:
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "monofont".to_owned());

        // Tell egui to use these fonts:
        cc.egui_ctx.set_fonts(fonts);
    }
    /// Font Data File Name with path
    pub fn add_myfont() -> FontDefinitions {
        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        #[cfg(not(feature = "raspi"))]
        fonts.font_data.insert(
            "profont".to_owned(),
            FontData::from_static(include_bytes!("../assets/newyork.ttf")), // for Mac
        );
        #[cfg(feature = "raspi")]
        fonts.font_data.insert(
            "profont".to_owned(),
            FontData::from_static(include_bytes!(
                "/home/pi/loopian/Loopian_Rust/assets/NewYork.ttf"
            )), // for linux
        );
        #[cfg(not(feature = "raspi"))]
        fonts.font_data.insert(
            "monofont".to_owned(),
            FontData::from_static(include_bytes!("../assets/courier.ttc")), // for Mac
        );
        #[cfg(feature = "raspi")]
        fonts.font_data.insert(
            "monofont".to_owned(),
            FontData::from_static(include_bytes!(
                "/home/pi/loopian/Loopian_Rust/assets/Courier.ttc"
            )), // for linux
        );
        fonts
    }
    fn app_end(&mut self, save: bool) {
        if save {
            self.history.gen_log();
        } else {
            println!("File wasn't saved.");
        }
        println!("That's all. Thank you!");
    }
    //*******************************************************************
    //      Input Text
    //*******************************************************************
    const CURSOR_MAX_VISIBLE_LOCATE: usize = 65;
    fn input_letter(&mut self, letters: Vec<&String>) {
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
    fn pressed_key(&mut self, key: &Key, modifiers: &Modifiers) {
        let itxt: String = self.input_text.clone();
        if key == &Key::Enter {
            self.pressed_enter(itxt);
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
    fn get_cursor_locate(&self) -> usize {
        if self.input_locate > Self::CURSOR_MAX_VISIBLE_LOCATE {
            Self::CURSOR_MAX_VISIBLE_LOCATE
        } else {
            self.input_locate
        }
    }
    fn pressed_enter(&mut self, itxt: String) {
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
            self.app_end(true);
            std::process::exit(0);
        } else {
            if (len >= 5 && &itxt[0..5] == "!load") || (len >= 2 && &itxt[0..2] == "!l") {
                // Load File
                self.load_file(&itxt[0..]);
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
                self.one_command(get_crnt_date_txt(), itxt, true);
            }
        }
    }
    fn load_file(&mut self, itxt: &str) {
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
            self.next_msr_tick = self.get_loaded_text(mt);
        } else {
            // 適切なファイルや中身がなかった場合
            self.scroll_lines.push((
                TextAttribute::Answer,
                "".to_string(),
                "No history".to_string(),
            ));
        }
    }
    fn auto_load_command(&mut self) {
        // from main loop
        if let Some(nmt) = self.next_msr_tick {
            let crnt: CrntMsrTick = self.guiev.get_msr_tick();
            if nmt.msr != LAST
                && nmt.msr > 0
                && nmt.msr - 1 == crnt.msr  // 一つ前の小節(両方とも1origin)
                && crnt.tick_for_onemsr - crnt.tick < 240
            {
                self.next_msr_tick = self.get_loaded_text(nmt);
            }
        }
    }
    fn get_loaded_text(&mut self, mt: CrntMsrTick) -> Option<CrntMsrTick> {
        let loaded = self.history.get_loaded_text(mt);
        for cmd in loaded.0.iter() {
            self.one_command(get_crnt_date_txt(), cmd.clone(), false);
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
    fn one_command(&mut self, time: String, itxt: String, verbose: bool) {
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
            match answer.1 {
                LIGHT_MODE => self.graph.set_mode(GraphMode::Light),
                DARK_MODE => self.graph.set_mode(GraphMode::Dark),
                RIPPLE_PATTERN => self.graph.set_noteptn(GraphNote::Ripple),
                VOICE_PATTERN => self.graph.set_noteptn(GraphNote::Voice),
                NO_MSG => {}
                _ => {}
            }
        }
    }
    //*******************************************************************
    //      Central Panel
    //*******************************************************************
    fn draw_central_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Configuration for CentralPanel
        let back_color = self.graph.back_color();
        let my_frame = egui::containers::Frame {
            inner_margin: egui::Margin {
                left: 0.,
                right: 0.,
                top: 0.,
                bottom: 0.,
            },
            outer_margin: egui::Margin {
                left: 0.,
                right: 0.,
                top: 0.,
                bottom: 0.,
            },
            rounding: egui::Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 0.0,
                se: 0.0,
            },
            shadow: eframe::epaint::Shadow {
                offset: Vec2::ZERO,
                blur: 0.0,
                spread: 0.0,
                color: back_color,
            },
            fill: back_color,
            stroke: egui::Stroke::new(0.0, back_color),
        };
        CentralPanel::default().frame(my_frame).show(ctx, |ui| {
            let visible_text = &self.input_text[self.visible_locate..];
            self.graph.update(
                ui,
                (
                    self.get_cursor_locate(),
                    &visible_text.to_string(),
                    &self.scroll_lines,
                    self.history_cnt,
                    self.cmd.get_input_part(),
                    &self.guiev,
                ),
                frame,
            );
            self.guiev.clear_graphic_ev();
        });
    }
}
//*******************************************************************
//     Egui/Eframe framework basic
//*******************************************************************
impl eframe::App for LoopianApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app_end(true);
    }
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // 40fps で画面更新
        ctx.request_repaint_after(Duration::from_millis(25));

        //  Get Keyboard Event from Egui::Context
        ctx.input(|i| {
            let mut letters: Vec<&String> = vec![];
            for ev in i.events.iter() {
                match ev {
                    Event::Text(ltr) => letters.push(ltr),
                    Event::Key {
                        key,
                        pressed,
                        modifiers,
                        repeat: _,
                        physical_key: _,
                    } => {
                        if pressed == &true {
                            self.pressed_key(&key, &modifiers);
                        }
                    }
                    _ => {}
                }
            }
            if letters.len() >= 1 {
                self.input_letter(letters);
            }
        });

        //  Read imformation from StackElapse
        self.guiev.read_from_ui_hndr(&mut self.cmd);

        //  Auto Load Function
        self.auto_load_command();

        //  Draw CentralPanel
        self.draw_central_panel(ctx, frame);
    }
}
//*******************************************************************
//      Main
//*******************************************************************
/// GUI/CUI 両方から呼ばれる
fn gen_elapse_thread() -> (Sender<ElpsMsg>, Receiver<UiMsg>) {
    //  create new thread & channel
    let (txmsg, rxmsg) = mpsc::channel();
    let (txui, rxui) = mpsc::channel();
    thread::spawn(move || {
        let mut est = ElapseStack::new(txui);
        loop {
            if est.periodic(rxmsg.try_recv()) {
                break;
            }
        }
    });
    (txmsg, rxui)
}
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Args: {:?}", args);
    if args.len() > 1 && args[1] == "server" {
        // CUI version
        cui_loop();
    } else {
        // GUI version
        let winsz = &Settings::load_settings().window_size;
        let sz_default = [winsz.window_x_default, winsz.window_y_default];
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(sz_default),
            ..eframe::NativeOptions::default()
        };
        let _ = eframe::run_native(
            "Loopian",
            options,
            Box::new(|cc| Ok(Box::new(LoopianApp::new(cc)))),
        );
    }
}
