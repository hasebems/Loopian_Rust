//  Created by Hasebe Masahiko on 2025/04/05.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::draw_graph::*;
use super::generative_view::{GenerativeView, GraphMode};
use nannou::prelude::*;
use std::collections::VecDeque;

//*******************************************************************
//      Screen Graphic
//*******************************************************************
struct NoteInfo(f32, f32, f32);

pub struct SineWave {
    mode: GraphMode,
    note_info: VecDeque<NoteInfo>,
    last_update_time: f32,
    view: VecDeque<f32>,
}

impl SineWave {
    const NUM_POINTS: usize = 512;
    const DECAY_RATE: f32 = 1.0 / 8.0;
    const AMPLITUDE: f32 = 100.0;
    const SPEED: f32 = 10.0;
    const MAX_NOTE_INFO: usize = 16;

    pub fn new(mode: GraphMode) -> Self {
        let view = VecDeque::from(vec![0.0; Self::NUM_POINTS + 1]); // VecDequeで初期化
        Self {
            mode,
            note_info: VecDeque::with_capacity(Self::MAX_NOTE_INFO),
            last_update_time: 0.0,
            view,
        }
    }

    fn note_on(&mut self, nt: i32, vel: i32, tm: f32) {
        if nt < 0 || !(0..=127).contains(&vel) {
            eprintln!("Invalid note or velocity: nt={}, vel={}", nt, vel);
            return;
        }
        let freq = (nt as f32 / 8.0).powf(2.0);
        self.note_info
            .push_back(NoteInfo(freq, vel as f32 / 127.0, tm));
        if self.note_info.len() > Self::MAX_NOTE_INFO {
            self.note_info.pop_front();
        }
    }

    fn update(&mut self, crnt_time: f32, _rs: Resize) {
        while self.last_update_time + 0.1 / Self::SPEED < crnt_time {
            self.last_update_time += 0.1 / Self::SPEED;
            let mut y = 0.0;
            for each_ni in &self.note_info {
                let elapsed_time = (self.last_update_time - each_ni.2).max(0.0);
                if elapsed_time < 4.0 {
                    let freq = each_ni.0;
                    let amp = each_ni.1 * Self::DECAY_RATE.powf(elapsed_time);
                    let phase = elapsed_time * freq;
                    y += amp * phase.sin();
                }
            }
            self.view.pop_front(); // 先頭要素を削除
            self.view.push_back(y); // 末尾に追加
        }
    }

    fn disp(&self, draw: Draw, rs: Resize) {
        let color = if self.mode == GraphMode::Light {
            GRAY
        } else {
            WHITE
        };
        let width = rs.get_full_size_x();
        let step = width / Self::NUM_POINTS as f32;
        let mut points = Vec::with_capacity(Self::NUM_POINTS + 1);
        for i in 0..=Self::NUM_POINTS {
            let x = i as f32 * step - width / 2.0;
            points.push(pt2(x, self.view[i] * Self::AMPLITUDE));
        }
        draw.polyline().weight(2.0).points(points).color(color);
    }
}

impl GenerativeView for SineWave {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        self.update(crnt_time, rs);
    }

    fn note_on(&mut self, nt: i32, vel: i32, _pt: i32, tm: f32) {
        self.note_on(nt, vel, tm);
    }

    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }

    fn disp(&self, draw: Draw, _crnt_time: f32, rs: Resize) {
        self.disp(draw, rs);
    }
}
