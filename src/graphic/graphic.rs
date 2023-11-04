//  Created by Hasebe Masahiko on 2023/10/31.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use eframe::{egui,egui::*};
use std::time::Instant;
use crate::{WINDOW_X, WINDOW_Y};
use crate::lpnlib::*;
use crate::cmd::cmdparse::LoopianCmd;
use super::waterripple::WaterRipple;

const MAZENTA: Color32 = Color32::from_rgb(255, 0, 255);
const TEXT_GRAY: Color32 = Color32::from_rgb(0,0,0);
const _TEXT_BG: Color32 = Color32::from_rgb(0,200,200);

const BACK_WHITE: Color32 = Color32::from_rgb(180, 180, 180);
const BACK_WHITE2: Color32 = Color32::from_rgb(128,128,128);
const _BACK_MAZENTA: Color32 = Color32::from_rgb(180, 160, 180);
const _BACK_GRAY: Color32 = Color32::from_rgb(48,48,48);
const BACK_DARK_GRAY: Color32 = Color32::from_rgb(32,32,32);
const BACK_GRAY2: Color32 = Color32::from_rgb(160, 160, 160);

const FONT16: f32 = 16.0;
const FONT16_WIDTH: f32 = 9.56;
const SPACE_LEFT: f32 = 30.0;
const SPACE_RIGHT: f32 = 970.0;

const _LEFT_MERGIN: f32 = 5.0;

pub struct Graphic {
    full_size: Pos2,
    nobj: Vec<WaterRipple>,
    start_time: Instant,
    frame_counter: i32,
}

