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
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use elapse::stack_elapse::ElapseStack;
use file::input_txt::InputText;
use graphic::guiev::GuiEv;
use graphic::noteobj::NoteObj;
use graphic::waterripple::WaterRipple;
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
#[derive(Default, Debug, Clone)]
pub struct Resize {
    full_size_x: f32,
    full_size_y: f32,
    eight_indic_top: f32,
    eight_indic_left: f32,
    scroll_txt_top: f32,
    scroll_txt_left: f32,
    input_txt_top: f32,
    input_txt_left: f32,
}
pub struct Model {
    ui_hndr: mpsc::Receiver<UiMsg>,
    itxt: InputText,
    guiev: GuiEv,
    crnt_time: f32,

    font_nrm: nannou::text::Font,
    font_italic: nannou::text::Font,
    font_newyork: nannou::text::Font,
    rs: Resize,
    nobj: Vec<Box<dyn NoteObj>>,
    graphmsg: Vec<i16>,
    gmode: GraphMode,
    gptn: GraphPattern,
}
fn model(app: &App) -> Model {
    let (txmsg, rxui) = gen_elapse_thread();
    app.new_window().view(view).build().unwrap();

    // フォントをロード（初期化時に一度だけ）
    let font_nrm = load_font(app, "CourierPrime-Regular.ttf");
    let font_italic = load_font(app, "CourierPrime-Italic.ttf");
    let font_newyork = load_font(app, "NewYork.ttf");

    // app に対する初期設定
    app.set_exit_on_escape(false);
    let win = app.main_window();
    win.set_title("Loopian");
    win.set_inner_size_pixels(2800, 1800);

    Model {
        ui_hndr: rxui,
        itxt: InputText::new(txmsg),
        //graph: Graphic::new(),
        guiev: GuiEv::new(true),
        crnt_time: 0.0,

        font_nrm,
        font_italic,
        font_newyork,
        rs: Resize::default(),
        nobj: Vec::new(),
        graphmsg: Vec::new(),
        gmode: GraphMode::Dark,
        gptn: GraphPattern::Ripple,
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
fn load_font(app: &App, font_path: &str) -> nannou::text::Font {
    let assets = app.assets_path().expect("The asset path cannot be found.");
    let font_path = assets.join("fonts").join(font_path); // フォントファイルのパスを指定

    // フォントファイルをバイト列として読み込む
    let mut file = File::open(&font_path).expect("Failed to open font file");
    let mut font_data = Vec::new();
    file.read_to_end(&mut font_data)
        .expect("Failed to load font file");

    // バイト列からフォントを作成
    nannou::text::Font::from_bytes(font_data).expect("Failed to analyze font file")
}

//*******************************************************************
//      Update & Event
//*******************************************************************
fn update(app: &App, model: &mut Model, _update: Update) {
    model.rs = resize(app);
    model.crnt_time = app.time;

    //  Read imformation from StackElapse
    read_from_ui_hndr(model);

    // Auto Load
    model
        .itxt
        .auto_load_command(&model.guiev, &mut model.graphmsg);

    // 画面モードの設定
    if model.graphmsg.len() > 0 {
        let msg = model.graphmsg[0];
        match msg {
            LIGHT_MODE => model.gmode = GraphMode::Light,
            DARK_MODE => model.gmode = GraphMode::Dark,
            RIPPLE_PATTERN => model.gptn = GraphPattern::Ripple,
            VOICE_PATTERN => model.gptn = GraphPattern::Voice,
            _ => (),
        }
        model.graphmsg.remove(0);
    }

    // Note Object の更新
    if let Some(gev) = model.guiev.get_graphic_ev() {
        for ev in gev {
            let nt: i32 = ev.key_num as i32;
            let vel: i32 = ev.vel as i32;
            let pt: i32 = ev.pt as i32;
            push_note_obj(model, nt, vel, pt, model.crnt_time);
        }
        model.guiev.clear_graphic_ev();
    }
    let nlen = model.nobj.len();
    let mut rls = vec![true; nlen];
    for (i, obj) in model.nobj.iter_mut().enumerate() {
        rls[i] = if !obj.update_model(model.crnt_time, model.rs.clone()) {
            false
        } else {
            true
        };
    }
    for i in 0..nlen {
        if !rls[i] {
            model.nobj.remove(i);
            break;
        }
    }
}
fn resize(app: &App) -> Resize {
    const EIGHT_INDIC_TOP: f32 = 40.0; // eight indicator
    const SCROLL_TXT_TOP: f32 = 80.0; // scroll text
    const INPUT_TXT_LOWER_MERGIN: f32 = 80.0; // input text
    const MIN_LEFT_MERGIN: f32 = 140.0;
    const MIN_RIGHT_MERGIN: f32 = 140.0;

    let win = app.main_window();
    let win_rect = win.rect();
    let win_width = win_rect.w();
    let win_height = win_rect.h();
    let st_left_mergin = -win_width / 2.0 + MIN_LEFT_MERGIN;

    Resize {
        full_size_x: win_width,
        full_size_y: win_height,
        eight_indic_top: win_height / 2.0 - EIGHT_INDIC_TOP,
        eight_indic_left: win_width / 2.0 - MIN_RIGHT_MERGIN,
        scroll_txt_top: win_height / 2.0 - SCROLL_TXT_TOP,
        scroll_txt_left: st_left_mergin,
        input_txt_top: -win_height / 2.0 + INPUT_TXT_LOWER_MERGIN,
        input_txt_left: 0.0,
    }
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
fn push_note_obj(model: &mut Model, nt: i32, vel: i32, _pt: i32, tm: f32) {
    model.nobj.push(Box::new(WaterRipple::new(
        nt as f32,
        vel as f32,
        tm,
        model.gmode,
    )));
}
fn event(_app: &App, model: &mut Model, event: Event) {
    model.itxt.window_event(event, &mut model.graphmsg);
}

//*******************************************************************
//      View
//*******************************************************************
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let tm = app.time;

    // 画面全体の背景色
    draw.background().color(get_color(model.gmode));

    //  Note Object の描画
    view_mine(model, draw.clone(), tm);

    // draw title
    draw.text("Loopian")
        .font(model.font_newyork.clone()) // 事前にロードしたフォントを使用
        .font_size(32)
        .color(WHITE)
        .center_justify()
        .x_y(0.0, 42.0 - model.rs.full_size_y / 2.0);

    // eight indicator
    eight_indicator(model, draw.clone());

    // scroll text
    scroll_text(model, draw.clone());

    // input text
    input_text(model, draw.clone(), tm);

    draw.to_frame(app, &frame).unwrap();
}
fn view_mine(model: &Model, draw: Draw, tm: f32) {
    //  Note Object の描画
    for obj in model.nobj.iter() {
        obj.disp(draw.clone(), tm, model.rs.clone());
    }
}
fn get_color(gmode: GraphMode) -> Srgb<u8> {
    match gmode {
        GraphMode::Dark => srgb::<u8>(0, 0, 0),
        GraphMode::Light => srgb::<u8>(255, 255, 255),
    }
}

//*******************************************************************
//      Display Text
//*******************************************************************
fn input_text(model: &Model, draw: Draw, tm: f32) {
    const INPUT_TXT_X_SZ: f32 = 1240.0;
    const INPUT_TXT_Y_SZ: f32 = 40.0;
    const LETTER_SZ_X: f32 = 16.0;
    const CURSOR_THICKNESS: f32 = 5.0;
    const LETTER_MARGIN_Y: f32 = 3.0;
    const PROMPT_LTR_NUM: f32 = 7.0;

    let input_bg_color: Srgb<u8> = srgb::<u8>(50, 50, 50);
    let input_locate_x: f32 = model.rs.input_txt_left; // 入力スペースの中心座標
    let input_locate_y: f32 = model.rs.input_txt_top; // 入力スペースの中心座標
    let input_start_x: f32 = input_locate_x - INPUT_TXT_X_SZ / 2.0 + 120.0;
    let cursor_y: f32 = input_locate_y - INPUT_TXT_Y_SZ / 2.0 + 2.0;
    let cursor_locate: f32 = model.itxt.get_cursor_locate() as f32;

    // Input Space
    draw.rect()
        .color(input_bg_color)
        .x_y(input_locate_x, input_locate_y)
        .w_h(INPUT_TXT_X_SZ, INPUT_TXT_Y_SZ)
        .stroke_weight(0.2)
        .stroke_color(WHITE);

    // Cursor
    if (tm % 0.5) < 0.3 {
        // Cursor Blink
        draw.rect()
            .color(LIGHTGRAY)
            .x_y(
                (cursor_locate + 1.0) * LETTER_SZ_X + input_start_x + 5.0,
                cursor_y,
            )
            .w_h(LETTER_SZ_X, CURSOR_THICKNESS);
    }

    // プロンプトの描画
    let hcnt = model.itxt.get_history_cnt();
    let prompt_txt: &str =
        &(format!("{:03}:", hcnt) + model.guiev.get_part_txt(model.itxt.get_input_part()) + ">");
    for (i, c) in prompt_txt.chars().enumerate() {
        draw.text(&c.to_string())
            .font(model.font_nrm.clone()) // 事前にロードしたフォントを使用
            .font_size(22)
            .color(MAGENTA)
            .left_justify()
            .x_y(
                (i as f32) * LETTER_SZ_X + input_start_x,
                input_locate_y + LETTER_MARGIN_Y,
            );
    }

    // テキストを描画
    for (i, c) in model.itxt.get_input_text().chars().enumerate() {
        draw.text(&c.to_string())
            .font(model.font_nrm.clone()) // 事前にロードしたフォントを使用
            .font_size(22)
            .color(WHITE)
            .left_justify()
            .x_y(
                ((i as f32) + PROMPT_LTR_NUM) * LETTER_SZ_X + input_start_x,
                input_locate_y + LETTER_MARGIN_Y,
            );
    }
}
fn scroll_text(model: &Model, draw: Draw) {
    const LINE_THICKNESS: f32 = 2.0;
    const SCRTXT_FONT_SIZE: u32 = 18;
    const SCRTXT_FONT_HEIGHT: f32 = 25.0;
    const SCRTXT_FONT_WIDTH: f32 = 12.0;
    const SPACE2_TXT_LEFT_MARGIN: f32 = 40.0;
    const SCRTXT_HEIGHT_LIMIT: f32 = 200.0;

    // generating max_line_in_window, and updating self.top_scroll_line
    let scroll_lines = model.itxt.get_scroll_lines();
    let lines = scroll_lines.len();
    let mut top_scroll_line = 0;
    let max_line_in_window =
        ((model.rs.full_size_y - SCRTXT_HEIGHT_LIMIT) as usize) / (SCRTXT_FONT_HEIGHT as usize);
    let mut crnt_line: usize = lines;
    let mut max_disp_line = max_line_in_window;
    let max_history = scroll_lines
        .iter()
        .filter(|x| x.0 == TextAttribute::Common)
        .collect::<Vec<_>>()
        .len();

    if lines < max_line_in_window {
        // not filled yet
        max_disp_line = lines;
    }
    let crnt_history = model.itxt.get_history_cnt();
    if crnt_history < max_history {
        crnt_line = 0;
        for i in 0..lines {
            if scroll_lines[i].0 == TextAttribute::Common {
                if crnt_line == crnt_history {
                    crnt_line = i;
                    break;
                }
                crnt_line += 1;
            }
        }
        if crnt_line < top_scroll_line {
            top_scroll_line = crnt_line;
        } else if crnt_line > top_scroll_line + max_line_in_window - 1 {
            top_scroll_line = crnt_line - max_line_in_window + 1;
        }
    } else if lines >= max_line_in_window {
        top_scroll_line = lines - max_line_in_window;
    }

    // Draw Letters
    for i in 0..max_disp_line {
        let past_text_set = scroll_lines[top_scroll_line + i].clone();
        let past_text = past_text_set.1.clone() + &past_text_set.2;
        let ltrcnt = past_text.chars().count();
        let center_adjust = ltrcnt as f32 * SCRTXT_FONT_WIDTH / 2.0;

        // line
        if top_scroll_line + i == crnt_line {
            draw.rect()
                .color(LIGHTGRAY)
                .x_y(
                    model.rs.scroll_txt_left + center_adjust - 60.0,
                    model.rs.scroll_txt_top - SCRTXT_FONT_HEIGHT * (i as f32) - 14.0,
                )
                .w_h(SCRTXT_FONT_WIDTH * (ltrcnt as f32), LINE_THICKNESS);
        }

        // string
        let (txt_color, fontname) = if past_text_set.0 == TextAttribute::Answer {
            (MAGENTA, &model.font_italic)
        } else if model.gmode == GraphMode::Light {
            (GRAY, &model.font_nrm)
        } else {
            (WHITE, &model.font_nrm)
        };
        for (j, d) in past_text.chars().enumerate() {
            draw.text(&d.to_string())
                .font(fontname.clone())
                .font_size(SCRTXT_FONT_SIZE)
                .color(txt_color)
                .left_justify()
                .x_y(
                    model.rs.scroll_txt_left
                        + SPACE2_TXT_LEFT_MARGIN
                        + SCRTXT_FONT_WIDTH * (j as f32),
                    model.rs.scroll_txt_top - SCRTXT_FONT_HEIGHT * (i as f32),
                );
        }
    }
}
fn eight_indicator(model: &Model, draw: Draw) {
    let txt_color = if model.gmode == GraphMode::Light {
        GRAY
    } else {
        WHITE
    };
    let msr = model.guiev.get_indicator(3);
    draw.text(msr)
        .font(model.font_nrm.clone())
        .font_size(40)
        .color(txt_color)
        .left_justify()
        .x_y(model.rs.eight_indic_left, model.rs.eight_indic_top)
        .w_h(400.0, 40.0);

    let bpm = model.guiev.get_indicator(1);
    draw.text("bpm:")
        .font(model.font_nrm.clone())
        .font_size(28)
        .color(MAGENTA)
        .left_justify()
        .x_y(
            model.rs.eight_indic_left + 40.0,
            model.rs.eight_indic_top - 70.0,
        )
        .w_h(400.0, 40.0);
    draw.text(bpm)
        .font(model.font_nrm.clone())
        .font_size(28)
        .color(txt_color)
        .left_justify()
        .x_y(
            model.rs.eight_indic_left + 170.0,
            model.rs.eight_indic_top - 70.0,
        )
        .w_h(400.0, 40.0);

    let meter = model.guiev.get_indicator(2);
    draw.text("meter:")
        .font(model.font_nrm.clone())
        .font_size(28)
        .color(MAGENTA)
        .left_justify()
        .x_y(
            model.rs.eight_indic_left + 40.0,
            model.rs.eight_indic_top - 110.0,
        )
        .w_h(400.0, 40.0);
    draw.text(meter)
        .font(model.font_nrm.clone())
        .font_size(28)
        .color(txt_color)
        .left_justify()
        .x_y(
            model.rs.eight_indic_left + 170.0,
            model.rs.eight_indic_top - 110.0,
        )
        .w_h(400.0, 40.0);

    let key = model.guiev.get_indicator(0);
    draw.text("key:")
        .font(model.font_nrm.clone())
        .font_size(28)
        .color(MAGENTA)
        .left_justify()
        .x_y(
            model.rs.eight_indic_left + 40.0,
            model.rs.eight_indic_top - 150.0,
        )
        .w_h(400.0, 40.0);
    draw.text(key)
        .font(model.font_nrm.clone())
        .font_size(28)
        .color(txt_color)
        .left_justify()
        .x_y(
            model.rs.eight_indic_left + 170.0,
            model.rs.eight_indic_top - 150.0,
        )
        .w_h(400.0, 40.0);

    for i in 0..4 {
        let pt = model.guiev.get_indicator(7 - i);
        draw.text(&(model.guiev.get_part_txt(3 - i).to_string() + pt))
            .font(model.font_nrm.clone())
            .font_size(20)
            .color(txt_color)
            .left_justify()
            .x_y(
                model.rs.eight_indic_left + 40.0,
                model.rs.eight_indic_top - 190.0 - (i as f32) * 30.0,
            )
            .w_h(400.0, 30.0);
    }
}
