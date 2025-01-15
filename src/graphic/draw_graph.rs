//  Created by Hasebe Masahiko on 2024/11/06.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;
use std::fs::File;
use std::io::Read;

use super::beatlissa::*;
use super::generative_view::*;
use super::guiev::*;
use super::lissajous::*;
use super::voice4::*;
use super::waterripple::WaterRipple;
use crate::cmd::txt_common::*;
use crate::file::input_txt::InputText;
use crate::lpnlib::*;

//*******************************************************************
//      struct Resize
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
impl Resize {
    pub fn new(app: &App) -> Resize {
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
    pub fn get_full_size_x(&self) -> f32 {
        self.full_size_x
    }
    pub fn get_full_size_y(&self) -> f32 {
        self.full_size_y
    }
}
//*******************************************************************
//      struct Graphic
//*******************************************************************
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextVisible {
    Full,
    Pale,
    VeryPale,
    Invisible,
}
impl TextVisible {
    fn next(&self) -> TextVisible {
        match self {
            TextVisible::Full => TextVisible::Pale,
            TextVisible::Pale => TextVisible::VeryPale,
            TextVisible::VeryPale => TextVisible::Invisible,
            TextVisible::Invisible => TextVisible::Full,
        }
    }
}
pub struct Graphic {
    graphmsg: Vec<i16>,
    font_nrm: nannou::text::Font,
    font_italic: nannou::text::Font,
    font_newyork: nannou::text::Font,
    rs: Resize,
    svce: Option<Box<dyn GenerativeView>>, // Generaative View
    gmode: GraphMode,                      // Graph Mode  (Light or Dark)
    gptn: GraphPattern,                    // Graph Pattern
    text_visible: TextVisible,
    crnt_time: f32,
    top_visible_line: usize,
    max_lines: usize,
    crnt_line: usize,
}

//*******************************************************************
//      impl Graphic
//*******************************************************************
impl Graphic {
    const SCRTXT_FONT_HEIGHT: f32 = 25.0;
    const SCRTXT_FONT_WIDTH: f32 = 12.0;
    const SCRTXT_HEIGHT_LIMIT: f32 = 240.0;

