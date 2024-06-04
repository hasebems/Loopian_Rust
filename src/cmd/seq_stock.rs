//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt2seq_ana::*;
use super::txt2seq_cmps::*;
use super::txt2seq_phr::*;
use super::txt_common::*;
use crate::lpnlib::*;

//*******************************************************************
//          Seq Data Stock Struct
//*******************************************************************
// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
pub struct SeqDataStock {
    pdt: Vec<Vec<PhraseDataStock>>,
    cdt: [CompositionDataStock; MAX_KBD_PART],
    input_mode: InputMode,
    cluster_memory: String,
    raw_additional: String,
    tick_for_onemsr: i32,
    tick_for_onebeat: i32,
    bpm: i16,
}
impl SeqDataStock {
    pub fn new() -> Self {
        let mut pd = Vec::new();
        for i in 0..MAX_KBD_PART {
            let mut vari = Vec::new();
            let base_note = Self::default_base_note(i);
            for _ in 0..MAX_PHRASE {
                vari.push(PhraseDataStock::new(base_note));
            }
            pd.push(vari);
        }
        Self {
            pdt: pd,
            cdt: Default::default(),
            input_mode: InputMode::Closer,
            cluster_memory: "".to_string(),
            raw_additional: "".to_string(),
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            tick_for_onebeat: DEFAULT_TICK_FOR_QUARTER,
            bpm: DEFAULT_BPM,
        }
    }
    pub fn get_pdstk(&self, part: usize, vari: usize) -> &PhraseDataStock {
        &self.pdt[part][vari]
    }
    pub fn get_cdstk(&self, part: usize) -> &CompositionDataStock {
        &self.cdt[part]
    }
    pub fn set_cluster_memory(&mut self, word: String) {
        self.cluster_memory = word;
    }
    pub fn set_raw_phrase(
        &mut self,
        part: usize,
        vari: usize,
        mut input_text: String,
    ) -> Option<bool> {
        if let Some(rs) = self.check_if_additional_phrase(input_text.clone()) {
            input_text = rs;
        } else {
            return Some(true); // additional なら true
        }
        if part < MAX_KBD_PART {
            if self.pdt[part][vari].set_raw(input_text, &self.cluster_memory) {
                self.pdt[part][vari].set_recombined(
                    self.input_mode,
                    self.bpm,
                    self.tick_for_onemsr,
                );
                return Some(false);
            }
        }
        None
    }
    pub fn set_raw_composition(&mut self, part: usize, input_text: String) -> bool {
        if part < MAX_KBD_PART {
            if self.cdt[part].set_raw(input_text) {
                self.cdt[part].set_recombined(self.tick_for_onemsr, self.tick_for_onebeat);
                return true;
            }
        }
        false
    }
    pub fn change_beat(&mut self, numerator: i16, denomirator: i16) {
        //println!("beat: {}/{}",numerator, denomirator);
        self.tick_for_onemsr =
            DEFAULT_TICK_FOR_ONE_MEASURE * (numerator as i32) / (denomirator as i32);
        self.tick_for_onebeat = DEFAULT_TICK_FOR_QUARTER * 4 / (denomirator as i32);
        self.recombine_all();
    }
    pub fn change_bpm(&mut self, bpm: i16) {
        self.bpm = bpm;
        self.recombine_phr_all();
    }
    pub fn change_oct(&mut self, oct: i32, relative: bool, part: usize) -> bool {
        let mut update = false;
        let new_bd: i32;
        if oct == 0 {
            // Reset Octave
            new_bd = Self::default_base_note(part);
            let dbn_now = self.pdt[part][0].base_note;
            if new_bd != dbn_now {
                update = true;
            }
        } else {
            let old = self.pdt[part][0].base_note / 12 - 1;
            let mut new = old;
            if relative {
                new += oct;
            } else {
                new = oct;
            }
            if new >= 8 {
                new = 7;
            } else if new < 1 {
                new = 1;
            }
            update = old != new;
            new_bd = (new + 1) * 12;
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
    pub fn check_if_additional_phrase(&mut self, raw: String) -> Option<String> {
        let strlen = raw.len();
        if strlen >= 9 && &raw[(strlen - 9)..(strlen - 3)] == "].rpt(" && &raw[(strlen - 2)..] == ")+" {
            let rpt_cnt = extract_number_from_parentheses(&raw[(strlen - 9)..]);
            for i in 0..(rpt_cnt + 1) {
                if i == 0 && self.raw_additional.len() == 0 {
                    // 1st time
                    self.raw_additional = (&raw[0..(strlen - 9)]).to_string();
                } else {
                    // 2nd and more time
                    self.raw_additional += &raw[1..(strlen - 9)];
                }
            }
            None
        } else if strlen >= 2 && &raw[(strlen - 2)..] == "]+" {
            if self.raw_additional.len() == 0 {
                // 1st time
                self.raw_additional = (&raw[0..(strlen - 2)]).to_string();
            } else {
                // 2nd and more time
                self.raw_additional += &raw[1..(strlen - 2)];
            }
            // additional (..]+) なら None を返す
            None
        } else {
            let mut newraw = raw.clone();
            if self.raw_additional.len() != 0 {
                // last time
                newraw = self.raw_additional.clone() + &raw[1..];
                println!("Additional Phrase: {:?}", newraw);
                self.raw_additional = String::from("");
            }
            Some(newraw)
        }
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
        (DEFAULT_NOTE_NUMBER as i32) + 12 * ((part_num as i32) - 2)
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
    phr: Vec<PhrEvt>,
    ana: Vec<AnaEvt>,
    atrb: Vec<bool>,
    do_loop: bool,
    whole_tick: i32,
}
impl PhraseDataStock {
    fn new(base_note: i32) -> Self {
        Self {
            base_note,
            raw: "".to_string(),
            cmpl_nt: vec!["".to_string()],
            cmpl_ex: vec!["".to_string()],
            phr: Vec::new(),
            ana: Vec::new(),
            atrb: vec![false, false],
            do_loop: true,
            whole_tick: 0,
        }
    }
    pub fn get_final(&self, part: i16, vari: i16) -> (ElpsMsg, ElpsMsg) {
        let do_loop = if vari == 0 && self.do_loop {
            true
        } else {
            false
        };
        (
            ElpsMsg::Phr(
                part,
                vari,
                PhrData {
                    whole_tick: self.whole_tick as i16,
                    auftakt: if self.atrb[0] { 1 } else { 0 },
                    do_loop,
                    evts: self.phr.clone(),
                },
            ),
            ElpsMsg::Ana(
                part,
                vari,
                AnaData {
                    evts: self.ana.clone(),
                },
            ),
        )
    }
    pub fn set_raw(&mut self, input_text: String, cluster_word: &str) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        let cmpl = complement_phrase(input_text, cluster_word);
        self.cmpl_nt = cmpl.0.clone();
        self.cmpl_ex = cmpl.1.clone();
        self.atrb = cmpl.2.clone();
        println!(
            "complement_phrase: {:?} exp: {:?} atrb: {:?}",
            cmpl.0, cmpl.1, cmpl.2
        );
        true
    }
    pub fn set_recombined(&mut self, input_mode: InputMode, bpm: i16, tick_for_onemsr: i32) {
        if self.cmpl_nt == [""] {
            //  clear
            self.phr = Vec::new();
            self.ana = Vec::new();
            println!("no_phrase...");
            return;
        }

        // 3.recombined data
        let (whole_tick, do_loop, rcmb) = recombine_to_internal_format(
            &self.cmpl_nt,
            &self.cmpl_ex,
            input_mode,
            self.base_note,
            tick_for_onemsr,
        );
        self.phr = rcmb;
        self.do_loop = do_loop;
        self.whole_tick = whole_tick;

        // 4.analysed data
        self.ana = analyse_data(&self.phr, &self.cmpl_ex);

        // 5.humanized data
        self.phr = beat_filter(&mut self.phr, bpm, tick_for_onemsr);
        println!("final_phrase: {:?}", self.phr);
        println!(
            "whole_tick: {:?} do_loop: {:?}",
            self.whole_tick, self.do_loop
        );
        println!("analyse: {:?}", self.ana);
    }
}

//*******************************************************************
//          Composition Data Stock Struct
//*******************************************************************
pub struct CompositionDataStock {
    raw: String,
    cmpl_cd: Vec<String>,
    cmpl_ex: Vec<String>,
    chord: Vec<ChordEvt>,
    do_loop: bool,
    whole_tick: i32,
}
impl Default for CompositionDataStock {
    fn default() -> Self {
        Self {
            raw: "".to_string(),
            cmpl_cd: vec!["".to_string()],
            cmpl_ex: vec!["".to_string()],
            chord: Vec::new(),
            do_loop: true,
            whole_tick: 0,
        }
    }
}
impl CompositionDataStock {
    pub fn get_final(&self, part: i16) -> ElpsMsg {
        ElpsMsg::Cmp(
            part,
            ChordData {
                whole_tick: self.whole_tick as i16,
                do_loop: self.do_loop,
                evts: self.chord.clone(),
            },
        )
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        if let Some(cmpl) = complement_composition(input_text) {
            self.cmpl_cd = cmpl[0].clone();
            self.cmpl_ex = cmpl[1].clone();
            println!("complement_composition: {:?} exp: {:?}", cmpl[0], cmpl[1]);
            true
        } else {
            println!("Composition input failed!");
            false
        }
    }
    pub fn set_recombined(&mut self, tick_for_onemsr: i32, tick_for_onebeat: i32) {
        if self.cmpl_cd == [""] {
            // clear
            self.chord = Vec::new();
            println!("no_composition...");
            return;
        }

        // 3.recombined data
        let (whole_tick, do_loop, rcmb) =
            recombine_to_chord_loop(&self.cmpl_cd, tick_for_onemsr, tick_for_onebeat);
        self.chord = rcmb;
        self.do_loop = do_loop;
        self.whole_tick = whole_tick;
        println!(
            "final_composition: {:?} whole_tick: {:?}",
            self.chord, self.whole_tick
        );
    }
}
