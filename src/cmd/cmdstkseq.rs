//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;
use super::cmdtxt2seq::*;

// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
pub struct SeqDataStock {
    pdt: [Option<Box<PhraseDataStock>>; lpnlib::MAX_USER_PART],
    _cdt: [Option<Box<CompositionDataStock>>; lpnlib::MAX_USER_PART],
    input_mode: lpnlib::InputMode,
}
impl SeqDataStock {
    pub fn new() -> Self {
        Self {
            pdt: Default::default(),
            _cdt: Default::default(),
            input_mode: lpnlib::InputMode::Closer,
        }
    }
    pub fn set_raw_phrase(&mut self, part: usize, input_text: String) -> bool {
        if part < lpnlib::MAX_USER_PART {
            let mut pd = Box::new(PhraseDataStock::new(part, self.input_mode));
            if pd.set_raw(input_text) {
                self.pdt[part] = Some(pd);
                return true
            }
        }
        false
    }
    pub fn _set_raw_composition(&self, _part: usize, _input_text: String) -> bool {
        false
    }
    pub fn _set_recombined(&self) {

    }
}
pub struct PhraseDataStock {
    part_num: usize,
    input_mode: lpnlib::InputMode,
    note_value: u8,
    raw: String,
    cmpl: Vec<u16>,
}
impl PhraseDataStock {
    pub fn new(part_num: usize, input_mode: lpnlib::InputMode) -> Self {
        Self {
            part_num,
            input_mode,
            note_value: 0,
            raw: "".to_string(),
            cmpl: vec![0],
        }
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        let (cmpl, note_value) = TextParse::complement_phrase(input_text);
        if cmpl.len() <= 1 {
            return false
        }
        else {
            self.cmpl = cmpl;
            self.note_value = note_value;
        }

        // 3-5.recombined data        
        self.set_recombined();
        true
    }
    pub fn set_recombined(&mut self) {
        // 3.recombined data
        // 4.analysed data
        // 5.humanized data
            // Add Filters 
            //               
    }
}
pub struct CompositionDataStock {

}
impl CompositionDataStock {
    pub fn new() -> Self {
        Self {

        }
    }    
}