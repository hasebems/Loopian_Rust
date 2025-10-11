//  Created by Hasebe Masahiko on 2025/10/09.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use crate::lpnlib::*;

//*******************************************************************
//          Phrase Data Stock Struct
//*******************************************************************
#[derive(Debug)]
pub struct PedalDataStock {
    _pdl: Vec<PhrEvt>,
}
impl PedalDataStock {
    pub fn new() -> Self {
        Self {
            _pdl: Vec::new(),
        }
    }
    pub fn set_raw(&mut self, _input_text: String, _cluster_word: &str) -> bool {
        true
    }
    pub fn set_recombined(
        &mut self,
        _input_mode: InputMode,
        _bpm: i16,
        _tick_for_onemsr: i32,
        _tick_for_beat: i32,
        _resend: bool,
    ) {}
    pub fn get_final(&self, _part: i16) -> ElpsMsg {
        println!("PedalDataStock::get_final is called");
        ElpsMsg::Ctrl(0)    // Dummy
    }
}
