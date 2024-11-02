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

use eframe::{egui, egui::*};
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use elapse::stack_elapse::ElapseStack;
use file::input_txt::InputText;
use file::settings::Settings;
use graphic::graphic::Graphic;
use graphic::guiev::GuiEv;
use lpnlib::*;
use server::server::cui_loop;

pub struct LoopianApp {
    ui_hndr: mpsc::Receiver<UiMsg>,
    itxt: InputText,
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
            itxt: InputText::new(txmsg),
            ui_hndr: rxui,
            graph: Graphic::new(),
            guiev: GuiEv::new(true),
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
            self.graph.update(
                ui,
                (
                    self.itxt.get_cursor_locate(),
                    &self.itxt.get_input_text(),
                    self.itxt.get_scroll_lines(),
                    self.itxt.get_history_cnt(),
                    self.itxt.get_input_part(),
                    &self.guiev,
                ),
                frame,
            );
            self.guiev.clear_graphic_ev();
        });
    }
    pub fn read_from_ui_hndr(&mut self) {
        loop {
            match self.ui_hndr.try_recv() {
                Ok(msg) => {
                    let key = self.itxt.get_indicator_key_stock();
                    self.guiev.set_indicator(msg, key);
                }
                Err(TryRecvError::Disconnected) => break, // Wrong!
                Err(TryRecvError::Empty) => break,
            }
        }
    }
}
//*******************************************************************
//     Egui/Eframe framework basic
//*******************************************************************
impl eframe::App for LoopianApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.itxt.gen_log(0, "".to_string());
        println!("That's all. Thank you!");
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
                            self.itxt.pressed_key(&key, &modifiers, &mut self.graph);
                        }
                    }
                    _ => {}
                }
            }
            if letters.len() >= 1 {
                self.itxt.input_letter(letters);
            }
        });

        //  Read imformation from StackElapse
        self.read_from_ui_hndr();

        //  Auto Load Function
        self.itxt.auto_load_command(&self.guiev, &mut self.graph);

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
