use nannou::prelude::*;
use nannou::text::Font;

use loopian_graphic_api::Resize;
use loopian_graphic_api::generative_view::{BeatObj, GenerativeView, GraphMode, NoteObj};

pub struct Spring {
    pub rs: Resize,
    pub font: Font,
    pub mode: GraphMode,
    pub osc: Vec<OscUnit>,
    pub notes: Vec<SpringNote>,
    pub beats: Vec<SpringBeat>,
    pub last_time: f32,
    pub global_phase: f32,
    pub drive_amp: f32,
}

impl Spring {
    const NUM_UNITS: usize = 28;
    const BASE_AMP: f32 = 0.11;
    const DRIVE_DECAY: f32 = 1.6;
    const PHASE_SPEED: f32 = 2.4;
    const PHASE_STEP: f32 = 0.42;
    const SPRING_COILS: usize = 14;

    pub fn new(font: Font) -> Self {
        let osc = (0..Self::NUM_UNITS)
            .map(|i| {
                let f = i as f32 / (Self::NUM_UNITS.saturating_sub(1)) as f32;
                OscUnit::new(f, f * TAU * 0.15)
            })
            .collect();

        Self {
            rs: Resize::default(),
            font,
            mode: GraphMode::Light,
            osc,
            notes: Vec::new(),
            beats: Vec::new(),
            last_time: 0.0,
            global_phase: 0.0,
            drive_amp: Self::BASE_AMP,
        }
    }

    pub fn clear(&mut self) {
        self.osc.iter_mut().for_each(OscUnit::reset_energy);
        self.notes.clear();
        self.beats.clear();
        self.drive_amp = Self::BASE_AMP;
    }

    fn spawn_note(&mut self, nt: i32, vel: i32, pt: i32, tm: f32) {
        let _ = (pt, tm);
        let idx = nt.rem_euclid(Self::NUM_UNITS as i32) as usize;
        let kick = map_range(vel as f32, 0.0, 127.0, 0.04, 0.28);
        if let Some(osc) = self.osc.get_mut(idx) {
            osc.energy = (osc.energy + kick).clamp(0.0, 1.0);
        }
        self.drive_amp = (self.drive_amp + kick * 0.4).clamp(Self::BASE_AMP, 0.65);
    }

    fn spawn_beat(&mut self, bt: i32, ct: f32, dt: f32) {
        let _ = bt;
        let _ = ct;
        self.drive_amp = (self.drive_amp + 0.06).clamp(Self::BASE_AMP, 0.65);
        for osc in &mut self.osc {
            osc.energy = (osc.energy + dt * 0.05).clamp(0.0, 1.0);
        }
    }

    fn style(&self) -> (Rgba, Rgba, Rgba, Rgba) {
        match self.mode {
            GraphMode::Light => (
                rgba(0.96, 0.96, 0.96, 1.0),
                rgba(0.84, 0.84, 0.84, 0.72),
                rgba(0.78, 0.78, 0.78, 0.96),
                rgba(0.995, 0.995, 0.995, 0.9),
            ),
            GraphMode::Dark => (
                rgba(0.94, 0.94, 0.94, 1.0),
                rgba(0.87, 0.87, 0.87, 0.76),
                rgba(0.9, 0.9, 0.9, 0.96),
                rgba(1.0, 1.0, 1.0, 0.92),
            ),
        }
    }

    fn draw_spring(
        &self,
        draw: &Draw,
        anchor: Point2,
        mass_center: Point2,
        spring_color: Rgba,
        stroke: f32,
    ) {
        let coils = Self::SPRING_COILS.max(4);
        let width = (self.rs.get_full_size_x() / 300.0).clamp(2.0, 7.0);
        let mut points = Vec::with_capacity(coils * 2 + 3);
        points.push(anchor);

        for i in 1..=coils * 2 {
            let t = i as f32 / (coils * 2 + 1) as f32;
            let x = anchor.x + if i % 2 == 0 { width } else { -width };
            let y = anchor.y + (mass_center.y - anchor.y) * t;
            points.push(pt2(x, y));
        }

        points.push(pt2(anchor.x, mass_center.y));
        draw.polyline()
            .weight(stroke)
            .points(points)
            .color(spring_color);
    }

    fn draw_mass(
        &self,
        draw: &Draw,
        center: Point2,
        radius: f32,
        mass_color: Rgba,
        highlight_color: Rgba,
    ) {
        draw.ellipse()
            .xy(center + vec2(radius * 0.12, -radius * 0.1))
            .radius(radius * 1.05)
            .rgba(0.58, 0.58, 0.58, 0.28);

        draw.ellipse().xy(center).radius(radius).color(mass_color);

        draw.ellipse()
            .xy(center + vec2(-radius * 0.28, radius * 0.3))
            .radius(radius * 0.42)
            .color(highlight_color);

        draw.ellipse()
            .xy(center + vec2(-radius * 0.1, radius * 0.08))
            .radius(radius * 0.82)
            .rgba(0.9, 0.9, 0.9, 0.18);
    }
}

