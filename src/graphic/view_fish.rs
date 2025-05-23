//  Created by Hasebe Masahiko on 2025/05/17.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php

use super::draw_graph::Resize;
use crate::graphic::generative_view::GenerativeView;
use crate::lpnlib::GraphMode;
use nannou::prelude::*;

// 定数定義を追加（必要に応じてモジュールやスコープに配置）
const TIME_INTERVAL: f32 = 5.0;
const MAX_FISHES: usize = 100;
const FISH_DRAW_STEPS: usize = 100;

//*******************************************************************
//      School of Fish Graphic
//*******************************************************************
pub struct SchoolOfFish {
    fishes: Vec<Fish>,
    past_time: f32,
    density: f32,
}
impl SchoolOfFish {
    pub fn new() -> Self {
        Self {
            fishes: Vec::new(),
            past_time: 0.0,
            density: 1.0,
        }
    }
}
impl GenerativeView for SchoolOfFish {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        // density に応じて魚の数を増やす
        if self.past_time + TIME_INTERVAL / self.density < crnt_time
            && self.fishes.len() < MAX_FISHES
        {
            // density が多いほど、このif文に入る回数が増える
            let ratio = (5.0 + self.density) / 5.0;
            let quantity = (ratio * ratio) as usize;
            // density が多いほど、for文の回数が増える
            for _ in 0..quantity {
                self.fishes.push(Fish::new(crnt_time, &rs));
            }
            self.past_time = crnt_time;
        }
        self.fishes.retain(|f| f.check_keep(crnt_time));

        // Remove fish that have moved out of the screen
        self.density *= 0.98;
        if self.density < 1.0 {
            self.density = 1.0;
        }
    }
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {
        self.density += 1.0;
        if self.density > 10.0 {
            self.density = 10.0;
        }
    }
    fn set_mode(&mut self, _mode: GraphMode) {}
    fn disp(
        &self,
        draw: Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    ) {
        for fish in &self.fishes {
            fish.disp(&draw, crnt_time, &rs);
        }
    }
}
//*******************************************************************
//      One Fish Graphic
//*******************************************************************
pub struct Fish {
    start_time: f32,   //  魚が左側に初めて表示された時間
    max_xsize: f32,    //  左右の幅
    depth: f32,        //  魚の上下の泳ぎの深さ
    speed: f32,        //  魚の泳ぎの速さ（遠さで自動調整）
    length: f32,       //  魚の長さ
    height: f32,       //  魚が表示される位置(y座標)
    thickness: f32,    //  魚の太さ
    swimming: f32,     //  魚の泳ぎの上下の動きの速さ
    strong_color: f32, //  魚の色の濃さ（遠さで自動調整）
}
impl Fish {
    pub fn new(start_time: f32, rs: &Resize) -> Self {
        let far = (random_f32() * 0.5) + 0.5; // 0.5 - 1.0
        let length = (random_f32() + 0.5) * 2.0;
        Self {
            start_time,
            max_xsize: rs.get_full_size_x() + 100.0 * length,
            depth: ((random_f32() * 0.2) + 1.0) * 10.0,
            speed: 250.0 * far,
            length,
            height: (random_f32() - 0.5) * rs.get_full_size_y() * 0.7,
            thickness: 10.0,
            swimming: random_f32() + 0.5,
            strong_color: far,
        }
    }
    pub fn check_keep(&self, crnt_time: f32) -> bool {
        let ofs_time = crnt_time - self.start_time;
        let locate = ofs_time * self.speed;
        self.max_xsize >= locate
    }
    pub fn disp(
        &self,
        draw: &Draw,
        crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: &Resize,
    ) {
        let ofs_time = crnt_time - self.start_time;
        let locate = ofs_time * self.speed - rs.get_full_size_x() / 2.0;
        // 定数化した描画ステップ数を利用
        for i in 0..FISH_DRAW_STEPS {
            let x = locate - (i as f32) * self.length;
            draw.ellipse()
                .x_y(
                    x,
                    (self.swimming * x / 50.0).sin() * self.depth + self.height,
                )
                .radius(
                    ((i as f32) * std::f32::consts::PI / FISH_DRAW_STEPS as f32).sin()
                        * self.thickness,
                )
                .gray(
                    ((i as f32) * std::f32::consts::PI / FISH_DRAW_STEPS as f32).sin()
                        * self.strong_color,
                );
        }
    }
}
