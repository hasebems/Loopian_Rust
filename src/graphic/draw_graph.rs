//  Created by Hasebe Masahiko on 2024/11/06.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::prelude::*;
use std::fs::File;
use std::io::Read;

use super::generative_view::*;
use super::guiev::*;
use super::view_waterripple::WaterRipple;
use crate::cmd::input_txt::InputText;
use crate::cmd::txt_common;
use crate::lpnlib::*;

//*******************************************************************
//      struct Resize
//*******************************************************************
#[derive(Default, Debug, Clone)]
pub struct Resize {
    full_size_x: f32,
    full_size_y: f32,
    eight_indic_left: f32,
    scroll_txt_left: f32,
    input_txt_top: f32,
    input_txt_left: f32,
}
impl Resize {
    pub fn new(app: &App) -> Resize {
        const INPUT_TXT_LOWER_MERGIN: f32 = 80.0; // input text
        const MIN_LEFT_MERGIN: f32 = 140.0;
        const MIN_RIGHT_MERGIN: f32 = 30.0;

        let win = app.main_window();
        let win_rect = win.rect();
        let win_width = win_rect.w();
        let win_height = win_rect.h();
        let st_left_mergin = -win_width / 2.0 + MIN_LEFT_MERGIN;

        Resize {
            full_size_x: win_width,
            full_size_y: win_height,
            eight_indic_left: win_width / 2.0 - MIN_RIGHT_MERGIN,
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
    graphmsg: Vec<GraphicMsg>,
    font_nrm: nannou::text::Font,
    font_bold: nannou::text::Font,
    font_italic: nannou::text::Font,
    font_title: nannou::text::Font,
    font_newyork: nannou::text::Font,
    rs: Resize,
    svce: Option<Box<dyn GenerativeView>>, // Generaative View
    gmode: GraphMode,                      // Graph Mode  (Light or Dark)
    gptn: GraphicPattern,                  // Graph Pattern
    text_visible: TextVisible,
    crnt_time: f32,
    top_visible_line: usize,
    max_lines: usize,
    crnt_line: usize,
    title: String,
    subtitle: String,
}

//*******************************************************************
//      impl Graphic
//*******************************************************************
impl Graphic {
    const SCRTXT_FONT_HEIGHT: f32 = 25.0;
    const SCRTXT_FONT_WIDTH: f32 = 12.0;
    const SCRTXT_BOTTOM_MARGIN: f32 = 160.0;
    const TOP_MARGIN: f32 = 40.0;
    const TOP_MARGIN_WITH_TITLE: f32 = 140.0;

    pub fn new(app: &App) -> Graphic {
        // フォントをロード（初期化時に一度だけ）
        let font_nrm = Self::load_font(app, "JetBrainsMono-ExtraLight.ttf");
        let font_bold = Self::load_font(app, "JetBrainsMono-SemiBold.ttf");
        let font_italic = Self::load_font(app, "JetBrainsMono-ExtraLightItalic.ttf");
        let font_title = Self::load_font(app, "JetBrainsMono-ExtraBold.ttf");
        let font_newyork = Self::load_font(app, "NewYork.ttf");

        Self {
            graphmsg: Vec::new(),
            font_nrm,  //: font_nrm.clone(),
            font_bold, //: font_bold.clone(),
            font_italic,
            font_title,
            font_newyork,
            rs: Resize::default(),
            svce: Some(Box::new(WaterRipple::new(GraphMode::Dark))),
            gmode: GraphMode::Dark,
            gptn: GraphicPattern::Ripple,
            text_visible: TextVisible::Full,
            crnt_time: 0.0,
            top_visible_line: 0,
            max_lines: 0,
            crnt_line: 0,
            title: String::new(),
            subtitle: String::new(),
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
    pub fn graph_msg(&mut self) -> &mut Vec<GraphicMsg> {
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
            // `msg` is owned; pass by reference to the handler.
            let msg = self.graphmsg.remove(0);
            self.rcv_graph_command(guiev, crnt_time, &msg);
        }

        // Note/Beat Event を受信、generative_view へ送る
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
                            .unwrap_or(100.0)
                            .clamp(20.0, 300.0); // 20 - 300 bpm の範囲
                        let draw_time = 60.0 / bpm; // 一拍あたりの描画時間
                        if let Some(sv) = self.svce.as_mut() {
                            sv.on_beat(beat, crnt_time, draw_time);
                        }
                    }
                }
            }
            guiev.clear_graphic_ev();
        }