impl GenerativeView for Spring {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) {
        self.rs = rs.clone();
        let dt = if self.last_time <= 0.0 {
            0.02
        } else {
            (crnt_time - self.last_time).clamp(0.001, 0.08)
        };
        self.last_time = crnt_time;

        self.global_phase += dt * Self::PHASE_SPEED;
        self.drive_amp += (Self::BASE_AMP - self.drive_amp) * dt * Self::DRIVE_DECAY;

        for (i, osc) in self.osc.iter_mut().enumerate() {
            osc.phase = self.global_phase + i as f32 * Self::PHASE_STEP + osc.phase_jitter;
            osc.energy += (0.0 - osc.energy) * dt * 1.1;
        }
    }

    fn note_on(&mut self, nt: i32, vel: i32, pt: i32, tm: f32) {
        self.spawn_note(nt, vel, pt, tm);
    }

    fn on_beat(&mut self, bt: i32, ct: f32, dt: f32) {
        self.spawn_beat(bt, ct, dt);
    }

    fn set_mode(&mut self, mode: GraphMode) {
        self.mode = mode;
    }

    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        let _ = (&self.font, crnt_time, rs.clone());
        let (_top_bar_color, spring_color, mass_color, highlight_color) = self.style();

        let w = rs.get_full_size_x();
        let h = rs.get_full_size_y();
        let left = -w * 0.46;
        let right = w * 0.46;
        let top_y = h * 0.38;
        let base_y = -h * 0.05;
        let span = right - left;
        let step = if self.osc.len() > 1 {
            span / (self.osc.len() - 1) as f32
        } else {
            0.0
        };

        for (i, osc) in self.osc.iter().enumerate() {
            let x = left + step * i as f32;
            let amp = h * (self.drive_amp + osc.energy * 0.18);
            let y = base_y + amp * osc.phase.sin();
            let wobble = (osc.phase * 0.5 + i as f32 * 0.07).sin() * (w / 320.0);
            let mass_center = pt2(x + wobble, y);
            let anchor = pt2(x, top_y);

            self.draw_spring(
                &draw,
                anchor,
                mass_center,
                spring_color,
                (w / 700.0).clamp(1.0, 2.6),
            );

            self.draw_mass(
                &draw,
                mass_center,
                (w / 95.0).clamp(5.5, 12.5),
                mass_color,
                highlight_color,
            );
        }
    }
}

pub struct OscUnit {
    pub x_ratio: f32,
    pub phase: f32,
    pub phase_jitter: f32,
    pub energy: f32,
}

impl OscUnit {
    pub fn new(x_ratio: f32, phase_jitter: f32) -> Self {
        Self {
            x_ratio,
            phase: 0.0,
            phase_jitter,
            energy: 0.0,
        }
    }

    pub fn reset_energy(&mut self) {
        self.energy = 0.0;
    }
}

pub struct SpringNote {
    pub born_time: f32,
    pub note_num: i32,
    pub velocity: i32,
    pub part: i32,
}

impl SpringNote {
    pub fn new(note_num: i32, velocity: i32, part: i32, born_time: f32) -> Self {
        Self {
            born_time,
            note_num,
            velocity,
            part,
        }
    }
}

impl NoteObj for SpringNote {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool {
        let _ = rs;
        crnt_time - self.born_time < 1.2
    }

    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        let age = (crnt_time - self.born_time).max(0.0);
        let alpha = (1.0 - age / 1.2).clamp(0.0, 1.0) * 0.45;
        if alpha <= 0.0 {
            return;
        }
        let x = map_range(
            self.part as f32,
            0.0,
            3.0,
            -rs.get_full_size_x() * 0.44,
            rs.get_full_size_x() * 0.44,
        );
        draw.ellipse()
            .x_y(x, rs.get_full_size_y() * 0.42)
            .radius((self.velocity as f32 / 127.0) * 10.0 + 2.0)
            .rgba(0.96, 0.96, 0.96, alpha);
    }
}

pub struct SpringBeat {
    pub born_time: f32,
    pub beat: i32,
}

impl SpringBeat {
    pub fn new(beat: i32, born_time: f32) -> Self {
        Self { born_time, beat }
    }
}

impl BeatObj for SpringBeat {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool {
        let _ = rs;
        crnt_time - self.born_time < 0.6
    }

    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize) {
        let age = (crnt_time - self.born_time).max(0.0);
        let alpha = (1.0 - age / 0.6).clamp(0.0, 1.0) * 0.25;
        draw.rect()
            .w_h(rs.get_full_size_x() * 0.94, 2.0)
            .x_y(0.0, rs.get_full_size_y() * 0.38)
            .rgba(0.92, 0.92, 0.92, alpha);
    }
}
