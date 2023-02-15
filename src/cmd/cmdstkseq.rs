//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;

// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
pub struct SeqDataStock {
    _pdt: [Option<Box<PhraseDataStock>>; lpnlib::MAX_USER_PART],
    _cdt: [Option<Box<CompositionDataStock>>; lpnlib::MAX_USER_PART],
}
impl SeqDataStock {
    pub fn new() -> Self {
        Self {
            _pdt: Default::default(),
            _cdt: Default::default(),
        }
    }
    pub fn set_raw_phrase(&self, _part: usize, _input_text: String) -> bool {
        false
    }
    pub fn _set_raw_composition(&self, _part: usize, _input_text: String) -> bool {
        false
    }
    pub fn _set_recombined(&self) {

    }
}
pub struct PhraseDataStock {

}
impl PhraseDataStock {
    pub fn new() -> Self {
        Self {

        }
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