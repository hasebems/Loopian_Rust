//  Created by Hasebe Masahiko on 2025/01/08.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
//  originally based on p5.js code by zhaoyc
//      https://openprocessing.org/sketch/1627198
//
use super::draw_graph::Resize;
use super::generative_view::*;
use nannou::prelude::*;

//*******************************************************************
//      Screen Graphic
//*******************************************************************
struct RealThread {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    alpha: f32,
    width: f32,
    count_down: usize,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct ThreadPoint {
    start_phase: f32,
    distance: f32,
    speed: f32,
    radius: f32,
    x: f32,
    y: f32,
}
impl ThreadPoint {
    const NORMAL_RADIUS: f32 = 150.0; // 何も鳴っていない時の半径

    pub fn new(start_phase: f32, distance: f32, speed: f32) -> Self {
        Self {
            start_phase,
            distance,
            speed,
            radius: Self::NORMAL_RADIUS,
            x: 0.0,
            y: 0.0,
        }
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
    pub fn move_point(&mut self, frame_count: f32) {
        self.x = self.distance * (self.start_phase + frame_count).cos()
            + (self.radius - self.distance)
                * (-self.speed * (self.start_phase + frame_count)).cos();
        self.y = self.distance * (self.start_phase + frame_count).sin()
            + (self.radius - self.distance)
                * (-self.speed * (self.start_phase + frame_count)).sin();
    }
    pub fn _distance(&self, another_point: &ThreadPoint) -> f32 {
        ((self.x - another_point.x).powi(2) + (self.y - another_point.y).powi(2)).sqrt()
    }
}
//*******************************************************************
pub struct CircleThread {
    rs: Resize,
    points: [ThreadPoint; Self::MAX_POINTS],
    threads: Vec<RealThread>,
    bigger_radius_target: f32,
    bigger_radius: f32,
    mode: GraphMode,
}
impl CircleThread {
    const MAX_THREAD_LENGTH: f32 = 120.0; // 糸を引く最大距離
    const MAX_POINTS: usize = 50; // ポイント数
    const SPEED: f32 = 0.3; // ポイントの移動速度
    const RADIUS_WIDTH: f32 = 50.0; // ポイントの半径の幅
    const SPEED_RANGE: f32 = 10.0; // ポイントの速度の幅
    const FADE_OUT_COUNT: usize = 10; // 線がフェードアウトするまでのフレーム数

    pub fn new() -> Self {
        let points: [ThreadPoint; Self::MAX_POINTS] = std::array::from_fn(|_| {
            ThreadPoint::new(
                random_range(0.0, 1000.0),
                random_range(0.0, Self::RADIUS_WIDTH),
                random_range(1.0, Self::SPEED_RANGE),
            )
        });
        Self {
            rs: Resize::default(),
            points,
            threads: Vec::new(),
            bigger_radius_target: 0.0,
            bigger_radius: 0.0,
            mode: GraphMode::Dark,
        }
    }
    fn fade_out_threads(&mut self) {
        // 既存の糸をフェードアウトさせる
        self.threads.retain_mut(|l| {
            if l.count_down > 0 {
                l.count_down -= 1;
                l.count_down > 0
            } else {
                false
            }
        });
    }
    fn gen_new_threads(&mut self) {
        // 新しい糸を生成する（距離の二乗で早期フィルタ・必要時のみsqrt）
        let threshold2 = Self::MAX_THREAD_LENGTH * Self::MAX_THREAD_LENGTH;
        // おおよその最大生成数を見積もって再確保を減らす
        let n = Self::MAX_POINTS;
        let potential_max = n * (n - 1) / 2; // 完全グラフの辺数
        if self.threads.capacity() < self.threads.len() + potential_max / 8 {
            self.threads.reserve(potential_max / 8);
        }

        for i in 0..Self::MAX_POINTS {
            let a = &self.points[i];
            for b in &self.points[i + 1..Self::MAX_POINTS] {
                let dx = a.x - b.x;
                let dy = a.y - b.y;
                let dist2 = dx * dx + dy * dy;
                if dist2 < threshold2 {
                    // 閾値を満たした時だけsqrtを計算
                    let dist = dist2.sqrt();
                    let alpha = map_range(dist, 0.0, Self::MAX_THREAD_LENGTH, 1.0, 0.0);
                    let width = map_range(dist, 0.0, Self::MAX_THREAD_LENGTH, 3.0, 0.1);
                    self.threads.push(RealThread {
                        x0: a.x,
                        y0: a.y,
                        x1: b.x,
                        y1: b.y,
                        alpha,
                        width,
                        count_down: Self::FADE_OUT_COUNT,
                    });
                }
            }
        }
    }
    fn dynamic_radius(&mut self) {
        // 徐々に目標値に近づける
        if self.bigger_radius_target > self.bigger_radius {
            self.bigger_radius += (self.bigger_radius_target - self.bigger_radius) / 2.0;
            if self.bigger_radius_target < self.bigger_radius + 0.01 {
                self.bigger_radius_target = 0.0;
            }
        } else {
            self.bigger_radius *= 0.95;
            if self.bigger_radius_target > self.bigger_radius {
                self.bigger_radius_target = self.bigger_radius;
            }
        }
    }
}
impl GenerativeView for CircleThread {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        self.rs = rs;

        // ポイントを移動
        self.points.iter_mut().for_each(|p| {
            p.set_radius(ThreadPoint::NORMAL_RADIUS * (1.0 + self.bigger_radius));
            p.move_point(crnt_time * Self::SPEED)
        });

        // 既存の線をフェードアウト
        self.fade_out_threads();

        // 新しい線を生成
        self.gen_new_threads();

        // 動的に半径を変化させる処理
        self.dynamic_radius();
    }
    /// Beat Event
    fn on_beat(&mut self, _bt: i32, _tm: f32, _dt: f32) {}
    /// Note Event
    fn note_on(&mut self, _nt: i32, vel: i32, _pt: i32, _tm: f32) {
        self.bigger_radius_target = vel as f32 / 127.0; // 0.0 ... 1.0
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
        let bgcolor = match self.mode {
            GraphMode::Dark => 1.0,
            GraphMode::Light => 0.0,
        };

        // 線を描画
        for thread in &self.threads {
            let alpha = thread.alpha / 1.1_f32.powf(thread.count_down as f32);
            let color = rgba(bgcolor, bgcolor, bgcolor, alpha);
            draw.line()
                .start(pt2(thread.x0, thread.y0))
                .end(pt2(thread.x1, thread.y1))
                .weight(thread.width)
                .color(color);
        }
    }
}
