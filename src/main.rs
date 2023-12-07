//  Created by Hasebe Masahiko on 2022/10/30.
//  Copyright (c) 2022 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
mod cmd;
mod elapse;
mod graphic;
mod lpnlib;
mod test;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use chrono::Local;
use eframe::{egui,egui::*};

use cmd::cmdparse;
use elapse::stack_elapse::ElapseStack;
use cmd::history::History;
use graphic::graphic::Graphic;

pub const WINDOW_X: f32 = 1000.0;        //  Main Window
pub const WINDOW_Y: f32 = 860.0;

pub struct LoopianApp {
    input_locate: usize,
    input_text: String,
    scroll_lines: Vec<(String, String)>,
    history_cnt: usize,
    cmd: cmdparse::LoopianCmd,
    history: History,
    graph: Graphic,
}

impl LoopianApp {
    //*******************************************************************
    //      App Initialize / Log File /  App End
    //*******************************************************************
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //  create new thread & channel
        let (txmsg, rxmsg) = mpsc::channel();
        let (txui, rxui) = mpsc::channel();
        thread::spawn(move || {
            match ElapseStack::new(txui) {
                Some(mut est) => {
                    loop { if est.periodic(rxmsg.try_recv()) {break;}}
                },
                None => {println!("Elps thread does't work")},
            }
        });

