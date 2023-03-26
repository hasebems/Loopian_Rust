//  Created by Hasebe Masahiko on 2022/10/30.
//  Copyright (c) 2022 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
mod cmd;
mod elapse;
mod lpnlib;

use std::fs;
use std::fs::File;
use std::io::Write;
//use std::io::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;
use chrono::Local;
use eframe::{egui::*};
use eframe::egui;

use cmd::cmdparse;
use elapse::stack_elapse::ElapseStack;

//#[derive(Default)]
pub struct LoopianApp {
    input_locate: usize,
    input_text: String,
    start_time: Instant,
    input_lines: Vec<(String, String)>,
    cmd: cmdparse::LoopianCmd,
    history: usize,
}

impl LoopianApp {
    const WINDOW_X: f32 = 900.0;
    const WINDOW_Y: f32 = 480.0;

    const SPACE_LEFT: f32 = 30.0;
    const SPACE_RIGHT: f32 = 870.0;
    const _LEFT_MERGIN: f32 = 5.0;
    const LETTER_WIDTH: f32 = 9.56;
    const BLOCK_LENGTH: f32 = 210.0;
    const NEXT_BLOCK: f32 = 220.0;

    const SPACE1_UPPER: f32 = 50.0;
    const SPACE1_HEIGHT: f32 = 30.0;
    const SPACE1_NEXT: f32 = 50.0;
    const SPACE2_UPPER: f32 = 150.0;    // scroll text
    const SPACE2_LOWER: f32 = 400.0;
    const SPACE3_UPPER: f32 = 420.0;    // input text
    const SPACE3_LOWER: f32 = 450.0;
    const MAX_INDICATOR: usize = 8;

    const TXT_LEFT_MARGIN: f32 = 5.0;

    const MAZENTA: Color32 = Color32::from_rgb(255, 0, 255);
    const TEXT_GRAY: Color32 = Color32::from_rgb(0,0,0);
    const _TEXT_BG: Color32 = Color32::from_rgb(0,200,200);

