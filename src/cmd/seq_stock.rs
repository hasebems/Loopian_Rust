//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use crate::elapse::ug_content::*;
use super::txt2seq_phr::*;
use super::txt2seq_cmps::*;
use super::txt2seq_ana::*;

//*******************************************************************
//          Seq Data Stock Struct
//*******************************************************************
// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
pub struct SeqDataStock {
    pdt: Vec<Vec<PhraseDataStock>>,
    cdt: [CompositionDataStock; MAX_USER_PART],
    input_mode: InputMode,
    tick_for_onemsr: i32,
    tick_for_onebeat: i32,
    bpm: i16,
}
impl SeqDataStock {
    pub fn new() -> Self {
        let mut pd = Vec::new();
        for i in 0..MAX_USER_PART {
            let mut vari = Vec::new();
            let base_note = Self::default_base_note(i);
            for _ in 0..MAX_VARI_PHRASE+1 {
                vari.push(PhraseDataStock::new(base_note));
            }
            pd.push(vari);
        }
        Self {
            pdt: pd,
            cdt: Default::default(),
            input_mode: InputMode::Closer,
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            tick_for_onebeat: DEFAULT_TICK_FOR_QUARTER,
            bpm: DEFAULT_BPM,
        }
    }
    pub fn get_pdstk(&self, part: usize, vari: usize) -> &PhraseDataStock {&self.pdt[part][vari]}
    pub fn get_cdstk(&self, part: usize) -> &CompositionDataStock {&self.cdt[part]}
    pub fn set_raw_phrase(&mut self, part: usize, vari: usize, input_text: String) -> bool {
        if part < MAX_USER_PART {
            if self.pdt[part][vari].set_raw(input_text) {
                self.pdt[part][vari].set_recombined(self.input_mode, self.bpm, self.tick_for_onemsr);
                return true
            }
        }
        false
    }
    pub fn set_raw_composition(&mut self, part: usize, input_text: String) -> bool {
        if part < MAX_USER_PART {
            if self.cdt[part].set_raw(input_text) {
                self.cdt[part].set_recombined(self.tick_for_onemsr, self.tick_for_onebeat);
                return true
            }
        }
        false
    }
    pub fn change_beat(&mut self, numerator: i16, denomirator: i16) {
        //println!("beat: {}/{}",numerator, denomirator);
        self.tick_for_onemsr = DEFAULT_TICK_FOR_ONE_MEASURE*(numerator as i32)/(denomirator as i32);
        self.tick_for_onebeat = DEFAULT_TICK_FOR_QUARTER*4/(denomirator as i32);
        self.recombine_all();
    }
    pub fn change_bpm(&mut self, bpm: i16) {
        self.bpm = bpm;
        self.recombine_phr_all();
    }
    pub fn change_oct(&mut self, oct: i32, relative: bool, part: usize) -> bool {
        let mut update = false;
        let new_bd: i32;
        if oct == 0 {   // Reset Octave
            new_bd = Self::default_base_note(part);
            let dbn_now = self.pdt[part][0].base_note;
            if new_bd != dbn_now {
                update = true;
            }
        }
        else {
            let old = self.pdt[part][0].base_note/12 - 1;
            let mut new = old;
            if relative {new += oct;}
            else        {new = oct;}
            if new >= 8 {new = 7;}
            else if new < 1 {new = 1;}
            update = old != new;
            new_bd = (new+1)*12;
        }
        if update {
            for epd in self.pdt[part].iter_mut() {
                epd.base_note = new_bd;
                epd.set_recombined(self.input_mode, self.bpm, self.tick_for_onemsr);
            }
        }
        update
    }
    pub fn change_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }
    fn recombine_phr_all(&mut self) {
        for pd in self.pdt.iter_mut() {
            for epd in pd.iter_mut() {
                epd.set_recombined(self.input_mode, self.bpm, self.tick_for_onemsr);
            }
        }
    }
    fn recombine_all(&mut self) {
        for (i, pd) in self.pdt.iter_mut().enumerate() {
            for epd in pd.iter_mut() {
                epd.set_recombined(self.input_mode, self.bpm, self.tick_for_onemsr);
            }
            self.cdt[i].set_recombined(self.tick_for_onemsr, self.tick_for_onebeat);
        }
    }
    fn default_base_note(part_num: usize) -> i32 {
        (DEFAULT_NOTE_NUMBER as i32) + 12*((part_num as i32) - 2)
    }
}

