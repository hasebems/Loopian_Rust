//  Created by Hasebe Masahiko on 2024/11/03.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
mod cmd;
mod elapse;
mod file;
//mod graphic;
mod lpnlib;
mod midi;
mod server;
mod test;

use std::env;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::fs::File;
use std::io::Read;
use nannou::prelude::*;

use elapse::stack_elapse::ElapseStack;
use file::input_txt::InputText;
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
pub struct Model {
    ui_hndr: mpsc::Receiver<UiMsg>,
    itxt: InputText,
    //graph: Graphic,
    //guiev: GuiEv,
    font: nannou::text::Font,
}
fn model(app: &App) -> Model {
    let (txmsg, rxui) = gen_elapse_thread();
    app.new_window().view(view).build().unwrap();

    // フォントをロード（初期化時に一度だけ）
    let assets = app.assets_path().expect("The asset path cannot be found.");
    let font_path = assets.join("fonts").join("CourierPrime-Regular.ttf"); // フォントファイルのパスを指定

    // フォントファイルをバイト列として読み込む
    let mut file = File::open(&font_path).expect("Failed to open font file");
    let mut font_data = Vec::new();
    file.read_to_end(&mut font_data)
        .expect("Failed to load font file");

    // バイト列からフォントを作成
    let font = nannou::text::Font::from_bytes(font_data).expect("Failed to analyze font file");

    Model {
        itxt: InputText::new(txmsg),
        ui_hndr: rxui,
        //graph: Graphic::new(),
        //guiev: GuiEv::new(true),
        font,
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
//      View & Event
//*******************************************************************
fn update(_app: &App, _model: &mut Model, _update: Update) {}

struct Settings {
    resolution: u32,
    scale: f32,
    rotation: f32,
    color: Srgb<u8>,
    position: Vec2,
}
fn event(_app: &App, model: &mut Model, event: Event) {
    let mut graphmsg: Vec<i16> = Vec::new();
    model.itxt.window_event(event, &mut graphmsg);
}
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let tm = app.time;

    let settings = Settings {
        resolution: 10,
        scale: 200.0,
        rotation: 0.0,
        color: GRAY,
        position: vec2(0.0, 0.0),
    };
    let rotation_radians = deg_to_rad(settings.rotation);
    draw.ellipse()
        .resolution(settings.resolution as f32)
        .xy(settings.position)
        .color(settings.color)
        .rotate(-rotation_radians)
        .radius(settings.scale);

    // input text
    input_text(model, draw.clone(), tm);

    draw.to_frame(app, &frame).unwrap();
}
fn input_text(model: &Model, draw: Draw, tm: f32) {
    const INPUT_LOCATE_X: f32 = 0.0;    // 入力スペースの中心座標
    const INPUT_LOCATE_Y: f32 = -200.0;
    const LENGTH: f32 = 800.0;
    const HIGHT: f32 = 40.0;
    const INPUT_START_X: f32 = INPUT_LOCATE_X - LENGTH / 2.0 + 120.0;
    const LETTER_SZ_X: f32 = 16.0;
    const CURSOR_Y: f32 = INPUT_LOCATE_Y - HIGHT / 2.0;
    const CURSOR_THICKNESS: f32 = 5.0;
    const LETTER_MARGIN_X: f32 = 5.0;
    const LETTER_MARGIN_Y: f32 = 3.0;
    const PROMPT_LTR_NUM: f32 = 7.0;
    let input_bg_color: Srgb<u8> = srgb::<u8>(50, 50, 50);

    // Input Space
    draw.rect()
        .color(input_bg_color)
        .x_y(INPUT_LOCATE_X, INPUT_LOCATE_Y)
        .w_h(LENGTH, HIGHT)
        .stroke_weight(0.2)
        .stroke_color(WHITE);

    // Cursor
    if (tm % 0.5) < 0.3 {   // Cursor Blink
        draw.rect()
            .color(LIGHTGRAY)
            .x_y(
                ((model.itxt.get_cursor_locate() as f32) + 1.0) * LETTER_SZ_X + INPUT_START_X + 5.0,
                CURSOR_Y,
            )
            .w_h(LETTER_SZ_X, CURSOR_THICKNESS);
    }

    // プロンプトの描画
    let part_name: [&str; 5] = ["L1","L2","R1","R2","__",];
    let hcnt = model.itxt.get_history_cnt();
    let prompt_txt: &str = &(format!("{:03}:", hcnt) + part_name[model.itxt.get_input_part()] + ">");
    for (i, c) in prompt_txt.chars().enumerate() {
        draw.text(&c.to_string())
            .font(model.font.clone()) // 事前にロードしたフォントを使用
            .font_size(22)
            .color(MAGENTA)
            .left_justify()
            .x_y(
                (i as f32) * LETTER_SZ_X + INPUT_START_X,
                INPUT_LOCATE_Y + LETTER_MARGIN_Y,
            );
    }

    // テキストを描画
    for (i, c) in model.itxt.get_input_text().chars().enumerate() {
        draw.text(&c.to_string())
            .font(model.font.clone()) // 事前にロードしたフォントを使用
            .font_size(22)
            .color(WHITE)
            .left_justify()
            .x_y(
                ((i as f32) + PROMPT_LTR_NUM)* LETTER_SZ_X + INPUT_START_X,
                INPUT_LOCATE_Y + LETTER_MARGIN_Y,
            );
    }
}