    const BACK_WHITE: Color32 = Color32::from_rgb(180, 180, 180);
    const BACK_WHITE2: Color32 = Color32::from_rgb(128,128,128);
    const _BACK_MAZENTA: Color32 = Color32::from_rgb(180, 160, 180);
    const BACK_GRAY: Color32 = Color32::from_rgb(48,48,48);
    const BACK_GRAY2: Color32 = Color32::from_rgb(160, 160, 160);

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
            start_time: Instant::now(), // Current Time
            input_lines: Vec::new(),
            cmd: cmdparse::LoopianCmd::new(txmsg, rxui),
            history: 0,
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
    fn gen_log(&mut self) {
        // フォルダ作成
        let path = Path::new("log");
        if !path.is_dir() {
            fs::create_dir_all(path).unwrap();
        }
        // 時間をファイル名に使う
        let file = Local::now().format("%Y-%m-%d_%H-%M-%S.txt").to_string();
        let path_str = "log/".to_string() + &file;
        let path = Path::new(&path_str);
        let display = path.display();
        // log収集
        let mut whole_txt: String = String::new();
        for line in self.input_lines.iter() {
            if line.0.len() > 0 {
                whole_txt += &line.0.to_string();
                whole_txt += &line.1.to_string();
                whole_txt += "\n";
            }
        }
        // ファイル作成
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };
        // ファイル書き込み
        match file.write_all(whole_txt.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why),
            Ok(_) => println!("successfully wrote to {}", display),
        }
    }
    fn app_end(&mut self) {
        self.gen_log();
        println!("That's all. Thank you!");
    }
    //*******************************************************************
    //      Update Screen
    //*******************************************************************
    fn update_title(ui: &mut egui::Ui) {
        ui.put(
            Rect { min: Pos2 {x:395.0, y:2.0}, max: Pos2 {x:505.0, y:47.0},}, //  location
            Label::new(RichText::new("Loopian")
                .size(32.0)
                .color(Color32::WHITE)
                .family(FontFamily::Proportional)
            )
        );
    }
    //*******************************************************************
    fn text_for_eight_indicator(&mut self, num: i32) -> String {
        let indi_txt;
        match num {
            0 => indi_txt = "key: ".to_string() + self.cmd.get_indicator(0),
            1 => indi_txt = "bpm: ".to_string() + self.cmd.get_indicator(1),
            2 => indi_txt = "beat:".to_string() + self.cmd.get_indicator(2),
            4 => indi_txt = "L1:".to_string() + self.cmd.get_indicator(4),
            5 => indi_txt = "L2:".to_string() + self.cmd.get_indicator(5),
            6 => indi_txt = "R1:".to_string() + self.cmd.get_indicator(6),
            7 => indi_txt = "R2:".to_string() + self.cmd.get_indicator(7),
            3 => indi_txt = self.cmd.get_indicator(3).to_string(),
            _ => indi_txt = "".to_string(),
        }
        indi_txt
    }
    fn update_eight_indicator(&mut self, ui: &mut egui::Ui) {
        let input_part = self.cmd.get_input_part();
        for i in 0..4 {
            for j in 0..2 {
                let mut back_color = Self::BACK_WHITE;
                if i as usize != input_part && j == 1 {back_color = Self::BACK_WHITE2;}
                let raw: f32 = Self::NEXT_BLOCK*(i as f32);
                let line: f32 = Self::SPACE1_NEXT*(j as f32);
                ui.painter().rect_filled(
                    Rect { min: Pos2 {x:Self::SPACE_LEFT + raw,
                                      y:Self::SPACE1_UPPER + line}, 
                           max: Pos2 {x:Self::BLOCK_LENGTH + raw,
                                      y:Self::SPACE1_UPPER + Self::SPACE1_HEIGHT + line},}, //  location
                    8.0,                //  curve
                    back_color,     //  color
                );
                let tx = self.text_for_eight_indicator(i + j*4);
                let ltrcnt = tx.chars().count();
                for k in 0..ltrcnt {
                    ui.put(Rect {
                        min: Pos2 {
                            x:Self::SPACE_LEFT + 10.0 + raw + Self::LETTER_WIDTH*(k as f32),
                            y:Self::SPACE1_UPPER + 2.0 + line},
                        max: Pos2 {
                            x:Self::SPACE_LEFT + 10.0 + raw + Self::LETTER_WIDTH*((k+1) as f32),
                            y:Self::SPACE1_UPPER + 27.0 + line},},
                        Label::new(RichText::new(&tx[k..k+1])
                            .size(16.0).color(Self::TEXT_GRAY)
                            .family(FontFamily::Monospace).text_style(TextStyle::Monospace))
                    );
                }
            }
        }
    }
    //*******************************************************************
    fn update_scroll_text(&self, ui: &mut egui::Ui) {
        const LETTER_HEIGHT: f32 = 25.0;
        const FONT_SIZE: f32 = 16.0;
        ui.painter().rect_filled(
            Rect::from_min_max( pos2(Self::SPACE_LEFT, Self::SPACE2_UPPER),
                                pos2(Self::SPACE_RIGHT, Self::SPACE2_LOWER)),
            2.0,                  //  curve
            Self::BACK_GRAY     //  color
        );
        let mut max_count = 10;
        let mut ofs_count = 0;
        if self.input_lines.len() < 10 {
            max_count = self.input_lines.len();
        }
        else {
            ofs_count = self.input_lines.len() - 10;
        }
        for i in 0..max_count {
            let past_text_set = self.input_lines[ofs_count+i].clone();
            let past_text = past_text_set.0 + &past_text_set.1;
            let cnt = past_text.chars().count();
            let txt_color = if i%2==0 {Color32::WHITE} else {Self::MAZENTA};
            ui.put(
                Rect { 
                    min: Pos2 {x:Self::SPACE_LEFT + Self::TXT_LEFT_MARGIN,
                               y:Self::SPACE2_UPPER + LETTER_HEIGHT*(i as f32)}, 
                    max: Pos2 {x:Self::SPACE_LEFT + Self::TXT_LEFT_MARGIN + Self::LETTER_WIDTH*(cnt as f32),
                               y:Self::SPACE2_UPPER + LETTER_HEIGHT*((i+1) as f32)},},
                Label::new(RichText::new(&past_text)
                    .size(FONT_SIZE)
                    .color(txt_color)
                    .family(FontFamily::Monospace)
                )
            );
        }
    }
    //*******************************************************************
    fn input_letter(&mut self, letters: Vec<&String>) {
        const CURSOR_MAX_LOCATE: usize = 79;
        if self.input_locate <= CURSOR_MAX_LOCATE {
            //println!("Letters:{:?}",letters);
            letters.iter().for_each(|ltr| {
                self.input_text.insert_str(self.input_locate,ltr);
                self.input_locate+=1;
            });
        }
    }
    fn update_input_text(&mut self, ui: &mut egui::Ui) {
        const CURSOR_LEFT_MARGIN: f32 = 10.0;
        const CURSOR_LOWER_MERGIN: f32 = 6.0;
        const CURSOR_TXT_LENGTH: f32 = 9.55;
        const CURSOR_THICKNESS: f32 = 4.0;
        const PROMPT_LETTERS: usize = 3;

        const TXT_UPPER_MARGIN: f32 = 2.0;
        const TXT_LOWER_MARGIN: f32 = -3.0;
        const FONT_SIZE: f32 = 16.0;

        // Paint Letter Space
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(Self::SPACE_LEFT,Self::SPACE3_UPPER),
                               pos2(Self::SPACE_RIGHT,Self::SPACE3_LOWER)),
            2.0,                              //  curve
            Self::BACK_GRAY     //  color
        );
        // Paint cursor
        let cursor = self.input_locate + PROMPT_LETTERS;
        let elapsed_time = self.start_time.elapsed().as_millis();
        if elapsed_time%500 > 200 {
            ui.painter().rect_filled(
                Rect { min: Pos2 {x:Self::SPACE_LEFT + CURSOR_LEFT_MARGIN + CURSOR_TXT_LENGTH*(cursor as f32),
                                y:Self::SPACE3_LOWER - CURSOR_LOWER_MERGIN},
                       max: Pos2 {x:Self::SPACE_LEFT + CURSOR_LEFT_MARGIN + CURSOR_TXT_LENGTH*((cursor+1) as f32) - 2.0,
                                y:Self::SPACE3_LOWER - CURSOR_LOWER_MERGIN + CURSOR_THICKNESS},},
                0.0,                              //  curve
                Self::BACK_GRAY2,  //  color
            );
        }
        // Draw Letters
        let prompt_mergin: f32 = Self::LETTER_WIDTH*(PROMPT_LETTERS as f32);
        let prompt_txt: &str = self.cmd.get_part_txt();
        // Prompt Text
        ui.put(
            Rect { 
                   min: Pos2 {x:Self::SPACE_LEFT + Self::TXT_LEFT_MARGIN,
                              y:Self::SPACE3_UPPER + TXT_UPPER_MARGIN},
                   max: Pos2 {x:Self::SPACE_LEFT + Self::TXT_LEFT_MARGIN + prompt_mergin,
                              y:Self::SPACE3_LOWER + TXT_LOWER_MARGIN},},
            Label::new(RichText::new(prompt_txt)
                .size(FONT_SIZE)
                .color(Self::MAZENTA)
                .family(FontFamily::Monospace))
        );
        let ltrcnt = self.input_text.chars().count();
        let input_mergin: f32 = prompt_mergin + 3.25;
        for i in 0..ltrcnt {    // 位置を合わせるため、１文字ずつ Label を作って並べて配置する
            // User Input
            ui.put(
                Rect { 
                    min: Pos2 {x:Self::SPACE_LEFT + Self::TXT_LEFT_MARGIN + input_mergin + Self::LETTER_WIDTH*(i as f32),
                               y:Self::SPACE3_UPPER + TXT_UPPER_MARGIN},
                    max: Pos2 {x:Self::SPACE_LEFT + Self::TXT_LEFT_MARGIN + input_mergin + Self::LETTER_WIDTH*((i+1) as f32),
                               y:Self::SPACE3_LOWER + TXT_LOWER_MARGIN},},
                Label::new(RichText::new(&self.input_text[i..i+1])
                    .size(FONT_SIZE)
                    .color(Color32::WHITE)
                    .family(FontFamily::Monospace)
                    .text_style(TextStyle::Monospace))
            );
        }
    }
    //*******************************************************************
    fn command_key(&mut self, key: &Key, modifiers: &Modifiers) {
        if key == &Key::Enter {
            if self.input_text.len() == 0 {return;}
            let dt = Local::now();
            let tm = dt.format("%Y-%m-%d %H:%M:%S ").to_string();
            self.input_lines.push((tm, self.input_text.clone()));
            if let Some(answer) = self.cmd.set_and_responce(&self.input_text) {
                self.input_lines.push(("".to_string(), answer));
                self.input_text = "".to_string();
                self.input_locate = 0;
                self.history = self.input_lines.len();
            }
            else {  // The end of the App
                self.app_end();
                std::process::exit(0);
            }
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
            let max_count = self.input_lines.len();
            if self.history >= 2 {self.history -= 2;}
            if max_count > 0 && self.history < max_count {
                self.input_text = self.input_lines[self.history].1.clone();
            }
            let maxlen = self.input_text.chars().count();
            if maxlen < self.input_locate {self.input_locate = maxlen;}
        }
        else if key == &Key::ArrowDown {
            let max_count = self.input_lines.len();
            if self.history < max_count {self.history += 2;}
            if max_count > 0 && self.history < max_count {
                self.input_text = self.input_lines[self.history].1.clone();
            }
            else if self.history >= max_count {
                self.input_text = "".to_string();
            }
            let maxlen = self.input_text.chars().count();
            if maxlen < self.input_locate {self.input_locate = maxlen;}
        }
    }
}

