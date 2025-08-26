//  Created by Hasebe Masahiko on 2024/11/03.
//  Copyright (c) 2024 Hasebe Masahiko.
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

use nannou::prelude::*;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use elapse::stack_elapse::ElapseStack;
use file::input_txt::InputText;
use file::settings::Settings;
use graphic::draw_graph::{Graphic, Resize};
use graphic::guiev::GuiEv;
use lpnlib::*;
use server::server_loop::cui_loop;

//*******************************************************************
//      Main
//*******************************************************************
fn main() {
    let args: Vec<String> = env::args().collect();

    //  Version
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("*** Hi, I'm Loopian.");
    println!("*** I'm so glad to see you!");
    println!("*** Loopian Version: {VERSION}");

    //  Args
    println!("*** Args: {args:?}");

    //  Setting file の存在確認
    if !Settings::find_setting_file() {
        return;
    }

    if args.len() > 1 && args[1] == "server" {
        // CUI version
        cui_loop();
    } else {
        // GUI version
        nannou::app(model).event(event).update(update).run();
    }
}

//*******************************************************************
//      Model
//*******************************************************************
pub struct Model {
    ui_hndr: mpsc::Receiver<UiMsg>,
    itxt: InputText,
    graph: Graphic,
    guiev: GuiEv,
    first_run: bool,
    // as you like
}
fn model(app: &App) -> Model {
    let (txmsg, rxui) = gen_elapse_thread();
    app.new_window().view(view).build().unwrap();

    // app に対する初期設定
    app.set_exit_on_escape(false);
    let win = app.main_window();
    let first_width = Settings::load_settings().window_size.window_x_default;
    let first_height = Settings::load_settings().window_size.window_y_default;
    win.set_title("Loopian");
    win.set_inner_size_pixels(first_width, first_height);

    Model {
        ui_hndr: rxui,
        itxt: InputText::new(txmsg),
        graph: Graphic::new(app),
        guiev: GuiEv::new(true),
        first_run: true,
    }
}
/// GUI/CUI 両方から呼ばれる
fn gen_elapse_thread() -> (Sender<ElpsMsg>, Receiver<UiMsg>) {
    //  create new thread & channel
    let (txmsg, rxmsg) = mpsc::channel();
    let (txui, rxui) = mpsc::channel();
    let _ = thread::Builder::new()
        .name("elapse".to_string())
        .spawn(move || {
            let mut est = ElapseStack::new(txui);
            loop {
                if est.periodic(rxmsg.try_recv()) {
                    break;
                }
            }
        });
    (txmsg, rxui)
}

//*******************************************************************
//      Update & Event
//*******************************************************************
fn update(app: &App, model: &mut Model, _update: Update) {
    model.graph.set_rs(Resize::new(app));
    let crnt_time = app.time;

    //  Read imformation from StackElapse
    read_from_ui_hndr(model);

    // Auto Load
    model
        .itxt
        .auto_load_command(&model.guiev, model.graph.graph_msg());

    //  Update Model
    model
        .graph
        .update_lpn_model(&mut model.guiev, &model.itxt, crnt_time);

    if model.first_run {
        //  起動時の設定
        model.first_run = false;
        if let Some(cmd) = Settings::load_settings().command.init_commands {
            cmd.iter().for_each(|c| {
                model.itxt.set_command(c.clone(), model.graph.graph_msg());
            });
        }
    }
    // as you like
}
fn read_from_ui_hndr(model: &mut Model) {
    loop {
        match model.ui_hndr.try_recv() {
            Ok(msg) => {
                let key = model.itxt.get_indicator_key_stock();
                model.guiev.set_indicator(msg, key);
            }
            Err(TryRecvError::Disconnected) => break, // Wrong!
            Err(TryRecvError::Empty) => break,
        }
    }
}
fn event(_app: &App, model: &mut Model, event: Event) {
    model.itxt.window_event(event, model.graph.graph_msg());
}

//*******************************************************************
//      View
//*******************************************************************
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let tm = app.time;

    // 画面全体の背景色
    draw.background().color(model.graph.get_bgcolor());

    // as you like

    //  Loopian View の描画
    model
        .graph
        .view_loopian(draw.clone(), &model.guiev, &model.itxt, tm);

    draw.to_frame(app, &frame).unwrap();
}
