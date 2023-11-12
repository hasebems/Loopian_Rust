//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use eframe::egui::*;

pub trait NoteObj {
    fn disp(&self, crnt_time: i32, ui: &mut Ui, fsz: Pos2) -> bool;
}