//*******************************************************************
//     Egui/Eframe framework basic
//*******************************************************************
impl eframe::App for LoopianApp {
    fn on_close_event(&mut self) -> bool {
        self.app_end();
        true
    }
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // repaint 100msec interval
        ctx.request_repaint_after(Duration::from_millis(100));

        //  Get Keyboard Event from Egui::Context
        let evts = ctx.input().events.clone();  
        let mut letters: Vec<&String> = vec![];
        for ev in evts.iter() {
            match ev {
                Event::Text(ltr) => letters.push(ltr),
                Event::Key {key,pressed, modifiers} => {
                    if pressed == &true { self.command_key(key, modifiers);}
                },
                _ => {},
            }
        }
        if letters.len() >= 1 {self.input_letter(letters);}

        // Configuration for CentralPanel
        let my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow { extrusion: 0.0, color: Color32::BLACK },
            fill: Color32::BLACK,
            stroke: egui::Stroke::new(0.0, Color32::BLACK),
        };

        // Draw CentralPanel
        CentralPanel::default().frame(my_frame).show(ctx, |ui| {
            Self::update_title(ui);
            self.update_eight_indicator(ui);

            //  scroll text
            self.update_scroll_text(ui);

            //  input text
            self.update_input_text(ui);
        });
    }
}

//*******************************************************************
//      Main
//*******************************************************************
fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some((LoopianApp::WINDOW_X, LoopianApp::WINDOW_Y).into()),
        resizable: false,
//        follow_system_theme: false,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native("Loopian", options, 
        Box::new(|cc| Box::new(LoopianApp::new(cc))));
}