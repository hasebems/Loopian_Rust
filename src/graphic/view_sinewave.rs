//  Created by Hasebe Masahiko on 2025/04/05.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::graphic::draw_graph::*;
use crate::graphic::generative_view::GenerativeView;
use crate::lpnlib::GraphMode;
use nannou::prelude::*;
use std::collections::VecDeque;

//*******************************************************************
//      Screen Graphic
//*******************************************************************
struct NoteInfo(f32, f32, f32);

pub struct SineWave {
    mode: GraphMode,
    amplitude: f32,
    speed: f32,
    note_info: VecDeque<NoteInfo>,
}

impl SineWave {
    const NUM_POINTS: usize = 512;
    const DECAY_RATE: f32 = 1.0 / 8.0;

    pub fn new(mode: GraphMode) -> Self {
        Self {
            mode,
            amplitude: 200.0,
            speed: 2.0,
            note_info: VecDeque::with_capacity(32),
        }
    }

    fn _note_on(&mut self, nt: i32, vel: i32, _pt: i32, tm: f32) {
        if nt < 0 || vel < 0 || vel > 127 {
            eprintln!("Invalid note or velocity: nt={}, vel={}", nt, vel);
            return;
        }
        let freq = (nt as f32 / 8.0).powf(2.0);
        self.note_info.push_back(NoteInfo(freq, vel as f32 / 127.0, tm));
        if self.note_info.len() > 32 {
            self.note_info.pop_front();
        }
    }

    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        let color = if self.mode == GraphMode::Light { GRAY } else { WHITE };

        let width = rs.get_full_size_x();
        let step = width / Self::NUM_POINTS as f32;

        let mut points = Vec::with_capacity(Self::NUM_POINTS + 1);
        for i in 0..=Self::NUM_POINTS {
            let x = i as f32 * step - width / 2.0;
            let mut y = 0.0;
            let x_sec = (10.0 / self.speed) * (Self::NUM_POINTS - i) as f32 / Self::NUM_POINTS as f32;
            for each_ni in &self.note_info {
                let elapsed_time = (crnt_time - each_ni.2 - x_sec).max(0.0);
                if elapsed_time < 4.0 {
                    let freq = each_ni.0;
                    let amp = each_ni.1 * Self::DECAY_RATE.powf(elapsed_time);
                    let phase = elapsed_time * freq;
                    y += amp * phase.sin();
                }
            }
            points.push(pt2(x, y * self.amplitude));
        }

        draw.polyline().weight(2.0).points(points).color(color);
    }
}

impl GenerativeView for SineWave {
    fn update_model(&mut self, _crnt_time: f32, _rs: Resize) {
        //self.phase = crnt_time * self.speed;
    }

    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, tm: f32) {
        if nt < 0 || vel < 0 || vel > 127 {
            eprintln!("Invalid note or velocity: nt={}, vel={}", nt, vel);
            return;
        }
        let freq = (nt as f32 / 8.0).powf(2.0);
        self.note_info.push_back(NoteInfo(freq, vel as f32 / 127.0, tm));
        if self.note_info.len() > 32 {
            self.note_info.pop_front();
        }
    }

    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }

    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        self.disp(draw, crnt_time, rs);
    }
}
