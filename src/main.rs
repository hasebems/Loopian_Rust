//  Created by Hasebe Masahiko on 2022/10/30.
//  Copyright (c) 2022 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
mod cmd;
mod elapse;

use eframe::{egui::*};
use eframe::egui;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;
use cmd::cmdparse;
use elapse::elapse_stack::ElapseStack;

//#[derive(Default)]
pub struct LoopianApp {
    input_locate: usize,
    input_text: String,
    start_time: Instant,
    input_lines: Vec<String>,
    cmd: cmdparse::LoopianCmd,
}

impl LoopianApp {
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

    const CURSOR_MERGIN: f32 = 6.0;
    const CURSOR_THICKNESS: f32 = 4.0;

    const PROMPT_LETTERS: usize = 3;
    const CURSOR_MAX_LOCATE: usize = 79;

    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //  create new thread & channel
        let (txtxt, rxtxt) = mpsc::channel();
        let (txui, rxui) = mpsc::channel();
        thread::spawn(move || {
            match ElapseStack::new(txui) {
                Some(mut est) => {
                    loop { if est.periodic(rxtxt.try_recv()) {break;}}
                },
                None => {println!("Play System does't work")},
            }
        });

        Self::init_font(cc);
        Self {
            input_locate: 0,
            input_text: String::new(),
            start_time: Instant::now(), // Current Time
            input_lines: Vec::new(),
            cmd: cmdparse::LoopianCmd::new(txtxt, rxui),
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
    //  for update()
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
    fn text_for_eight_indicator(&mut self, num: i32) -> String {
        let indi_txt;
        match num {
            0 => indi_txt = "key:".to_string() + self.cmd.get_indicator(0),
            1 => indi_txt = "bpm:".to_string() + self.cmd.get_indicator(1),
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
        for i in 0..4 {
            for j in 0..2 {
                let raw: f32 = Self::NEXT_BLOCK*(i as f32);
                let line: f32 = Self::SPACE1_NEXT*(j as f32);
                ui.painter().rect_filled(
                    Rect { min: Pos2 {x:Self::SPACE_LEFT + raw,
                                      y:Self::SPACE1_UPPER + line}, 
                           max: Pos2 {x:Self::BLOCK_LENGTH + raw,
                                      y:Self::SPACE1_UPPER + Self::SPACE1_HEIGHT + line},}, //  location
                    8.0,                //  curve
                    Color32::from_rgb(180, 180, 180),     //  color
                );
                let tx = self.text_for_eight_indicator(i + j*4);
                let ltrcnt = tx.chars().count();
                ui.put(
                    Rect { min: Pos2 {x:Self::SPACE_LEFT + 10.0 + raw,
                                      y:Self::SPACE1_UPPER + 2.0 + line},
                           max: Pos2 {x:Self::SPACE_LEFT + 10.0 + raw + Self::LETTER_WIDTH*(ltrcnt as f32),
                                      y:Self::SPACE1_UPPER + 27.0 + line},},
                    Label::new(RichText::new(&tx)
                        .size(16.0).color(Color32::from_rgb(48,48,48))
                        .family(FontFamily::Monospace).text_style(TextStyle::Monospace))
                );
            }
        }
    }
    fn update_scroll_text(&self, ui: &mut egui::Ui) {
        ui.painter().rect_filled(
            Rect::from_min_max( pos2(Self::SPACE_LEFT, Self::SPACE2_UPPER),
                                pos2(Self::SPACE_RIGHT, Self::SPACE2_LOWER)),
            2.0,                              //  curve
            Color32::from_rgb(48, 48, 48)     //  color
        );
        const LETTER_HEIGHT: f32 = 25.0;
        let mut max_count = 10;
        let mut ofs_count = 0;
        if self.input_lines.len() < 10 {
            max_count = self.input_lines.len();
        }
        else {
            ofs_count = self.input_lines.len() - 10;
        }
        for i in 0..max_count {
            let past_text = self.input_lines[ofs_count+i].clone();
            let cnt = past_text.chars().count();
            let txt_color = if i%2==0 {Color32::WHITE} else {Color32::from_rgb(255,0,255)};
            ui.put(
                Rect { min: Pos2 {x:Self::SPACE_LEFT + 5.0,
                                  y:Self::SPACE2_UPPER + LETTER_HEIGHT*(i as f32)}, 
                       max: Pos2 {x:Self::SPACE_LEFT + 5.0 + Self::LETTER_WIDTH*(cnt as f32),
                                  y:Self::SPACE2_UPPER + LETTER_HEIGHT*((i+1) as f32)},},
                Label::new(RichText::new(&past_text)
                    .size(16.0)
                    .color(txt_color)
                    .family(FontFamily::Monospace)
                )
            );
        }
    }
    fn command_key(&mut self, key: &Key) {
        if key == &Key::Enter {
            if self.input_text.len() == 0 {return;}
            self.input_lines.push(self.input_text.clone());
            if let Some(answer) = self.cmd.set_and_responce(&self.input_text) {
                self.input_lines.push(answer.clone());
                self.input_text = "".to_string();
                self.input_locate = 0;
            }
            else {  // The end of the App
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
            if self.input_locate > 0 {self.input_locate -= 1;}
            //println!("Key>>{:?}",key);
        }
        else if key == &Key::ArrowRight {
            self.input_locate += 1;
            let maxlen = self.input_text.chars().count();
            if self.input_locate > maxlen { self.input_locate = maxlen;}
            //println!("Key>>{:?}",key);
        }
    }
    fn input_letter(&mut self, letters: Vec<&String>) {
        if self.input_locate <= Self::CURSOR_MAX_LOCATE {
            //println!("Letters:{:?}",letters);
            letters.iter().for_each(|ltr| {
                if *ltr==" " {self.input_text.insert_str(self.input_locate,"_");}
                else         {self.input_text.insert_str(self.input_locate,ltr);}
                self.input_locate+=1;
            });
        }
    }
    fn update_input_text(&mut self, ui: &mut egui::Ui) {
        // Paint Letter Space
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(Self::SPACE_LEFT,Self::SPACE3_UPPER),
                               pos2(Self::SPACE_RIGHT,Self::SPACE3_LOWER)),
            2.0,                              //  curve
            Color32::from_rgb(48, 48, 48)     //  color
        );
        // Paint cursor
        let cursor = self.input_locate + Self::PROMPT_LETTERS;
        let elapsed_time = self.start_time.elapsed().as_millis();
        if elapsed_time%500 > 200 {
            ui.painter().rect_filled(
                Rect { min: Pos2 {x:Self::SPACE_LEFT + 10.0 + 9.5*(cursor as f32),
                                y:Self::SPACE3_LOWER - Self::CURSOR_MERGIN},
                       max: Pos2 {x:Self::SPACE_LEFT + 8.0 + 9.5*((cursor+1) as f32),
                                y:Self::SPACE3_LOWER - Self::CURSOR_MERGIN + Self::CURSOR_THICKNESS},},
                0.0,                              //  curve
                Color32::from_rgb(160, 160, 160)  //  color
            );
        }
        // Draw Letters
        let prompt_mergin: f32 = Self::LETTER_WIDTH*(Self::PROMPT_LETTERS as f32);
        ui.put( // Prompt
            Rect { min: Pos2 {x:Self::SPACE_LEFT + 5.0,
                              y:Self::SPACE3_UPPER + 2.0},
                   max: Pos2 {x:Self::SPACE_LEFT + 5.0 + prompt_mergin,
                              y:Self::SPACE3_LOWER - 3.0},},
            Label::new(RichText::new("R1>")
                .size(16.0).color(Color32::from_rgb(0,200,200)).family(FontFamily::Monospace))
        );
        let ltrcnt = self.input_text.chars().count();
        let input_mergin: f32 = prompt_mergin + 3.25;
        ui.put( // User Input
            Rect { min: Pos2 {x:Self::SPACE_LEFT + 5.0 + input_mergin,
                              y:Self::SPACE3_UPPER + 2.0},
                   max: Pos2 {x:Self::SPACE_LEFT + 5.0 + input_mergin + Self::LETTER_WIDTH*(ltrcnt as f32),
                              y:Self::SPACE3_LOWER - 3.0},},
            Label::new(RichText::new(&self.input_text)
                .size(16.0).color(Color32::WHITE).family(FontFamily::Monospace).text_style(TextStyle::Monospace))
        );
    }
}

impl eframe::App for LoopianApp {
    fn on_close_event(&mut self) -> bool {
        println!("App will end!");
        thread::sleep(Duration::from_millis(500));
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
                Event::Key {key,pressed, modifiers:_} => {
                    if pressed == &true { self.command_key(key);}
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

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some((900.0, 480.0).into()),
        resizable: false,
//        follow_system_theme: false,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native("Loopian", options, Box::new(|cc| Box::new(LoopianApp::new(cc))));
    println!("Bye, thank you!");
}