//*******************************************************************
//          Phrase Data Stock Struct
//*******************************************************************
pub struct PhraseDataStock {
    base_note: i32,
    raw: String,
    cmpl_nt: Vec<String>,
    cmpl_ex: Vec<String>,
    rcmb: UgContent,
    ana: UgContent,
    whole_tick: i32,
}
impl PhraseDataStock {
    fn new(base_note: i32) -> Self {
        Self {
            base_note,
            raw: "".to_string(),
            cmpl_nt: vec!["".to_string()],
            cmpl_ex: vec!["".to_string()],
            rcmb: UgContent::new(),
            ana: UgContent::new(),
            whole_tick: 0,
        }
    }
    pub fn get_final(&self) -> (Vec<i16>, Vec<i16>) {
        let mut ret_rcmb: Vec<i16> = vec![self.whole_tick as i16];
        for ev in self.rcmb.naked().iter() {
            ret_rcmb.append(&mut ev.clone());
        }
        let mut ret_ana: Vec<i16> = vec![self.whole_tick as i16];
        for ev in self.ana.naked().iter() {
            if ev.len() != TYPE_BEAT_SIZE {
                ret_ana.append(&mut vec![ev[TYPE], ev[EXPR], 0,0,0,0]);
            }
            else {
                ret_ana.append(&mut ev.clone());
            }
        }
        (ret_rcmb, ret_ana)
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        let cmpl = complement_phrase(input_text);
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
    pub fn set_recombined(&mut self, input_mode: InputMode, bpm: i16, tick_for_onemsr: i32) {
        if self.cmpl_nt == [""] {
            //  clear
            self.rcmb = UgContent::new();
            self.ana = UgContent::new();
            println!("no_phrase...");
            return
        }

        // 3.recombined data
        let (whole_tick, rcmb) = recombine_to_internal_format(
            &self.cmpl_nt, &self.cmpl_ex, input_mode,
            self.base_note, tick_for_onemsr);
        self.rcmb = rcmb;
        self.whole_tick = whole_tick;

        // 4.analysed data
        self.ana = analyse_data(&self.rcmb, &self.cmpl_ex);

        // 5.humanized data
        let human1 = beat_filter(&mut self.rcmb, bpm, tick_for_onemsr);
        self.rcmb = crispy_tick(&human1, &self.cmpl_ex);
        println!("final_phrase: {:?} whole_tick: {:?}", self.rcmb.naked(), self.whole_tick);
        println!("analyse: {:?}", self.ana.naked());
    }
}

//*******************************************************************
//          Composition Data Stock Struct
//*******************************************************************
pub struct CompositionDataStock {
    raw: String,
    cmpl_cd: Vec<String>,
    cmpl_ex: Vec<String>,
    rcmb: UgContent,
    whole_tick: i32,
}
impl Default for CompositionDataStock {
    fn default() -> Self {
        Self {
            raw: "".to_string(),
            cmpl_cd: vec!["".to_string()],
            cmpl_ex: vec!["".to_string()],
            rcmb: UgContent::new(),
            whole_tick: 0,
        }
    }
}
impl CompositionDataStock {
    pub fn get_final(&self) -> Vec<i16> {
        let mut ret_rcmb: Vec<i16> = vec![self.whole_tick as i16];
        for ev in self.rcmb.naked().iter() {
            ret_rcmb.append(&mut ev.clone());
        }
        ret_rcmb
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        if let Some(cmpl) = complement_composition(input_text){
            self.cmpl_cd = cmpl[0].clone();
            self.cmpl_ex = cmpl[1].clone();
            println!("complement_composition: {:?} exp: {:?}",cmpl[0],cmpl[1]);
            true
        }
        else {
            println!("Composition input failed!");
            false
        }
    }
    pub fn set_recombined(&mut self, tick_for_onemsr: i32, tick_for_onebeat: i32) {
        if self.cmpl_cd == [""] {
            // clear
            self.rcmb = UgContent::new();
            println!("no_composition...");
            return
        }

        // 3.recombined data
        let (whole_tick, rcmb) = 
            recombine_to_chord_loop(&self.cmpl_cd, tick_for_onemsr, tick_for_onebeat);
        self.rcmb = rcmb;
        self.whole_tick = whole_tick;
        println!("final_composition: {:?} whole_tick: {:?}", self.rcmb.naked(), self.whole_tick);
    }
}