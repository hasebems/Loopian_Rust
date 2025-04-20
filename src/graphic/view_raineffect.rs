//  Created by Hasebe Masahiko on 2025/01/08.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php

use crate::graphic::generative_view::GenerativeView;
use nannou::prelude::*;
use std::collections::VecDeque;
//use crate::graphic::draw_graph::*;
use super::draw_graph::Resize;
use crate::lpnlib::GraphMode;

//*******************************************************************
//      Rain Effect Graphic
//*******************************************************************
pub struct RainEffect {
    mode: GraphMode,
    lines: VecDeque<(Point2, Point2, f32)>, // (start, end, alpha)
    last_time: f32,
    rain_dencity: f32, // 0.1 - 1.0
    counter: i32,
    thinning_rate: i32,
}

impl RainEffect {
    const FADE_RATE: f32 = 0.05; // Rate at which lines fade out
    const DENSITY_FADE_RATE: f32 = 0.95; // Rate at which density fades out
    const RAIN_DIAGONAL_WIDTH: f32 = 50.0; // Width of the rain lines

    pub fn new(mode: GraphMode) -> Self {
        Self {
            mode,
            lines: VecDeque::new(),
            last_time: 0.0,
            rain_dencity: 0.1,
            counter: 0,
            thinning_rate: 4,
        }
    }
    fn add_one_line(&mut self, rs: &Resize) {
        // Add a new line with full opacity
        let storm = Self::RAIN_DIAGONAL_WIDTH * (self.rain_dencity - 0.5) * 2.0;
        let start_x = random_range(-rs.get_full_size_x() / 2.0, rs.get_full_size_x() / 2.0);
        let end_x = start_x + random_range(-storm, storm);
        let start_y = rs.get_full_size_y() / 2.0;
        let end_y = -rs.get_full_size_y() / 2.0;
        self.lines
            .push_back((pt2(start_x, start_y), pt2(end_x, end_y), 1.0));
    }
    fn update_rate(&mut self) {
        // Update the thinning rate based on the current density
        if self.rain_dencity < 0.5 {
            // 0.5 以下なら間引き
            if self.rain_dencity < 0.1 {
                self.rain_dencity = 0.1;
            } else {
                let p = ((0.5 - self.rain_dencity) * 5.0).round() as usize;
                self.thinning_rate = pow(2, p); //1,2,4
            }
        } else {
            // 0.5 より大きいなら線を増やす
            self.thinning_rate = 1;
        }
    }
}

impl GenerativeView for RainEffect {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        // Check if enough time has passed to add a new line
        if crnt_time - self.last_time < 0.1 {
            return; // Skip if not enough time has passed
        }
        self.counter += 1;
        self.last_time = crnt_time;
        if self.counter % 10 == 0 {
            //println!("RainEffect: {}-{}", self.rain_dencity, self.thinning_rate);
        }

        // Add new random lines
        if self.thinning_rate > 1 {
            if self.counter % self.thinning_rate == 0 {
                self.add_one_line(&rs);
            }
        } else {
            let max = ((self.rain_dencity * 10.0) - 4.0) as usize;
            for _ in 0..max {
                self.add_one_line(&rs);
            }
        }

        // Fade out old lines
        for line in &mut self.lines {
            line.2 -= Self::FADE_RATE; // Reduce alpha
        }

        // Remove fully transparent lines
        self.lines.retain(|line| line.2 > 0.0);

        // Update density
        self.rain_dencity *= Self::DENSITY_FADE_RATE; // Fade out density
        self.update_rate();
    }

    fn note_on(&mut self, _nt: i32, vel: i32, _pt: i32, _tm: f32) {
        // No specific behavior for note_on in this effect
        let diff = 1.0 - self.rain_dencity;
        self.rain_dencity = 1.0 - diff * (1.0 - 0.5 * ((vel as f32) / 127.0));
        self.update_rate();
    }

    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }

    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        _rs: Resize,
    ) {
        let md = if self.mode == GraphMode::Light {
            GRAY
        } else {
            WHITE
        };
        for (start, end, alpha) in &self.lines {
            let alpha_int = (*alpha * 255.0).clamp(0.0, 255.0) as u8;
            draw.line()
                .start(*start)
                .end(*end)
                .color(rgba(md.red, md.green, md.blue, alpha_int)); // Changed color to white
        }
    }
}