    pub fn new(app: &App) -> Graphic {
        // フォントをロード（初期化時に一度だけ）
        let font_nrm = Self::load_font(app, "CourierPrime-Regular.ttf");
        let font_italic = Self::load_font(app, "CourierPrime-Italic.ttf");
        let font_newyork = Self::load_font(app, "NewYork.ttf");

        Self {
            graphmsg: Vec::new(),
            font_nrm: font_nrm.clone(),
            font_italic,
            font_newyork,
            rs: Resize::default(),
            svce: Some(Box::new(WaterRipple::new(GraphMode::Dark))),
            gmode: GraphMode::Dark,
            gptn: GraphPattern::Ripple,
            text_visible: TextVisible::Full,
            crnt_time: 0.0,
            top_visible_line: 0,
            max_lines: 0,
            crnt_line: 0,
        }
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
    pub fn graph_msg(&mut self) -> &mut Vec<i16> {
        &mut self.graphmsg
    }
    pub fn set_rs(&mut self, rs: Resize) {
        self.rs = rs;
    }

    //*******************************************************************
    //      Operate Events & Update Model
    //          crnt_time: [sec]
    //*******************************************************************
    pub fn update_lpn_model(&mut self, guiev: &mut GuiEv, itxt: &InputText, crnt_time: f32) {
        self.crnt_time = crnt_time;

        // 画面モードの変化イベントの受信
        if !self.graphmsg.is_empty() {
            let msg = self.graphmsg[0];
            self.rcv_graph_command(guiev, crnt_time, msg);
            self.graphmsg.remove(0);
        }

        // Note/Beat Event を受信、viewobj へ送る
        if let Some(gev) = guiev.get_graphic_ev() {
            for ev in gev {
                match ev {
                    GraphicEv::NoteEv(nev) => {
                        let nt: i32 = nev.key_num as i32;
                        let vel: i32 = nev.vel as i32;
                        let pt: i32 = nev.pt as i32;
                        if let Some(sv) = self.svce.as_mut() {
                            sv.note_on(nt, vel, pt, crnt_time);
                        }
                    }
                    GraphicEv::BeatEv(beat) => {
                        let bpm = guiev
                            .get_indicator(INDC_BPM)
                            .parse::<f32>()
                            .unwrap_or(100.0);
                        let draw_time = (60.0 / bpm) + 0.1;
                        if let Some(sv) = self.svce.as_mut() {
                            sv.on_beat(beat, crnt_time, draw_time);
                        }
                    }
                }
            }
            guiev.clear_graphic_ev();
        }

        // viewobj の更新
        if let Some(sv) = self.svce.as_mut() {
            sv.update_model(crnt_time, self.rs.clone());
        }

        // Scroll Text の更新
        self.update_scroll_text(itxt);
    }
    /// Graphic Command の受信
    fn rcv_graph_command(&mut self, guiev: &mut GuiEv, crnt_time: f32, msg: i16) {
        match msg {
            LIGHT_MODE => {
                self.gmode = GraphMode::Light;
                if let Some(sv) = self.svce.as_mut() {
                    sv.set_mode(GraphMode::Light);
                }
            }
            DARK_MODE => {
                self.gmode = GraphMode::Dark;
                if let Some(sv) = self.svce.as_mut() {
                    sv.set_mode(GraphMode::Dark);
                }
            }
            // ◆◆◆ Graphic Pattern が追加されたらここにも追加
            RIPPLE_PATTERN => {
                self.gptn = GraphPattern::Ripple;
                self.svce = Some(Box::new(WaterRipple::new(self.gmode)));
            }
            VOICE_PATTERN => {
                self.gptn = GraphPattern::Voice4;
                self.svce = Some(Box::new(Voice4::new(self.font_nrm.clone())));
            }
            LISSAJOUS_PATTERN => {
                self.gptn = GraphPattern::Lissajous;
                self.svce = Some(Box::new(Lissajous::new(self.gmode)));
            }
            BEATLISSA_PATTERN => {
                self.gptn = GraphPattern::BeatLissa;
                let mut obj = BeatLissa::new(crnt_time, self.gmode);
                let mt = guiev.get_indicator(INDC_METER).to_string();
                let num = split_by('/', mt);
                obj.set_beat_inmsr(num[0].parse::<i32>().unwrap_or(0));
                self.svce = Some(Box::new(obj));
            }
            TEXT_VISIBLE_CTRL => {
                self.text_visible = self.text_visible.next();
            }
            _ => (),
        }
    }
    pub fn get_bgcolor(&self) -> Srgb<u8> {
        match self.gmode {
            GraphMode::Dark => srgb::<u8>(0, 0, 0),
            GraphMode::Light => srgb::<u8>(255, 255, 255),
        }
    }
    fn update_scroll_text(&mut self, itxt: &InputText) {
        // generating max_lines_in_window, and updating self.top_scroll_line
        let scroll_texts = itxt.get_scroll_lines();
        let lines = scroll_texts.len();
        let mut top_visible_line = self.top_visible_line;
        let max_lines_in_window = ((self.rs.full_size_y - Graphic::SCRTXT_HEIGHT_LIMIT) as usize)
            / (Graphic::SCRTXT_FONT_HEIGHT as usize);
        let mut max_lines = max_lines_in_window;
        let max_histories = scroll_texts
            .iter()
            .filter(|x| x.0 == TextAttribute::Common)
            .collect::<Vec<_>>()
            .len();
        if lines < max_lines_in_window {
            // not filled yet
            max_lines = lines;
        }

        // Adjust top_visible_line
        let crnt_history = itxt.get_history_locate();
        let mut crnt_line: usize = lines;
        if crnt_history < max_histories {
            // 対応する履歴が全体のどの位置にあるかを調べる
            let mut linecnt = 0;
            for (i, st) in scroll_texts.iter().enumerate().take(lines) {
                if st.0 == TextAttribute::Common {
                    if linecnt == crnt_history {
                        crnt_line = i;
                        break;
                    }
                    linecnt += 1;
                }
            }
            if crnt_line < top_visible_line {
                top_visible_line = crnt_line;
            } else if crnt_line >= top_visible_line + max_lines_in_window {
                top_visible_line = crnt_line - max_lines_in_window + 1;
            }
        } else if lines >= max_lines_in_window {
            top_visible_line = lines - max_lines_in_window;
        }

        self.top_visible_line = top_visible_line;
        self.max_lines = max_lines;
        self.crnt_line = crnt_line;
    }

    //*******************************************************************
    //      View (no mutable self)
    //*******************************************************************
    pub fn view_loopian(&self, draw: Draw, guiev: &GuiEv, itxt: &InputText, tm: f32) {
        // Scroll Text の表示
        if self.text_visible != TextVisible::Full && self.text_visible != TextVisible::Invisible {
            self.scroll_text(draw.clone(), itxt, self.text_visible);
        }

        // Gererative Pattern
        self.view_loopian_generative_view(draw.clone(), tm);

        // Input Text 表示
        if self.text_visible != TextVisible::Invisible && self.text_visible == TextVisible::Full {
            self.scroll_text(draw.clone(), itxt, self.text_visible);
        }
        if self.text_visible != TextVisible::Invisible && self.text_visible != TextVisible::VeryPale
        {
            self.input_text(draw.clone(), guiev, itxt, tm);
        }
        self.title(draw.clone());
        self.eight_indicator(draw.clone(), guiev);
    }
    fn view_loopian_generative_view(&self, draw: Draw, tm: f32) {
        if let Some(sv) = self.svce.as_ref() {
            sv.disp(draw.clone(), tm, self.rs.clone());
        }
    }
    /// title の描画
    fn title(&self, draw: Draw) {
        let title_color = if self.gmode == GraphMode::Light {
            GRAY
        } else {
            WHITE
        };
        draw.text("Loopian")
            .font(self.font_newyork.clone()) // 事前にロードしたフォントを使用
            .font_size(32)
            .color(title_color)
            .center_justify()
            .x_y(0.0, 42.0 - self.rs.full_size_y / 2.0);
    }
    /// Eight Indicator の描画
    fn eight_indicator(&self, draw: Draw, guiev: &GuiEv) {
        let txt_color = if self.gmode == GraphMode::Light {
            GRAY
        } else {
            WHITE
        };
        let msr = guiev.get_indicator(INDC_TICK);
        draw.text(msr)
            .font(self.font_nrm.clone())
            .font_size(40)
            .color(txt_color)
            .left_justify()
            .x_y(self.rs.eight_indic_left, self.rs.eight_indic_top)
            .w_h(400.0, 40.0);

        let bpm = guiev.get_indicator(INDC_BPM);
        draw.text("bpm:")
            .font(self.font_nrm.clone())
            .font_size(28)
            .color(MAGENTA)
            .left_justify()
            .x_y(
                self.rs.eight_indic_left + 40.0,
                self.rs.eight_indic_top - 70.0,
            )
            .w_h(400.0, 40.0);
        draw.text(bpm)
            .font(self.font_nrm.clone())
            .font_size(28)
            .color(txt_color)
            .left_justify()
            .x_y(
                self.rs.eight_indic_left + 170.0,
                self.rs.eight_indic_top - 70.0,
            )
            .w_h(400.0, 40.0);

        let meter = guiev.get_indicator(INDC_METER);
        draw.text("meter:")
            .font(self.font_nrm.clone())
            .font_size(28)
            .color(MAGENTA)
            .left_justify()
            .x_y(
                self.rs.eight_indic_left + 40.0,
                self.rs.eight_indic_top - 110.0,
            )
            .w_h(400.0, 40.0);
        draw.text(meter)
            .font(self.font_nrm.clone())
            .font_size(28)
            .color(txt_color)
            .left_justify()
            .x_y(
                self.rs.eight_indic_left + 170.0,
                self.rs.eight_indic_top - 110.0,
            )
            .w_h(400.0, 40.0);

        let key = guiev.get_indicator(INDC_KEY);
        draw.text("key:")
            .font(self.font_nrm.clone())
            .font_size(28)
            .color(MAGENTA)
            .left_justify()
            .x_y(
                self.rs.eight_indic_left + 40.0,
                self.rs.eight_indic_top - 150.0,
            )
            .w_h(400.0, 40.0);
        draw.text(key)
            .font(self.font_nrm.clone())
            .font_size(28)
            .color(txt_color)
            .left_justify()
            .x_y(
                self.rs.eight_indic_left + 170.0,
                self.rs.eight_indic_top - 150.0,
            )
            .w_h(400.0, 40.0);

        for i in 0..4 {
            let pt = guiev.get_indicator(7 - i);
            draw.text(&(guiev.get_part_txt(3 - i).to_string() + pt))
                .font(self.font_nrm.clone())
                .font_size(20)
                .color(txt_color)
                .left_justify()
                .x_y(
                    self.rs.eight_indic_left + 40.0,
                    self.rs.eight_indic_top - 190.0 - (i as f32) * 30.0,
                )
                .w_h(400.0, 30.0);
        }
    }
    /// Input Text の描画
    fn input_text(&self, draw: Draw, guiev: &GuiEv, itxt: &InputText, tm: f32) {
        const INPUT_TXT_X_SZ: f32 = 1240.0;
        const INPUT_TXT_Y_SZ: f32 = 40.0;
        const LETTER_SZ_X: f32 = 16.0;
        const CURSOR_THICKNESS: f32 = 5.0;
        const LETTER_MARGIN_Y: f32 = 3.0;
        const PROMPT_LTR_NUM: f32 = 7.0;

        let input_bg_color: Srgb<u8> = srgb::<u8>(50, 50, 50);
        let input_locate_x: f32 = self.rs.input_txt_left; // 入力スペースの中心座標
        let input_locate_y: f32 = self.rs.input_txt_top; // 入力スペースの中心座標
        let input_start_x: f32 = input_locate_x - INPUT_TXT_X_SZ / 2.0 + 120.0;
        let cursor_y: f32 = input_locate_y - INPUT_TXT_Y_SZ / 2.0 + 2.0;
        let cursor_locate: f32 = itxt.get_cursor_locate() as f32;

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
        let hcnt = itxt.get_history_locate() % 1000;
        let prompt_txt: &str =
            &(format!("{:03}:", hcnt) + guiev.get_part_txt(itxt.get_input_part()) + ">");
        for (i, c) in prompt_txt.chars().enumerate() {
            draw.text(&c.to_string())
                .font(self.font_nrm.clone()) // 事前にロードしたフォントを使用
                .font_size(22)
                .color(MAGENTA)
                .left_justify()
                .x_y(
                    (i as f32) * LETTER_SZ_X + input_start_x,
                    input_locate_y + LETTER_MARGIN_Y,
                );
        }

        // テキストを描画
        for (i, c) in itxt.get_input_text().chars().enumerate() {
            draw.text(&c.to_string())
                .font(self.font_nrm.clone()) // 事前にロードしたフォントを使用
                .font_size(22)
                .color(WHITE)
                .left_justify()
                .x_y(
                    ((i as f32) + PROMPT_LTR_NUM) * LETTER_SZ_X + input_start_x,
                    input_locate_y + LETTER_MARGIN_Y,
                );
        }
    }
    /// Scroll Text の描画
    fn scroll_text(&self, draw: Draw, itxt: &InputText, text_visible: TextVisible) {
        const LINE_THICKNESS: f32 = 2.0;
        const SCRTXT_FONT_SIZE: u32 = 18;
        const SPACE2_TXT_LEFT_MARGIN: f32 = 40.0;

        // Draw Letters
        let top_visible_line = self.top_visible_line;
        let max_lines = self.max_lines;
        let crnt_line = self.crnt_line;
        let scroll_texts = itxt.get_scroll_lines();
        for i in 0..max_lines {
            if top_visible_line + i >= scroll_texts.len() {
                break;
            }
            let past_text_set = scroll_texts[top_visible_line + i].clone();
            let past_text = past_text_set.1.clone() + &past_text_set.2;
            let ltrcnt = past_text.chars().count();
            let center_adjust = ltrcnt as f32 * Graphic::SCRTXT_FONT_WIDTH / 2.0;

            // underline
            if top_visible_line + i == crnt_line {
                draw.rect()
                    .color(LIGHTGRAY)
                    .x_y(
                        self.rs.scroll_txt_left + center_adjust - 60.0,
                        self.rs.scroll_txt_top - Graphic::SCRTXT_FONT_HEIGHT * (i as f32) - 14.0,
                    )
                    .w_h(Graphic::SCRTXT_FONT_WIDTH * (ltrcnt as f32), LINE_THICKNESS);
            }

            // string
            let alpha = match text_visible {
                TextVisible::Full => 1,
                TextVisible::Pale => 2,
                TextVisible::VeryPale => 3,
                TextVisible::Invisible => 0,
            };
            let (txt_color, fontname) = if past_text_set.0 == TextAttribute::Answer {
                let magenta_with_alpha = Srgb::new(
                    MAGENTA.red / alpha,
                    MAGENTA.green / alpha,
                    MAGENTA.blue / alpha,
                );
                (magenta_with_alpha, &self.font_italic)
            } else if self.gmode == GraphMode::Light {
                let gray_with_alpha =
                    Srgb::new(GRAY.red / alpha, GRAY.green / alpha, GRAY.blue / alpha);
                (gray_with_alpha, &self.font_nrm)
            } else {
                let white_with_alpha =
                    Srgb::new(WHITE.red / alpha, WHITE.green / alpha, WHITE.blue / alpha);
                (white_with_alpha, &self.font_nrm)
            };
            for (j, d) in past_text.chars().enumerate() {
                draw.text(&d.to_string())
                    .font(fontname.clone())
                    .font_size(SCRTXT_FONT_SIZE)
                    .color(txt_color)
                    .left_justify()
                    .x_y(
                        self.rs.scroll_txt_left
                            + SPACE2_TXT_LEFT_MARGIN
                            + Graphic::SCRTXT_FONT_WIDTH * (j as f32),
                        self.rs.scroll_txt_top - Graphic::SCRTXT_FONT_HEIGHT * (i as f32),
                    );
            }
        }
    }
}