        // generative_view の更新
        if let Some(sv) = self.svce.as_mut() {
            sv.update_model(crnt_time, self.rs.clone());
        }

        // Scroll Text の更新
        self.update_scroll_text(itxt);
    }
    /// Graphic Command の受信
    fn rcv_graph_command(&mut self, guiev: &mut GuiEv, crnt_time: f32, msg: &GraphicMsg) {
        match msg {
            GraphicMsg::LightMode => {
                self.gmode = GraphMode::Light;
                if let Some(sv) = self.svce.as_mut() {
                    sv.set_mode(GraphMode::Light);
                }
            }
            GraphicMsg::DarkMode => {
                self.gmode = GraphMode::Dark;
                if let Some(sv) = self.svce.as_mut() {
                    sv.set_mode(GraphMode::Dark);
                }
            }
            GraphicMsg::TextVisibleCtrl => {
                self.text_visible = self.text_visible.next();
            }
            GraphicMsg::Title(title, subtitle) => {
                self.title = title.clone();
                self.subtitle = subtitle.clone();
            }
            _ => {
                // graphic pattern の変更
                let (gptn, svce) =
                    get_view_instance(guiev, crnt_time, msg, self.gmode, self.font_nrm.clone());
                if let Some(gptn) = gptn {
                    self.gptn = gptn;
                    if let Some(svce) = svce {
                        self.svce = Some(svce);
                    }
                }
            }
        }
    }
    pub fn get_bgcolor(&self) -> Srgb<u8> {
        match self.gmode {
            GraphMode::Dark => srgb::<u8>(16, 16, 16),
            GraphMode::Light => srgb::<u8>(240, 240, 240),
        }
    }
    fn update_scroll_text(&mut self, itxt: &InputText) {
        // generating max_lines_in_window, and updating self.top_scroll_line
        let scroll_texts = itxt.get_scroll_lines();
        let lines = scroll_texts.len();
        let mut top_visible_line = self.top_visible_line;
        let sz_y_limit = if self.title.is_empty() {
            Graphic::SCRTXT_BOTTOM_MARGIN + Graphic::TOP_MARGIN
        } else {
            Graphic::SCRTXT_BOTTOM_MARGIN + Graphic::TOP_MARGIN_WITH_TITLE
        };
        let max_lines_in_window =
            ((self.rs.full_size_y - sz_y_limit) as usize) / (Graphic::SCRTXT_FONT_HEIGHT as usize);
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
        // Title の表示
        self.view_title(draw.clone());

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

        // Eight Indicator の表示
        self.eight_indicator(draw.clone(), guiev);
    }
    fn view_loopian_generative_view(&self, draw: Draw, tm: f32) {
        if let Some(sv) = self.svce.as_ref() {
            sv.disp(draw.clone(), tm, self.rs.clone());
        }
    }
    /// title の描画
    fn view_title(&self, draw: Draw) {
        const TOP_MARGIN: f32 = 40.0;
        const SUB_TOP_MARGIN: f32 = 90.0;
        let title_color = self.get_text_color(false);
        if !self.subtitle.is_empty() {
            draw.text(self.subtitle.as_str())
                .font(self.font_title.clone()) // 事前にロードしたフォントを使用
                .font_size(16)
                .color(title_color)
                .center_justify()
                .x_y(0.0, self.rs.full_size_y / 2.0 - SUB_TOP_MARGIN)
                .w_h(self.rs.full_size_x - 80.0, 80.0);
        }
        if !self.title.is_empty() {
            draw.text(self.title.as_str())
                .font(self.font_title.clone()) // 事前にロードしたフォントを使用
                .font_size(36)
                .color(title_color)
                .center_justify()
                .x_y(0.0, self.rs.full_size_y / 2.0 - TOP_MARGIN)
                .w_h(self.rs.full_size_x - 80.0, 80.0);
        }
        draw.text("Loopian")
            .font(self.font_newyork.clone()) // 事前にロードしたフォントを使用
            .font_size(28)
            .color(title_color)
            .center_justify()
            .x_y(0.0, 35.0 - self.rs.full_size_y / 2.0);
        draw.text(&txt_common::get_crnt_date_txt())
            .font(self.font_nrm.clone())
            .font_size(16)
            .color(title_color)
            .x_y(
                self.rs.full_size_x / 2.0 - 100.0,
                30.0 - self.rs.full_size_y / 2.0,
            );
    }
    /// Eight Indicator の描画
    fn eight_indicator(&self, draw: Draw, guiev: &GuiEv) {
        let txt_color = self.get_text_color(false);
        let msr = guiev.get_indicator(INDC_TICK);
        let top_mergin = if self.title.is_empty() {
            Graphic::TOP_MARGIN
        } else {
            Graphic::TOP_MARGIN_WITH_TITLE
        };
        let eight_indic_top = self.rs.full_size_y / 2.0 - top_mergin;
        draw.text(msr)
            .font(self.font_bold.clone())
            .font_size(40)
            .color(txt_color)
            .left_justify()
            .x_y(self.rs.eight_indic_left - 25.0, eight_indic_top)
            .w_h(400.0, 40.0);

        let bpm = guiev.get_indicator(INDC_BPM);
        let txt_mcolor = self.get_text_color(true);
        draw.text("bpm:")
            .font(self.font_bold.clone())
            .font_size(28)
            .color(txt_mcolor)
            .left_justify()
            .x_y(self.rs.eight_indic_left, eight_indic_top - 70.0)
            .w_h(400.0, 40.0);
        draw.text(bpm)
            .font(self.font_bold.clone())
            .font_size(28)
            .color(txt_color)
            .left_justify()
            .x_y(self.rs.eight_indic_left + 120.0, eight_indic_top - 70.0)
            .w_h(400.0, 40.0);

        let meter = guiev.get_indicator(INDC_METER);
        draw.text("meter:")
            .font(self.font_bold.clone())
            .font_size(28)
            .color(txt_mcolor)
            .left_justify()
            .x_y(self.rs.eight_indic_left, eight_indic_top - 110.0)
            .w_h(400.0, 40.0);
        draw.text(meter)
            .font(self.font_bold.clone())
            .font_size(28)
            .color(txt_color)
            .left_justify()
            .x_y(self.rs.eight_indic_left + 120.0, eight_indic_top - 110.0)
            .w_h(400.0, 40.0);

        let key = guiev.get_indicator(INDC_KEY);
        draw.text("key:")
            .font(self.font_bold.clone())
            .font_size(28)
            .color(txt_mcolor)
            .left_justify()
            .x_y(self.rs.eight_indic_left, eight_indic_top - 150.0)
            .w_h(400.0, 40.0);
        draw.text(key)
            .font(self.font_bold.clone())
            .font_size(28)
            .color(txt_color)
            .left_justify()
            .x_y(self.rs.eight_indic_left + 120.0, eight_indic_top - 150.0)
            .w_h(400.0, 40.0);

        for i in 0..4 {
            let pt = guiev.get_indicator(7 - i);
            draw.text(&(guiev.get_part_txt(3 - i).to_string() + pt))
                .font(self.font_bold.clone())
                .font_size(20)
                .color(txt_color)
                .left_justify()
                .x_y(
                    self.rs.eight_indic_left,
                    eight_indic_top - 190.0 - (i as f32) * 30.0,
                )
                .w_h(400.0, 30.0);
        }
    }
    /// Input Text の描画
    fn input_text(&self, draw: Draw, guiev: &GuiEv, itxt: &InputText, tm: f32) {
        const INPUT_TXT_Y_SZ: f32 = 40.0;
        const LETTER_SZ_X: f32 = 15.0;
        const CURSOR_THICKNESS: f32 = 5.0;
        const CURSOR_HEIGHT_ADJ: f32 = 18.0;
        const LETTER_MARGIN_Y: f32 = 3.0;
        const PROMPT_LTR_NUM: f32 = 3.0; // プロンプト文字数分のスペース

        let input_txt_w_sz = self.rs.get_full_size_x() - 100.0;
        let input_bg_color: Srgb<u8> = srgb::<u8>(50, 50, 50);
        let input_locate_x: f32 = self.rs.input_txt_left; // 入力スペースの中心座標
        let input_locate_y: f32 = self.rs.input_txt_top; // 入力スペースの中心座標
        let left_edge: f32 = input_locate_x - input_txt_w_sz / 2.0;
        let input_start_x: f32 = left_edge + 10.0;

        let (input_lines, cursor_locate, cursor_line) =
            itxt.get_input_text(input_txt_w_sz - PROMPT_LTR_NUM * LETTER_SZ_X, LETTER_SZ_X);
        let lines = if input_lines.is_empty() {
            1
        } else {
            input_lines.len()
        };

        // 行数に応じて入力ボックスの中心と高さを決める（下端を固定して上方向へ伸ばす）
        let line_h = INPUT_TXT_Y_SZ;
        let box_h = (lines as f32) * line_h;
        // ボックスの中心位置（下端を固定したまま高さを伸ばすための中心計算）
        let box_center_y = input_locate_y + ((lines as f32 - 1.0) * line_h) / 2.0;

        // Input Space（高さは行数に応じる）
        draw.rect()
            .color(input_bg_color)
            .x_y(input_locate_x, box_center_y)
            .w_h(input_txt_w_sz, box_h)
            .stroke_weight(0.2)
            .stroke_color(WHITE);

        // 下端基準のテキスト描画用 Y 基準（bottom baseline）
        let bottom_y = box_center_y - box_h / 2.0 + 20.0;
        // プロンプト／カーソルのベース Y（最下行）
        let base_y = bottom_y + line_h / 2.0 + LETTER_MARGIN_Y - line_h / 2.0;

        // プロンプトの描画（最上行に揃える）
        let prompt_txt: &str = &(guiev.get_part_txt(itxt.get_input_part()).to_string() + ">");
        let txt_color = self.get_text_color(true);
        for (i, c) in prompt_txt.chars().enumerate() {
            draw.text(&c.to_string())
                .font(self.font_bold.clone()) // 事前にロードしたフォントを使用
                .font_size(24)
                .color(txt_color)
                .left_justify()
                .x_y(
                    (i as f32) * LETTER_SZ_X + input_start_x,
                    base_y + (lines as f32 - 1.0) * line_h,
                )
                .w_h(LETTER_SZ_X, line_h);
        }

        // Cursor（最下行に揃える）
        let cursor_y = base_y - CURSOR_HEIGHT_ADJ;
        if (tm % 0.5) < 0.3 {
            // Cursor Blink
            draw.rect()
                .color(LIGHTGRAY)
                .x_y(
                    (cursor_locate + PROMPT_LTR_NUM) * LETTER_SZ_X + input_start_x,
                    cursor_y + ((lines - cursor_line - 1) as f32) * line_h,
                )
                .w_h(LETTER_SZ_X, CURSOR_THICKNESS);
        }

        // テキストを描画（最下行を基準に上方向へ積む）
        let txt_color = self.get_text_color(false);
        for (l, displayed_txt) in input_lines.iter().enumerate() {
            let displayed_txt = displayed_txt.as_str();
            // 下からのオフセット（最下行が offset = 0）
            let offset_from_bottom = (lines - 1).saturating_sub(l) as f32;
            let y = base_y + offset_from_bottom * line_h;
            for (i, c) in displayed_txt.chars().enumerate() {
                draw.text(&c.to_string())
                    .font(self.font_bold.clone()) // 事前にロードしたフォントを使用
                    .font_size(24)
                    .color(txt_color)
                    .left_justify()
                    .x_y(
                        ((i as f32) + PROMPT_LTR_NUM) * LETTER_SZ_X + input_start_x,
                        y,
                    )
                    .w_h(LETTER_SZ_X, line_h);
            }
        }
        // 座標チェック用デバッグ表示
        //draw.rect().color(srgb8(255,0,0)).x_y(left_edge, base_y).w_h(4.0,4.0);
        //draw.rect().color(srgb8(0,255,0)).x_y(input_start_x, base_y).w_h(4.0,4.0);
    }
    /// Scroll Text の描画
    fn scroll_text(&self, draw: Draw, itxt: &InputText, text_visible: TextVisible) {
        const LINE_THICKNESS: f32 = 2.0;
        const SCRTXT_FONT_SIZE: u32 = 18;
        const SPACE2_TXT_LEFT_MARGIN: f32 = 40.0;

        // Draw Letters
        let top_mergin = if self.title.is_empty() {
            Graphic::TOP_MARGIN
        } else {
            Graphic::TOP_MARGIN_WITH_TITLE
        };
        let scroll_txt_top = self.rs.full_size_y / 2.0 - top_mergin;
        let top_visible_line = self.top_visible_line;
        let max_lines = self.max_lines;
        let crnt_line = self.crnt_line;
        let scroll_texts = itxt.get_scroll_lines();

        for i in 0..max_lines {
            if top_visible_line + i >= scroll_texts.len() {
                break;
            }
            let past_text_set = scroll_texts[top_visible_line + i].clone();
            let answer = past_text_set.0 == TextAttribute::Answer;
            //let past_text = past_text_set.1.clone() + &past_text_set.2;
            let past_text = past_text_set.2;
            let ltrcnt = past_text.chars().count();
            let center_adjust = ltrcnt as f32 * Graphic::SCRTXT_FONT_WIDTH / 2.0;

            // underline
            if top_visible_line + i == crnt_line {
                draw.rect()
                    .color(srgb::<u8>(200, 200, 200))
                    .x_y(
                        self.rs.scroll_txt_left + center_adjust - 60.0,
                        scroll_txt_top - Graphic::SCRTXT_FONT_HEIGHT * (i as f32) - 14.0,
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
            let tcolor = self.get_text_color(answer);
            let txt_color = Srgb::new(
                tcolor.red / alpha,
                tcolor.green / alpha,
                tcolor.blue / alpha,
            );
            let fontname = if answer {
                &self.font_italic
            } else {
                &self.font_nrm
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
                        scroll_txt_top - Graphic::SCRTXT_FONT_HEIGHT * (i as f32),
                    );
            }
        }
    }
    const ALMOST_WHITE: u8 = 230;
    fn get_text_color(&self, magenta: bool) -> Srgb<u8> {
        if magenta {
            //Srgb::new(Self::ALMOST_WHITE, 0, Self::ALMOST_WHITE)
            Srgb::new(255, 102, 204)
        } else if self.gmode == GraphMode::Light {
            Srgb::new(GRAY.red, GRAY.green, GRAY.blue)
        } else {
            Srgb::new(Self::ALMOST_WHITE, Self::ALMOST_WHITE, Self::ALMOST_WHITE)
        }
    }
}
