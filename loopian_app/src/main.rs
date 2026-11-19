//  Created by Hasebe Masahiko on 2024/11/03.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
mod cmd;
mod common;
mod elapse;
mod elapse_loop;
mod file;
mod graphic;
mod midi;
mod server;
mod test;

use bevy::app::{App as BevyApp, Plugin, PreUpdate};
use bevy::ecs::message::MessageReader;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use nannou::prelude::*;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::thread;

use cmd::input_txt::{InputText, KeyInput};
use common::lpnlib::*;
use elapse::stack_elapse::ElapseStack;
use file::settings::Settings;
use graphic::draw_graph::{Graphic, Resize};
use graphic::guiev::GuiEv;
use server::server_loop::cui_loop;

//*******************************************************************
//      Main
//*******************************************************************
const VERSION: &str = env!("CARGO_PKG_VERSION");
fn main() {
    let args: Vec<String> = env::args().collect();

    //  Version
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
        nannou::app(model)
            .update(update)
            .add_plugin(KeyInputPlugin)
            .run();
    }
}

//*******************************************************************
//      Model
//*******************************************************************
pub struct Model {
    ui_hndr: Mutex<mpsc::Receiver<UiMsg>>, // Model は Sync が必要なため Mutex で包む
    itxt: InputText,
    graph: Graphic,
    guiev: GuiEv,
    first_run: bool,
    window_px: (u32, u32), // 設定ファイルで指定された window サイズ（pixel）
    window_resize_cnt: u32, // window サイズ合わせを試みる残りフレーム数
    // as you like
}
fn model(app: &App) -> Model {
    let (txmsg, rxui) = gen_elapse_thread();

    // window の初期設定
    let first_width = Settings::load_settings().window_size.window_x_default;
    let first_height = Settings::load_settings().window_size.window_y_default;
    // サイズは起動後に adjust_window_size() で合わせる
    app.new_window::<Model>()
        .title("Loopian")
        // 垂直同期: 画面の書き換えに合わせて描画する
        // （nannou 0.20 の既定の Mailbox は macOS では使えず、垂直同期なしの Immediate になるため）
        .present_mode(PresentMode::Fifo)
        .view(view)
        .build();

    Model {
        ui_hndr: Mutex::new(rxui),
        itxt: InputText::new(txmsg),
        graph: Graphic::new(app),
        guiev: GuiEv::new(true),
        first_run: true,
        window_px: (first_width, first_height),
        window_resize_cnt: 60,
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
fn update(app: &App, model: &mut Model) {
    adjust_window_size(app, model);
    model.graph.set_rs(Resize::new(app));
    let crnt_time = app.time();

    //  Key Input
    read_key_input(model);

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
/// 設定ファイルの window サイズ（pixel）に合わせる
/// nannou 0.20 では、window を作る前に pixel で指定すると Retina の拡大率が反映されないため、
/// 起動後に拡大率を読み取り、point に換算して設定する。拡大率が確定するまで数フレーム繰り返す
fn adjust_window_size(app: &App, model: &mut Model) {
    if model.window_resize_cnt == 0 {
        return;
    }
    let win = app.main_window();
    let (px_w, px_h) = model.window_px;
    let size = win.size_pixels();
    if size.x == px_w && size.y == px_h {
        model.window_resize_cnt = 0;
        return;
    }
    let sf = win.scale_factor();
    win.set_size_points(px_w as f32 / sf, px_h as f32 / sf);
    model.window_resize_cnt -= 1;
}
fn read_from_ui_hndr(model: &mut Model) {
    let rx = model.ui_hndr.get_mut().unwrap();
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                model.itxt.sync_during_play(&msg);
                let key = model.itxt.get_indicator_key_stock();
                model.guiev.set_indicator(msg, key);
            }
            Err(TryRecvError::Disconnected) => break, // Wrong!
            Err(TryRecvError::Empty) => break,
        }
    }
}

//*******************************************************************
//      Key Input
//*******************************************************************
/// Bevy から受け取ったキー入力を、update で処理するまで貯めておく
static KEY_INPUT_QUEUE: Mutex<Vec<KeyInput>> = Mutex::new(Vec::new());

/// キー入力を受け取るプラグイン
/// nannou 0.20 の received_character はキーを離したときにも文字を渡すため、
/// Bevy の KeyboardInput を直接受け取り、押した/離した/文字を順番どおりに渡す
struct KeyInputPlugin;
impl Plugin for KeyInputPlugin {
    fn build(&self, app: &mut BevyApp) {
        app.add_systems(PreUpdate, collect_key_input);
    }
}
fn collect_key_input(mut events: MessageReader<KeyboardInput>) {
    let mut queue = KEY_INPUT_QUEUE.lock().unwrap();
    for ev in events.read() {
        queue.push(match ev.state {
            ButtonState::Pressed => {
                KeyInput::Pressed(ev.key_code, ev.text.as_ref().map(|t| t.to_string()))
            }
            ButtonState::Released => KeyInput::Released(ev.key_code),
        });
    }
}
fn read_key_input(model: &mut Model) {
    let inputs = std::mem::take(&mut *KEY_INPUT_QUEUE.lock().unwrap());
    for input in inputs {
        model.itxt.key_input(input, model.graph.graph_msg());
    }
}

//*******************************************************************
//      View
//*******************************************************************
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let tm = app.time();

    // 画面全体の背景色
    draw.background().color(model.graph.bgcolor());

    // as you like

    //  Loopian View の描画
    model
        .graph
        .view_loopian(draw.clone(), &model.guiev, &model.itxt, tm);
}