        Self::init_font(cc);
        Self {
            input_locate: 0,
            input_text: String::new(),
            scroll_lines: Vec::new(),
            history_cnt: 0,
            cmd: cmdparse::LoopianCmd::new(txmsg, rxui),
            history: History::new(),
            graph: Graphic::new(),
        }
    }
    fn init_font(cc: &eframe::CreationContext<'_>) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        fonts.font_data.insert(
            "profont".to_owned(),
            FontData::from_static(include_bytes!("../assets/newyork.ttf")),
        );
        fonts.font_data.insert(
            "monofont".to_owned(),
            FontData::from_static(include_bytes!("../assets/courier.ttc")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(FontFamily::Proportional)    //  search value of this key
            .or_default()                       //  if not found
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
    fn app_end(&mut self, save: bool) {
        if save {self.history.gen_log();}
        else {println!("File wasn't saved.");}
        println!("That's all. Thank you!");
    }
    //*******************************************************************
    //      Input Text
    //*******************************************************************
    fn input_letter(&mut self, letters: Vec<&String>) {
        const CURSOR_MAX_LOCATE: usize = 65;
        if self.input_locate <= CURSOR_MAX_LOCATE {
            //println!("Letters:{:?}",letters);
            letters.iter().for_each(|ltr| {
                self.input_text.insert_str(self.input_locate,ltr);
                self.input_locate+=1;
            });
        }
    }
    fn pressed_key(&mut self, key: &Key, modifiers: &Modifiers) {
        let itxt: String = self.input_text.clone();
        if key == &Key::Enter {
            self.pressed_enter(itxt);
        }
        else if key == &Key::Backspace {
            if self.input_locate > 0 {
                self.input_locate -= 1;
                self.input_text.remove(self.input_locate);
            }
            //println!("Key>>{:?}",key);
        }
        else if key == &Key::ArrowLeft {
            if modifiers.shift {self.input_locate = 0;}
            else if self.input_locate > 0 {self.input_locate -= 1;}
            //println!("Key>>{:?}",key);
        }
        else if key == &Key::ArrowRight {
            let maxlen = self.input_text.chars().count();
            if modifiers.shift {self.input_locate = maxlen;}
            else {self.input_locate += 1;}
            if self.input_locate > maxlen {self.input_locate = maxlen;}
            //println!("Key>>{:?}",key);
        }
        else if key == &Key::ArrowUp {
            if let Some(txt) = self.history.arrow_up() {
                self.input_text = txt.0;
                self.history_cnt = txt.1;
            }
            let maxlen = self.input_text.chars().count();
            if maxlen < self.input_locate {self.input_locate = maxlen;}
        }
        else if key == &Key::ArrowDown {
            if let Some(txt) = self.history.arrow_down() {
                self.input_text = txt.0;
                self.history_cnt = txt.1;
            }
            let maxlen = self.input_text.chars().count();
            if maxlen < self.input_locate {self.input_locate = maxlen;}
        }
    }
    fn pressed_enter(&mut self, itxt: String) {
        if itxt.len() == 0 {return;}
        let dt = Local::now();
        let time = dt.format("%Y-%m-%d %H:%M:%S ").to_string();
        self.input_text = "".to_string();
        self.input_locate = 0;

        if itxt.chars().count() >= 5 && &itxt[0..5] == "load " {
            self.scroll_lines.push((time.clone(), itxt.clone()));     // for display text
            // load のときだけ特別処理
            let command_stk: Vec<String>;
            command_stk = self.history.load_lpn(&itxt[5..]);
            if command_stk.len() == 0 {
                self.scroll_lines.push(("".to_string(), "No history".to_string()));
            }
            else {
                self.scroll_lines.push(("".to_string(), "Loaded in history".to_string()));
                for cmd in command_stk.iter() {
                    self.one_command(time.clone(), cmd.clone());
                }
            }
        }
        else {
            self.one_command(time, itxt);
        }
    }
    fn one_command(&mut self, time: String, itxt: String) {
        // 通常のコマンド入力
        self.history_cnt = self.history.set_scroll_text(time.clone(), itxt.clone());// for history
        if let Some(answer) = self.cmd.set_and_responce(&itxt) {// for work
            if answer == "nosave" {  // The end of the App
                self.app_end(false);
                std::process::exit(0);
            }
            else {  // normal command
                self.scroll_lines.push((time.clone(), itxt.clone()));     // for display text
                self.scroll_lines.push(("".to_string(), answer));
            }
        }
        else {  // The end of the App
            self.app_end(true);
            std::process::exit(0);
        }
    }
    fn draw_central_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let mut ntev: Vec<String> = Vec::new();
        while let Some(kmsg) = self.cmd.move_ev_from_gev() {
            ntev.push(kmsg);
        }

        // Configuration for CentralPanel
        let back_color = self.graph.back_color();
        let my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow { extrusion: 0.0, color: back_color },
            fill: back_color,
            stroke: egui::Stroke::new(0.0, back_color),
        };
        CentralPanel::default().frame(my_frame).show(ctx, |ui| {
            self.graph.update(ui,
                (self.input_locate, 
                &self.input_text,
                &self.scroll_lines,
                self.history_cnt,
                &self.cmd),
                self.cmd.get_graphic_msg(),
                frame, ntev);
        });
    }
}
//*******************************************************************
//     Egui/Eframe framework basic
//*******************************************************************
impl eframe::App for LoopianApp {
    fn on_close_event(&mut self) -> bool {
        self.app_end(true);
        true
    }
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // 40fps で画面更新
        ctx.request_repaint_after(Duration::from_millis(25));

        //  Get Keyboard Event from Egui::Context
        ctx.input(|i|{ 
            let mut letters: Vec<&String> = vec![];
            for ev in i.events.iter() {
                match ev {
                    Event::Text(ltr) => letters.push(ltr),
                    Event::Key {key,pressed, modifiers, repeat:_} => {
                        if pressed == &true { self.pressed_key(&key, &modifiers);}
                    },
                    _ => {},
                }
            }
            if letters.len() >= 1 {self.input_letter(letters);}
        }); 

        //  Read imformation from StackElapse
        self.cmd.read_from_ui_hndr();

        //  Draw CentralPanel
        self.draw_central_panel(ctx, frame);
    }
}
//*******************************************************************
//      Main
//*******************************************************************
fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some((WINDOW_X, WINDOW_Y).into()),
//        resizable: false,
//        follow_system_theme: false,
        ..eframe::NativeOptions::default()
    };
    let _ = eframe::run_native("Loopian", options, 
        Box::new(|cc| Box::new(LoopianApp::new(cc))));
}