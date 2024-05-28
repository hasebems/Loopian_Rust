//  Created by Hasebe Masahiko on 2023/10/31.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::noteobj::NoteObj;
use super::voice::Voice4;
use super::waterripple::WaterRipple;
use crate::cmd::cmdparse::LoopianCmd;
use crate::cmd::txt_common::*;
use crate::lpnlib::*;
use crate::setting::*;
use eframe::{egui, egui::*};
use rand::{rngs, thread_rng, Rng};
use std::time::Instant;

const MAZENTA: Color32 = Color32::from_rgb(255, 0, 255);
const TEXT_GRAY: Color32 = Color32::from_rgb(0, 0, 0);
const _TEXT_BG: Color32 = Color32::from_rgb(0, 200, 200);

const BACK_WHITE0: Color32 = Color32::from_rgb(220, 220, 220); // LIGHTの明るいグレー
const BACK_WHITE1: Color32 = Color32::from_rgb(180, 180, 180); // DARKの明るいグレー
const BACK_WHITE2: Color32 = Color32::from_rgb(128, 128, 128); // DARKの薄暗いグレー
const _BACK_MAZENTA: Color32 = Color32::from_rgb(180, 160, 180);
const _BACK_GRAY: Color32 = Color32::from_rgb(48, 48, 48);
const BACK_DARK_GRAY: Color32 = Color32::from_rgb(32, 32, 32);
const BACK_GRAY2: Color32 = Color32::from_rgb(160, 160, 160);
const FONT16: f32 = 16.0;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextAttribute {
    Common,
    Answer,
}
pub struct Graphic {
    full_size: Pos2,
    nobj: Vec<Box<dyn NoteObj>>,
    start_time: Instant,
    frame_counter: i32,          // one per 20msec
    _frame_counter_old_dbg: i32, // debug
    rndm: rngs::ThreadRng,
    mode: i16,
    note_ptn: i16,
    top_scroll_line: usize,
    _last_location: usize,
}
struct Resize {
    eight_indic_top: f32,
    eight_indic_left: f32,
    scroll_txt_top: f32,
    scroll_txt_left: f32,
    input_txt_top: f32,
    input_txt_left: f32,
}
impl Graphic {
    pub fn new() -> Graphic {
        Self {
            full_size: Pos2 {
                x: WINDOW_X_DEFAULT,
                y: WINDOW_Y_DEFAULT,
            },
            nobj: Vec::new(),
            start_time: Instant::now(),
            frame_counter: 0,
            _frame_counter_old_dbg: 0,
            rndm: thread_rng(),
            mode: DARK_MODE,
            note_ptn: RIPPLE_PATTERN,
            top_scroll_line: 0,
            _last_location: 0,
        }
    }
    pub fn update(
        &mut self,
        ui: &mut Ui,
        infs: (
            usize,                                 // cursor position
            &String,                               // input text
            &Vec<(TextAttribute, String, String)>, // scroll text(TextAttribute::Common/Answer, time, text)
            usize,                                 // selected scroll text line
            &LoopianCmd,                           // eight indicator
        ),
        msg: i16,
        _frame: &mut eframe::Frame,
        ntev: Vec<String>,
    ) {
        if msg != NO_MSG {
            match msg {
                DARK_MODE => self.mode = DARK_MODE,
                LIGHT_MODE => self.mode = LIGHT_MODE,
                RIPPLE_PATTERN => self.note_ptn = RIPPLE_PATTERN,
                VOICE_PATTERN => self.note_ptn = VOICE_PATTERN,
                _ => {}
            }
        }

        // window size を得る
        let new_x = ui.available_size().x;
        let new_y = ui.available_size().y;
        if new_x != self.full_size.x {
            self.full_size.x = new_x;
            println!("New Window Size X={}", new_x);
        }
        if new_y != self.full_size.y {
            self.full_size.y = new_y;
            println!("New Window Size Y={}", new_y);
        }
        let rs = self.resize();

        // frame_counter の更新
        const FPS: i32 = 1000 / 50;
        let time = self.start_time.elapsed();
        self.frame_counter = (time.as_millis() as i32) / FPS;

        //  Note Object の描画
        for ev in ntev.iter() {
            let nt_vel = split_by('/', ev.clone());
            let nt: i32 = nt_vel[0].parse().unwrap_or(0);
            let vel: i32 = nt_vel[1].parse().unwrap_or(0);
            let pt: i32 = nt_vel[2].parse().unwrap_or(0);
            let rnd: f32 = self.rndm.gen();
            self.push_note_obj(nt, vel, pt, rnd);
        }
        let nlen = self.nobj.len();
        let mut rls = vec![true; nlen];
        for (i, obj) in self.nobj.iter_mut().enumerate() {
            if obj.disp(self.frame_counter, ui, self.full_size) == false {
                rls[i] = false;
            }
        }
        for i in 0..nlen {
            // 一度に一つ消去
            if !rls[i] {
                self.nobj.remove(i);
                break;
            }
        }

        // Title 描画
        self.update_title(ui);

        // Eight Indicator 描画
        self.update_eight_indicator(ui, infs.4, &rs);

        // Scroll Text 描画
        self.update_scroll_text(ui, infs.2, infs.3, &rs);

        // Input Text 描画
        self.update_input_text(ui, infs, &rs);
    }
    pub fn back_color(&self) -> Color32 {
        if self.mode == DARK_MODE {
            Color32::BLACK
        } else {
            Color32::WHITE
        }
    }
    fn letter_color(&self) -> Color32 {
        if self.mode == DARK_MODE {
            Color32::WHITE
        } else {
            Color32::BLACK
        }
    }
    fn light_box_color(&self) -> Color32 {
        if self.mode == DARK_MODE {
            BACK_WHITE1
        } else {
            BACK_WHITE0
        }
    }
    fn resize(&self) -> Resize {
        const EIGHT_INDIC_TOP: f32 = 40.0; // eight indicator
        const SCROLL_TXT_TOP: f32 = 200.0; // scroll text
        const INPUT_TXT_TOP_SZ: f32 = 100.0; // input text
        const MIN_LEFT_MERGIN: f32 = 140.0;
        let it_left_mergin = (self.full_size.x - 940.0) / 2.0;
        let mut st_left_mertin = 0.0;
        if self.full_size.x > 1200.0 {
            st_left_mertin = 200.0;
        } else if self.full_size.x > 1000.0 {
            st_left_mertin = self.full_size.x - 1000.0;
        }
        Resize {
            eight_indic_top: EIGHT_INDIC_TOP,
            eight_indic_left: MIN_LEFT_MERGIN,
            scroll_txt_top: SCROLL_TXT_TOP,
            scroll_txt_left: st_left_mertin,
            input_txt_top: self.full_size.y - INPUT_TXT_TOP_SZ,
            input_txt_left: it_left_mergin,
        }
    }
    fn push_note_obj(&mut self, nt: i32, vel: i32, pt: i32, rnd: f32) {
        match self.note_ptn {
            RIPPLE_PATTERN => self.nobj.push(Box::new(WaterRipple::new(
                nt as f32,
                vel as f32,
                rnd,
                self.frame_counter,
                self.mode,
            ))),
            VOICE_PATTERN => self.nobj.push(Box::new(Voice4::new(
                nt as f32,
                vel as f32,
                pt,
                self.frame_counter,
                self.mode,
            ))),
            _ => {}
        }
    }
    //*******************************************************************
    //      Update Screen
    //*******************************************************************
    fn update_title(&self, ui: &mut egui::Ui) {
        ui.put(
            Rect {
                min: Pos2 {
                    x: self.full_size.x / 2.0 - 50.0,
                    y: self.full_size.y - 50.0,
                },
                max: Pos2 {
                    x: self.full_size.x / 2.0 + 50.0,
                    y: self.full_size.y - 10.0,
                },
            }, //  location
            Label::new(
                RichText::new("Loopian")
                    .size(28.0)
                    .color(self.letter_color())
                    .family(FontFamily::Proportional),
            ),
        );
    }
    //*******************************************************************
    fn update_eight_indicator(&mut self, ui: &mut egui::Ui, cmd: &LoopianCmd, rs: &Resize) {
        const SPACE1_NEXT: f32 = 50.0;
        const BLOCK_LENGTH: f32 = 200.0;
        const BLOCK_HEIGHT: f32 = 30.0;
        const MIN_MERGIN: f32 = 20.0; // (NEXT_BLOCK - BLOCK_LENGTH)/2
        const EI_FONT16_WIDTH: f32 = 9.56;

        let mut interval: f32 = 240.0;
        let mut min_left: f32 = rs.eight_indic_left;
        if self.full_size.x > 1000.0 {
            let times = (self.full_size.x - 1000.0) / 1500.0 + 1.0;
            let center = self.full_size.x / 2.0;
            interval = (BLOCK_LENGTH + MIN_MERGIN) * times;
            min_left = center - interval * 1.5;
        }

        let input_part = cmd.get_input_part();
        let mut back_color;
        for i in 0..MAX_INDICATOR / 2 {
            for j in 0..2 {
                back_color = self.light_box_color();
                if i as usize != input_part && j == 1 {
                    back_color = BACK_WHITE2;
                }

                let raw: f32 = interval * (i as f32);
                let line: f32 = SPACE1_NEXT * (j as f32);
                ui.painter().rect_filled(
                    Rect {
                        min: Pos2 {
                            x: min_left + raw - BLOCK_LENGTH / 2.0,
                            y: rs.eight_indic_top + line,
                        },
                        max: Pos2 {
                            x: min_left + raw - BLOCK_LENGTH / 2.0 + BLOCK_LENGTH,
                            y: rs.eight_indic_top + BLOCK_HEIGHT + line,
                        },
                    }, //  location
                    8.0,        //  curve
                    back_color, //  color
                );
                let tx = self.text_for_eight_indicator(i + j * 4, cmd);
                let ltrcnt = tx.chars().count();
                for k in 0..ltrcnt {
                    ui.put(
                        Rect {
                            min: Pos2 {
                                x: min_left + raw - BLOCK_LENGTH / 2.0
                                    + 10.0
                                    + EI_FONT16_WIDTH * (k as f32),
                                y: rs.eight_indic_top + 2.0 + line,
                            },
                            max: Pos2 {
                                x: min_left + raw - BLOCK_LENGTH / 2.0
                                    + 10.0
                                    + EI_FONT16_WIDTH * ((k + 1) as f32),
                                y: rs.eight_indic_top + 27.0 + line,
                            },
                        },
                        Label::new(
                            RichText::new(&tx[k..k + 1])
                                .size(FONT16)
                                .color(TEXT_GRAY)
                                .family(FontFamily::Monospace)
                                .text_style(TextStyle::Monospace),
                        ),
                    );
                }
            }
        }
    }
    fn text_for_eight_indicator(&mut self, num: usize, cmd: &LoopianCmd) -> String {
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
    fn update_scroll_text(
        &mut self,
        ui: &mut egui::Ui,
        scroll_lines: &Vec<(TextAttribute, String, String)>,
        crnt_history: usize,
        rs: &Resize,
    ) {
        const SPACE2_TXT_LEFT_MARGIN: f32 = 40.0;
        const FONT16_HEIGHT: f32 = 25.0;
        const FONT16_WIDTH: f32 = 10.0;

        // generating max_line_in_window, and updating self.top_scroll_line
        let letter_color = self.letter_color();
        let lines = scroll_lines.len();
        let max_line_in_window = ((self.full_size.y - 340.0) as usize) / 25;
        let mut crnt_line: usize = lines;
        let mut max_disp_line = max_line_in_window;
        let max_history = scroll_lines
            .iter()
            .filter(|x| x.0 == TextAttribute::Common)
            .collect::<Vec<_>>()
            .len();

        if lines < max_line_in_window {
            // not filled yet
            self.top_scroll_line = 0;
            max_disp_line = lines;
        }
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
            if crnt_line < self.top_scroll_line {
                self.top_scroll_line = crnt_line;
            } else if crnt_line > self.top_scroll_line + max_line_in_window - 1 {
                self.top_scroll_line = crnt_line - max_line_in_window + 1;
            }
        } else if lines >= max_line_in_window {
            self.top_scroll_line = lines - max_line_in_window;
        }

        // debug
        //        if self.frame_counter > self._frame_counter_old_dbg + 50 || self._last_location != lines {
        //            println!("crnt_history:{}, line:{}, top:{}, max:{}", crnt_history, crnt_line, self.top_scroll_line, max_history);
        //            self._frame_counter_old_dbg = self.frame_counter;
        //            self._last_location = lines;
        //        }

        // Draw Letters
        for i in 0..max_disp_line {
            let past_text_set = scroll_lines[self.top_scroll_line + i].clone();
            let past_text = past_text_set.1.clone() + &past_text_set.2;
            let ltrcnt = past_text.chars().count();

            // line
            if self.top_scroll_line + i == crnt_line {
                ui.painter().rect_filled(
                    Rect {
                        min: Pos2 {
                            x: rs.scroll_txt_left + SPACE2_TXT_LEFT_MARGIN,
                            y: rs.scroll_txt_top + FONT16_HEIGHT * (i as f32) + 21.0,
                        },
                        max: Pos2 {
                            x: rs.scroll_txt_left
                                + SPACE2_TXT_LEFT_MARGIN
                                + FONT16_WIDTH * (ltrcnt as f32),
                            y: rs.scroll_txt_top + FONT16_HEIGHT * (i as f32) + 23.0,
                        },
                    },
                    0.0,        //  curve
                    BACK_GRAY2, //  color
                );
            }

            // string
            let txt_color = if past_text_set.0 == TextAttribute::Answer {
                MAZENTA
            } else {
                letter_color
            };
            for j in 0..ltrcnt {
                // 位置を合わせるため、１文字ずつ Label を作って並べて配置する
                ui.put(
                    Rect {
                        min: Pos2 {
                            x: rs.scroll_txt_left
                                + SPACE2_TXT_LEFT_MARGIN
                                + FONT16_WIDTH * (j as f32),
                            y: rs.scroll_txt_top + FONT16_HEIGHT * (i as f32),
                        },
                        max: Pos2 {
                            x: rs.scroll_txt_left
                                + SPACE2_TXT_LEFT_MARGIN
                                + FONT16_WIDTH * ((j as f32) + 1.0),
                            y: rs.scroll_txt_top + FONT16_HEIGHT * ((i + 1) as f32),
                        },
                    },
                    Label::new(
                        RichText::new(past_text[j..j + 1].to_string())
                            .size(FONT16)
                            .color(txt_color)
                            .family(FontFamily::Monospace),
                    ),
                );
            }
        }
    }
    //*******************************************************************
    fn update_input_text(
        &mut self,
        ui: &mut egui::Ui,
        infs: (
            usize,                                 // cursor position
            &String,                               // input text
            &Vec<(TextAttribute, String, String)>, // scroll text
            usize,                                 // selected scroll text line
            &LoopianCmd,                           // eight indicator
        ),
        rs: &Resize,
    ) {
        const CURSOR_LEFT_MARGIN: f32 = 10.0;
        const CURSOR_LOWER_MERGIN: f32 = 6.0;
        //        const CURSOR_TXT_LENGTH: f32 = 9.55;  // FONT 16p
        const CURSOR_TXT_LENGTH: f32 = 11.95; // FONT 20p
        const CURSOR_THICKNESS: f32 = 4.0;
        const PROMPT_LETTERS: usize = 8; // "000: R1>"

        const INPUTTXT_UPPER_MARGIN: f32 = 0.0;
        const INPUTTXT_LOWER_MARGIN: f32 = 0.0;

        const INPUTTXT_FONT_SIZE: f32 = 20.0;
        const INPUTTXT_LETTER_WIDTH: f32 = 11.95;
        const PROMPT_MERGIN: f32 = INPUTTXT_LETTER_WIDTH * (PROMPT_LETTERS as f32);
        const INPUT_MERGIN_OFFSET: f32 = 3.25;
        const INPUT_MERGIN: f32 = PROMPT_MERGIN + INPUT_MERGIN_OFFSET;

        const INPUT_TXT_Y_SZ: f32 = 40.0;
        const INPUT_TXT_X_SZ: f32 = 940.0;

        const SPACE3_TXT_LEFT_MARGIN: f32 = 5.0;

        // Paint Letter Space
        ui.painter().rect_filled(
            Rect::from_min_max(
                pos2(rs.input_txt_left, rs.input_txt_top),
                pos2(
                    rs.input_txt_left + INPUT_TXT_X_SZ,
                    rs.input_txt_top + INPUT_TXT_Y_SZ,
                ),
            ),
            2.0,            //  curve
            BACK_DARK_GRAY, //  color
        );

        // Paint cursor
        let cursor = infs.0 + PROMPT_LETTERS;
        let elapsed_time = self.start_time.elapsed().as_millis();
        if elapsed_time % 500 > 200 {
            ui.painter().rect_filled(
                Rect {
                    min: Pos2 {
                        x: rs.input_txt_left
                            + CURSOR_LEFT_MARGIN
                            + CURSOR_TXT_LENGTH * (cursor as f32),
                        y: rs.input_txt_top + INPUT_TXT_Y_SZ - CURSOR_LOWER_MERGIN,
                    },
                    max: Pos2 {
                        x: rs.input_txt_left
                            + CURSOR_LEFT_MARGIN
                            + CURSOR_TXT_LENGTH * ((cursor + 1) as f32)
                            - 2.0,
                        y: rs.input_txt_top + INPUT_TXT_Y_SZ - CURSOR_LOWER_MERGIN
                            + CURSOR_THICKNESS,
                    },
                },
                0.0,        //  curve
                BACK_GRAY2, //  color
            );
        }

        // Draw Letters
        let mut hcnt = infs.3;
        if hcnt >= 1000 {
            hcnt %= 1000;
        }
        let prompt_txt: &str = &(format!("{:03}: ", hcnt) + infs.4.get_part_txt() + ">");

        // Prompt Text
        ui.put(
            Rect {
                min: Pos2 {
                    x: rs.input_txt_left + SPACE3_TXT_LEFT_MARGIN - 2.0,
                    y: rs.input_txt_top + INPUTTXT_UPPER_MARGIN,
                },
                max: Pos2 {
                    x: rs.input_txt_left + SPACE3_TXT_LEFT_MARGIN + PROMPT_MERGIN,
                    y: rs.input_txt_top + INPUT_TXT_Y_SZ + INPUTTXT_LOWER_MARGIN,
                },
            },
            Label::new(
                RichText::new(prompt_txt)
                    .size(INPUTTXT_FONT_SIZE)
                    .color(MAZENTA)
                    .family(FontFamily::Monospace),
            ),
        );
        // User Input
        let ltrcnt = infs.1.chars().count();
        for i in 0..ltrcnt {
            // 位置を合わせるため、１文字ずつ Label を作って並べて配置する
            ui.put(
                Rect {
                    min: Pos2 {
                        x: rs.input_txt_left
                            + SPACE3_TXT_LEFT_MARGIN
                            + INPUT_MERGIN
                            + INPUTTXT_LETTER_WIDTH * (i as f32),
                        y: rs.input_txt_top + INPUTTXT_UPPER_MARGIN,
                    },
                    max: Pos2 {
                        x: rs.input_txt_left
                            + SPACE3_TXT_LEFT_MARGIN
                            + INPUT_MERGIN
                            + INPUTTXT_LETTER_WIDTH * ((i + 1) as f32),
                        y: rs.input_txt_top + INPUT_TXT_Y_SZ + INPUTTXT_LOWER_MARGIN,
                    },
                },
                Label::new(
                    RichText::new(infs.1[i..i + 1].to_string())
                        .size(INPUTTXT_FONT_SIZE)
                        .color(Color32::WHITE)
                        .family(FontFamily::Monospace)
                        .text_style(TextStyle::Monospace),
                ),
            );
        }
    }
}
