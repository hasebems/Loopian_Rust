//  Created by Hasebe Masahiko on 2025/01/08.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use nannou::noise::{NoiseFn, Perlin};
use nannou::prelude::*;
use super::draw_graph::Resize;
use super::generative_view::{GenerativeView, GraphMode};

//*******************************************************************
//      Screen Graphic
//*******************************************************************
pub struct WaveStick {
    pub rs: Resize,
    perlin: Perlin,
    t: f32,   // p5 のフレームカウンタ相当
    amp: f32, // 振幅倍率
    prev_time: f32,
    amp_target: f32,
}
impl WaveStick {
    const DEFAULT_AMP: f32 = 0.1;
    const DECAY_RATE: f32 = 2.5;
    const RESPONSE_SPEED: f32 = 1.5;
    const BRIGHTNESS: f32 = 50.0;   // 0..255

    pub fn new() -> Self {
        Self {
            rs: Resize::default(),
            perlin: Perlin::new(),
            t: 0.0,
            amp: Self::DEFAULT_AMP,
            prev_time: 0.0,
            amp_target: Self::DEFAULT_AMP,
        }
    }
    // 線形補間の小ユーティリティ
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}
impl GenerativeView for WaveStick {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        self.rs = rs;
        self.t += 1.0;
        let diff_time = crnt_time - self.prev_time;
        self.prev_time = crnt_time;
        // 強さを徐々に目標値へ近づけ、目標値は徐々に DEFAULT_AMP へ近づける
        self.amp += (self.amp_target - self.amp) * diff_time * Self::RESPONSE_SPEED;
        self.amp_target += (Self::DEFAULT_AMP - self.amp_target) * diff_time * Self::DECAY_RATE;
    }
    /// Beat Event
    fn on_beat(&mut self, _bt: i32, _tm: f32, _dt: f32) {}
    /// Note Event
    fn note_on(&mut self, _nt: i32, vel: i32, _pt: i32, _tm: f32) {
        self.amp_target += 0.005 * vel as f32;
    }
    fn set_mode(&mut self, _mode: GraphMode) {}
    fn disp(
        &self,
        draw: Draw,
        _crnt_time: f32, //  const FPS(50msec) のカウンター
        rs: Resize,
    ) {
        //let rect = app.window_rect();
        let w = rs.get_full_size_x(); //rect.w();
        let h = rs.get_full_size_y(); //rect.h();
        let sw = w / 100.0; // p5: width / 100 と同じ比率

        // 行の間隔（p5: height / 15 ごと）
        let row_step = h / 15.0;
        // y=0..height → nannou座標では -h/2..+h/2
        // なので行インデックスで回す
        for (n, row_y) in (0..=15).map(|i| -h / 2.0 + i as f32 * row_step).enumerate() {
            // p5: translate((n % 2) * sw, y)
            let x_offset = if n % 2 == 0 { 0.0 } else { sw };

            // x は -200..width+200（px）相当。nannou座標へ合わせるため左右に200px 相当を足す。
            // ここでは画面幅のスケールに合わせて 200px ≒ w/ (width_in_px) が不明なので、
            // 目視で同等感のある余白として sw*6 を採用（必要ならここを調整）。
            let margin = sw * 6.0;
            let start_x = -w / 2.0 - margin + x_offset;
            let end_x = w / 2.0 + margin;
            let step_x = sw * 2.0;

            let mut x = start_x;
            while x < end_x {
                // p5: n1 = noise(y, x/width, t/400)
                // noise crate は [-1,1] 範囲を返すので、p5 の (noise - 0.5) * 2 と同等
                let x_norm = (x + w / 2.0) / w; // 0..1
                let y_norm = (row_y + h / 2.0) / h; // 0..1
                let t1 = self.t / 400.0;

                let n1 = self.perlin.get([y_norm as f64, x_norm as f64, t1 as f64]) as f32;
                let n2 = self
                    .perlin
                    .get([(n1 * 2.0) as f64, x_norm as f64, t1 as f64])
                    as f32;

                // p5 と同様の位相合成
                let s = ((x_norm + y_norm) * PI + self.t / 50.0).sin();
                let c = ((x_norm + y_norm) * PI + self.t / 100.0).cos();

                let n1 = n1 * s;
                let n2 = n2 * c;

                // p5: y1 = n1 * height/2, y2 = n2 * height/2
                let y1 = row_y + n1 * (h / 2.0) * self.amp;
                let y2 = row_y + n2 * (h / 2.0) * self.amp;

                // ここから「縦方向グラデーションのストローク」を段階線で再現
                // p5 のグラデーション: 先端が alpha=100、末端が alpha=0
                // 透明度は 0.0..1.0、100/255 ≒ 0.392
                let a_start = Self::BRIGHTNESS / 255.0;
                let a_end = 0.0;
                let segments = 20; // 分割数（見た目と速度のバランスで調整可）

                // 方向に依らず上から下へ補間
                let (ya, yb) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };

                for i in 0..segments {
                    let t0 = i as f32 / segments as f32;
                    let t1 = (i + 1) as f32 / segments as f32;
                    let yy0 = Self::lerp(ya, yb, t0);
                    let yy1 = Self::lerp(ya, yb, t1);
                    let alpha = Self::lerp(a_start, a_end, t0);

                    draw.line()
                        .start(pt2(x, yy0))
                        .end(pt2(x, yy1))
                        .weight(sw)
                        .rgba(1.0, 1.0, 1.0, alpha);
                }
                x += step_x;
            }
        }
    }
}