impl Graphic {
    pub fn new() -> Graphic {
        Self {
            full_size: Pos2 {x:WINDOW_X, y: WINDOW_Y},
            nobj: Vec::new(),
            start_time: Instant::now(),
            frame_counter: 0,
        }
    }
    pub fn update(&mut self, ui: &mut Ui, 
        infs : (usize, &String, &Vec<(String, String)>, usize, &LoopianCmd),
        frame: &mut eframe::Frame, ntev: Option<String>) {

            // window size を得る
        self.full_size.x = frame.info().window_info.size.x;
        self.full_size.y = frame.info().window_info.size.y;

        // frame_counter の更新
        const FPS: i32 = 1000/50;
        let time = self.start_time.elapsed();
        self.frame_counter = (time.as_millis() as i32)/FPS;

        //  Note Object の描画
        if let Some(kmsg) = ntev {
            let nt_vel = split_by('/', kmsg);
            let nt: i32 = nt_vel[0].parse().unwrap();
            let vel: i32 = nt_vel[1].parse().unwrap();
            self.nobj.push(WaterRipple::new(nt as f32, vel as f32, self.frame_counter));
        }
        let nlen = self.nobj.len();
        let mut rls = vec![true; nlen];
        for (i, obj) in self.nobj.iter_mut().enumerate() {
            if obj.disp(self.frame_counter, ui) == false {
                rls[i] = false;
            }
        }
        for i in 0..nlen {  // 一度に一つ消去
            if !rls[i] {self.nobj.remove(i); break;}
        }

        // Title 描画
        self.update_title(ui);

        // Eight Indicator 描画
        self.update_eight_indicator(ui, infs.4);

        // Scroll Text 描画
        self.update_scroll_text(ui, infs.2);

        // Input Text 描画
        self.update_input_text(ui, infs.0, infs.1, infs.3, infs.4);
    }
    //*******************************************************************
    //      Update Screen
    //*******************************************************************
    fn update_title(&self, ui: &mut egui::Ui) {
        ui.put(
            Rect {
                min: Pos2 { x:self.full_size.x/2.0 - 40.0,
                            y:self.full_size.y - 50.0},
                max: Pos2 { x:self.full_size.x/2.0 + 40.0, 
                            y:self.full_size.y - 10.0},
            }, //  location
            Label::new(RichText::new("Loopian")
                .size(24.0)
                .color(Color32::WHITE)
                .family(FontFamily::Proportional)
            )
        );
    }
    //*******************************************************************
    fn update_eight_indicator(&mut self, ui: &mut egui::Ui, cmd: &LoopianCmd) {

        const NEXT_BLOCK: f32 = 235.0;      // (SPACE_RIGHT - SPACE_LEFT)/4
        const SPACE1_NEXT: f32 = 50.0;
        const SPACE1_LEFT_ADJ: f32 = 20.0;  // (NEXT_BLOCK - BLOCK_LENGTH)/2
        const SPACE1_UPPER: f32 = 80.0;     // eight indicator
        const BLOCK_LENGTH: f32 = 195.0;
        const SPACE1_HEIGHT: f32 = 30.0;

        let input_part = cmd.get_input_part();
        for i in 0..4 {
            for j in 0..2 {
                let mut back_color = BACK_WHITE;
                if i as usize != input_part && j == 1 {back_color = BACK_WHITE2;}
                let raw: f32 = NEXT_BLOCK*(i as f32);
                let line: f32 = SPACE1_NEXT*(j as f32);
                ui.painter().rect_filled(
                    Rect { min: Pos2 {x:SPACE_LEFT + SPACE1_LEFT_ADJ + raw,
                                      y:SPACE1_UPPER + line}, 
                           max: Pos2 {x:SPACE_LEFT + SPACE1_LEFT_ADJ + BLOCK_LENGTH + raw,
                                      y:SPACE1_UPPER + SPACE1_HEIGHT + line},}, //  location
                    8.0,                //  curve
                    back_color,     //  color
                );
                let tx = self.text_for_eight_indicator(i + j*4, cmd);
                let ltrcnt = tx.chars().count();
                for k in 0..ltrcnt {
                    ui.put(Rect {
                        min: Pos2 {
                            x:SPACE_LEFT + SPACE1_LEFT_ADJ + 10.0 + raw + FONT16_WIDTH*(k as f32),
                            y:SPACE1_UPPER + 2.0 + line},
                        max: Pos2 {
                            x:SPACE_LEFT + SPACE1_LEFT_ADJ + 10.0 + raw + FONT16_WIDTH*((k+1) as f32),
                            y:SPACE1_UPPER + 27.0 + line},},
                        Label::new(RichText::new(&tx[k..k+1])
                            .size(FONT16).color(TEXT_GRAY)
                            .family(FontFamily::Monospace).text_style(TextStyle::Monospace))
                    );
                }
            }
        }
    }
    fn text_for_eight_indicator(&mut self, num: i32, cmd: &LoopianCmd) -> String {
        let indi_txt;
        match num {
            0 => indi_txt = "key: ".to_string() + cmd.get_indicator(0),
            1 => indi_txt = "bpm: ".to_string() + cmd.get_indicator(1),
            2 => indi_txt = "beat:".to_string() + cmd.get_indicator(2),
            4 => indi_txt = "L1:".to_string() + cmd.get_indicator(4),
            5 => indi_txt = "L2:".to_string() + cmd.get_indicator(5),
            6 => indi_txt = "R1:".to_string() + cmd.get_indicator(6),
            7 => indi_txt = "R2:".to_string() + cmd.get_indicator(7),
            3 => indi_txt = cmd.get_indicator(3).to_string(),
            _ => indi_txt = "".to_string(),
        }
        indi_txt
    }
    //*******************************************************************
    fn update_scroll_text(&self, ui: &mut egui::Ui, scroll_lines: &Vec<(String,String)>) {
        const MAX_SCROLL_LINES: usize = 20;
        const SPACE2_TXT_LEFT_MARGIN: f32 = 40.0;
        const SPACE2_UPPER: f32 = 200.0;    // scroll text
        const FONT16_HEIGHT: f32 = 25.0;

        // Paint Letter Space
//        ui.painter().rect_filled(
//            Rect::from_min_max( pos2(Self::SPACE_LEFT, Self::SPACE2_UPPER),
//                                pos2(Self::SPACE_RIGHT, Self::SPACE2_LOWER)),
//            2.0,                //  curve
//            Self::BACK_GRAY     //  color
//        );

        let lines = scroll_lines.len();
        let max_count = if lines < MAX_SCROLL_LINES {lines} else {MAX_SCROLL_LINES};
        let ofs_count = if lines < MAX_SCROLL_LINES {0} else {lines - MAX_SCROLL_LINES};
        // Draw Letters
        for i in 0..max_count {
            let past_text_set = scroll_lines[ofs_count+i].clone();
            let past_text = past_text_set.0.clone() + &past_text_set.1;
            let cnt = past_text.chars().count();
            let txt_color = if i%2==0 {Color32::WHITE} else {MAZENTA};
            ui.put(
                Rect { 
                    min: Pos2 {x:SPACE_LEFT + SPACE2_TXT_LEFT_MARGIN,
                               y:SPACE2_UPPER + FONT16_HEIGHT*(i as f32)}, 
                    max: Pos2 {x:SPACE_LEFT + SPACE2_TXT_LEFT_MARGIN + FONT16_WIDTH*(cnt as f32),
                               y:SPACE2_UPPER + FONT16_HEIGHT*((i+1) as f32)},},
                Label::new(RichText::new(&past_text)
                    .size(FONT16)
                    .color(txt_color)
                    .family(FontFamily::Monospace)
                )
            );
        }
    }
    //*******************************************************************
    fn update_input_text(&mut self, ui: &mut egui::Ui,
        input_locate: usize, input_text: &String, history_cnt: usize, cmd: &LoopianCmd) {

            const CURSOR_LEFT_MARGIN: f32 = 10.0;
        const CURSOR_LOWER_MERGIN: f32 = 6.0;
//        const CURSOR_TXT_LENGTH: f32 = 9.55;  // FONT 16p
        const CURSOR_TXT_LENGTH: f32 = 11.95;   // FONT 20p
        const CURSOR_THICKNESS: f32 = 4.0;
        const PROMPT_LETTERS: usize = 8;

        const INPUTTXT_UPPER_MARGIN: f32 = 0.0;
        const INPUTTXT_LOWER_MARGIN: f32 = 0.0;

        const INPUTTXT_FONT_SIZE: f32 = 20.0;
        const INPUTTXT_LETTER_WIDTH: f32 = 11.95;
        const PROMPT_MERGIN: f32 = INPUTTXT_LETTER_WIDTH*(PROMPT_LETTERS as f32);
        const INPUT_MERGIN_OFFSET: f32 = 3.25;
        const INPUT_MERGIN: f32 = PROMPT_MERGIN + INPUT_MERGIN_OFFSET;

        const SPACE3_UPPER: f32 = 760.0;    // input text
        const SPACE3_LOWER: f32 = 800.0;

        const SPACE3_TXT_LEFT_MARGIN: f32 = 5.0;

        // Paint Letter Space
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(SPACE_LEFT,SPACE3_UPPER),
                               pos2(SPACE_RIGHT,SPACE3_LOWER)),
            2.0,                       //  curve
            BACK_DARK_GRAY     //  color
        );

        // Paint cursor
        let cursor = input_locate + PROMPT_LETTERS;
        let elapsed_time = self.start_time.elapsed().as_millis();
        if elapsed_time%500 > 200 {
            ui.painter().rect_filled(
                Rect { min: Pos2 {x:SPACE_LEFT + CURSOR_LEFT_MARGIN + CURSOR_TXT_LENGTH*(cursor as f32),
                                y:SPACE3_LOWER - CURSOR_LOWER_MERGIN},
                       max: Pos2 {x:SPACE_LEFT + CURSOR_LEFT_MARGIN + CURSOR_TXT_LENGTH*((cursor+1) as f32) - 2.0,
                                y:SPACE3_LOWER - CURSOR_LOWER_MERGIN + CURSOR_THICKNESS},},
                0.0,                              //  curve
                BACK_GRAY2,  //  color
            );
        }

        // Draw Letters
        let mut hcnt = history_cnt;
        if hcnt >= 1000 {hcnt %= 1000;}
        let prompt_txt: &str = &(format!("{:03}: ", hcnt) + cmd.get_part_txt());

        // Prompt Text
        ui.put(
            Rect { 
                   min: Pos2 {x:SPACE_LEFT + SPACE3_TXT_LEFT_MARGIN - 2.0,
                              y:SPACE3_UPPER + INPUTTXT_UPPER_MARGIN},
                   max: Pos2 {x:SPACE_LEFT + SPACE3_TXT_LEFT_MARGIN + PROMPT_MERGIN,
                              y:SPACE3_LOWER + INPUTTXT_LOWER_MARGIN},},
            Label::new(RichText::new(prompt_txt)
                .size(INPUTTXT_FONT_SIZE)
                .color(MAZENTA)
                .family(FontFamily::Monospace))
        );
        // User Input
        let ltrcnt = input_text.chars().count();
        for i in 0..ltrcnt {    // 位置を合わせるため、１文字ずつ Label を作って並べて配置する
            ui.put(
                Rect { 
                    min: Pos2 {x:SPACE_LEFT + SPACE3_TXT_LEFT_MARGIN + INPUT_MERGIN + 
                                 INPUTTXT_LETTER_WIDTH*(i as f32),
                               y:SPACE3_UPPER + INPUTTXT_UPPER_MARGIN},
                    max: Pos2 {x:SPACE_LEFT + SPACE3_TXT_LEFT_MARGIN + INPUT_MERGIN + 
                                 INPUTTXT_LETTER_WIDTH*((i+1) as f32),
                               y:SPACE3_LOWER + INPUTTXT_LOWER_MARGIN},},
                Label::new(RichText::new(input_text[i..i+1].to_string())
                    .size(INPUTTXT_FONT_SIZE)
                    .color(Color32::WHITE)
                    .family(FontFamily::Monospace)
                    .text_style(TextStyle::Monospace))
            );
        }
    }
}