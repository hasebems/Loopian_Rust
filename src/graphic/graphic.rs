//  Created by Hasebe Masahiko on 2023/10/31.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use eframe::{egui,egui::*};
use crate::{WINDOW_X, WINDOW_Y};
use crate::lpnlib::*;
use crate::cmd::cmdparse::LoopianCmd;
use super::waterripple::WaterRipple;

#[derive(Default)]
pub struct Graphic {
    full_size: Pos2,
    nobj: Vec<WaterRipple>,
}

impl Graphic {
    pub fn new() -> Graphic {
        Self {
            full_size: Pos2 {x:WINDOW_X, y: WINDOW_Y},
            nobj: Vec::new(),
        }
    }
    pub fn update(&mut self, ui: &mut Ui, cmd: &mut LoopianCmd, frame: &mut eframe::Frame, counter: i32) {
        self.full_size.x = frame.info().window_info.size.x;
        self.full_size.y = frame.info().window_info.size.y;

        self.update_title(ui);

        if let Some(kmsg) = cmd.get_ev_from_gev() {
            cmd.remove_from_gev(0);
            let nt_vel = split_by('/', kmsg);
            let nt: i32 = nt_vel[0].parse().unwrap();
            let vel: i32 = nt_vel[1].parse().unwrap();
            self.nobj.push(WaterRipple::new(nt as f32, vel as f32, counter));
        }

        let nlen = self.nobj.len();
        let mut rls = vec![true; nlen];
        for (i, obj) in self.nobj.iter_mut().enumerate() {
            if obj.disp(counter, ui) == false {
                rls[i] = false;
            }
        }
        for i in 0..nlen {  // 一度に一つ消去
            if !rls[i] {self.nobj.remove(i); break;}
        }

    }
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
}