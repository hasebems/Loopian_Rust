//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;
use super::txt2seq_phr::*;
use super::txt2seq_cmps::*;

// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
pub struct SeqDataStock {
    pdt: [PhraseDataStock; lpnlib::MAX_USER_PART],
    cdt: [CompositionDataStock; lpnlib::MAX_USER_PART],
    input_mode: lpnlib::InputMode,
    tick_for_onemsr: i32,
    tick_for_onebeat: i32,
    bpm: u16,
}
impl SeqDataStock {
    pub fn new() -> Self {
        Self {
            pdt: Default::default(),
            cdt: Default::default(),
            input_mode: lpnlib::InputMode::Closer,
            tick_for_onemsr: lpnlib::DEFAULT_TICK_FOR_ONE_MEASURE,
            tick_for_onebeat: lpnlib::DEFAULT_TICK_FOR_QUARTER,
            bpm: lpnlib::DEFAULT_BPM,
        }
    }
    pub fn get_pdstk(&self, part: usize) -> &PhraseDataStock {&self.pdt[part]}
    pub fn set_raw_phrase(&mut self, part: usize, input_text: String) -> bool {
        if part < lpnlib::MAX_USER_PART {
            if self.pdt[part].set_raw(input_text) {
                self.pdt[part].set_recombined(self.input_mode, self.bpm, self.tick_for_onemsr);
                return true
            }
        }
        false
    }
    pub fn set_raw_composition(&mut self, part: usize, input_text: String) -> bool {
        if part < lpnlib::MAX_USER_PART {
            if self.cdt[part].set_raw(input_text) {
                self.cdt[part].set_recombined(self.tick_for_onemsr, self.tick_for_onebeat);
                return true
            }
        }
        false
    }
    pub fn change_beat(&mut self, beat_count: u16, base_note: u16) {
        println!("beat: {}/{}",beat_count, base_note);
        self.tick_for_onemsr = lpnlib::DEFAULT_TICK_FOR_ONE_MEASURE*(beat_count as i32)/(base_note as i32);
        self.tick_for_onebeat = lpnlib::DEFAULT_TICK_FOR_QUARTER*4/(base_note as i32);
        self.recombine_phr_all();
    }
    pub fn change_bpm(&mut self, bpm: u16) {
        self.bpm = bpm;
        self.recombine_phr_all();
    }
    fn recombine_phr_all(&mut self) {
        for pd in self.pdt.iter_mut() {
            pd.set_recombined(self.input_mode, self.bpm, self.tick_for_onemsr);
        }
    }
}
pub struct PhraseDataStock {
    oct_setting: i32,
    raw: String,
    cmpl_nt: Vec<String>,
    cmpl_ex: Vec<String>,
    rcmb: Vec<Vec<u16>>,
    whole_tick: i32,
}
impl Default for PhraseDataStock {
    fn default() -> Self {
        Self {
            oct_setting: 0,
            raw: "".to_string(),
            cmpl_nt: vec!["".to_string()],
            cmpl_ex: vec!["".to_string()],
            rcmb: Vec::new(),
            whole_tick: 0,
        }
    }
}
impl PhraseDataStock {
    pub fn get_final(&self) -> Vec<u16> {
        let mut ret_rcmb: Vec<u16> = vec![self.whole_tick as u16];
        for ev in self.rcmb.iter() {
            ret_rcmb.append(&mut ev.clone());
        }
        ret_rcmb
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        let cmpl = TextParse::complement_phrase(input_text);
        if cmpl.len() <= 1 {
            println!("Phrase input failed!");
            return false
        }
        else {
            self.cmpl_nt = cmpl[0].clone();
            self.cmpl_ex = cmpl[1].clone();
        }
        println!("complement_phrase: {:?} exp: {:?}",cmpl[0],cmpl[1]);
        true
    }
    pub fn set_recombined(&mut self, input_mode: lpnlib::InputMode, bpm: u16, tick_for_onemsr: i32) {
        if self.cmpl_nt == [""] {return}

        // 3.recombined data
        let (whole_tick, rcmb) = TextParse::recombine_to_internal_format(
            &self.cmpl_nt, &self.cmpl_ex, input_mode,
            self.oct_setting, tick_for_onemsr);
        self.rcmb = rcmb;
        self.whole_tick = whole_tick;

        // 4.analysed data

        // 5.humanized data
        self.rcmb = TextParse::beat_filter(&mut self.rcmb, bpm, tick_for_onemsr);
        println!("final_phrase: {:?} whole_tick: {:?}", self.rcmb, self.whole_tick);
    }
}
pub struct CompositionDataStock {
    raw: String,
    cmpl_cd: Vec<String>,
    cmpl_ex: Vec<String>,
    rcmb: Vec<Vec<u16>>,
    whole_tick: i32,
}
impl Default for CompositionDataStock {
    fn default() -> Self {
        TextParseCmps::something_todo();
        Self {
            raw: "".to_string(),
            cmpl_cd: vec!["".to_string()],
            cmpl_ex: vec!["".to_string()],
            rcmb: Vec::new(),
            whole_tick: 0,
        }
    }    
}
impl CompositionDataStock {
    pub fn _get_final(&self) -> Vec<u16> {
        let ret_rcmb: Vec<u16> = vec![self.whole_tick as u16];
        ret_rcmb
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        let cmpl = TextParseCmps::complement_composition(input_text);
        self.cmpl_cd = cmpl[0].clone();
        self.cmpl_ex = cmpl[1].clone();
        if self.cmpl_cd == [""] {
            println!("Phrase input failed!");
            return false;
        }
        println!("complement_composition: {:?} exp: {:?}",cmpl[0],cmpl[1]);
        true
    }
    pub fn set_recombined(&mut self, tick_for_onemsr: i32, tick_for_onebeat: i32) {
        if self.cmpl_cd == [""] {return}

        // 3.recombined data
        let (whole_tick, rcmb) = 
            TextParseCmps::recombine_to_chord_loop(&self.cmpl_cd, tick_for_onemsr, tick_for_onebeat);
        self.rcmb = rcmb;
        self.whole_tick = whole_tick;
        println!("final_composition: {:?} whole_tick: {:?}", self.rcmb, self.whole_tick);
    }
}