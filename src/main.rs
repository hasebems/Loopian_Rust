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
use graphic::graphic::{Graphic, Resize};
use graphic::guiev::GuiEv;
use lpnlib::*;
use server::server::cui_loop;

//*******************************************************************
//      Main
//*******************************************************************
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Args: {:?}", args);
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
const FIRST_WIDTH: u32 = 2800;
const FIRST_HEIGHT: u32 = 1800;

pub struct Model {
    ui_hndr: mpsc::Receiver<UiMsg>,
    itxt: InputText,
    graph: Graphic,
    guiev: GuiEv,
}
fn model(app: &App) -> Model {
    let (txmsg, rxui) = gen_elapse_thread();
    app.new_window().view(view).build().unwrap();

    // app に対する初期設定
    app.set_exit_on_escape(false);
    let win = app.main_window();
    win.set_title("Loopian");
    win.set_inner_size_pixels(FIRST_WIDTH, FIRST_HEIGHT);

    Model {
        ui_hndr: rxui,
        itxt: InputText::new(txmsg),
        graph: Graphic::new(app),
        guiev: GuiEv::new(true),
    }
}
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

//*******************************************************************
//      Update & Event
//*******************************************************************
fn update(app: &App, model: &mut Model, _update: Update) {
    model.graph.set_rs(Resize::resize(app));
    let crnt_time = app.time;

    //  Read imformation from StackElapse
    read_from_ui_hndr(model);

    // Auto Load
    model
        .itxt
        .auto_load_command(&model.guiev, model.graph.graph_msg());

    //  Update Model
    model.graph.update_lpn_model(&mut model.guiev, crnt_time);

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
    draw.background().color(model.graph.get_color());

    // as you like

    //  Note Object の描画
    model.graph.view_mine(draw.clone(), tm);

    // title
    model.graph.title(draw.clone());

    // eight indicator
    model.graph.eight_indicator(draw.clone(), &model.guiev);

    // scroll text
    model.graph.scroll_text(draw.clone(), &model.itxt);

    // input text
    model
        .graph
        .input_text(draw.clone(), &model.guiev, &model.itxt, tm);

    draw.to_frame(app, &frame).unwrap();
}
