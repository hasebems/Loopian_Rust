//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use super::txt2seq_ana::*;
use super::txt2seq_cmps::*;
use super::txt2seq_phr::*;
use crate::lpnlib::*;

//*******************************************************************
//          Seq Data Stock Struct
//*******************************************************************
// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
#[derive(Debug)]
pub struct SeqDataStock {
    pdt: Vec<Vec<PhraseDataStock>>,
    cdt: [CompositionDataStock; MAX_COMPOSITION_PART],
    input_mode: InputMode,
    cluster_memory: String,
    raw_additional: String,
    tick_for_onemsr: i32,
    tick_for_beat: i32,
    bpm: i16,
}
impl SeqDataStock {
    pub fn new() -> Self {
        let mut pd = Vec::new();
        for i in 0..MAX_KBD_PART {
            let mut vari = Vec::new();
            let base_note = Self::default_base_note(i);
            for _ in 0..(MAX_VARIATION + 1) {
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
            tick_for_beat: DEFAULT_TICK_FOR_QUARTER,
            bpm: DEFAULT_BPM,
        }
    }
    pub fn get_pdstk(&self, part: usize, vari: PhraseAs) -> &PhraseDataStock {
        let num = match vari {
            PhraseAs::Normal => 0,
            PhraseAs::Variation(v) => v,
            PhraseAs::Measure(_m) => MAX_VARIATION,
        };
        &self.pdt[part][num]
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
        vari: PhraseAs,
        mut input_text: String,
    ) -> Option<bool> {
        if let Some(rs) = self.check_if_additional_phrase(input_text.clone()) {
            input_text = rs;
        } else {
            return Some(true); // additional なら true
        }
        if part < MAX_KBD_PART {
            let num = match vari {
                PhraseAs::Normal => 0,
                PhraseAs::Variation(v) => v,
                PhraseAs::Measure(_m) => MAX_VARIATION,
            };
            if self.pdt[part][num].set_raw(input_text, &self.cluster_memory) {
                self.pdt[part][num].set_recombined(
                    self.input_mode,
                    self.bpm,
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                );
                return Some(false);
            }
        }
        None
    }
    pub fn del_raw_phrase(&mut self, part: usize) {
        if part < MAX_KBD_PART {
            for i in 0..(MAX_VARIATION + 1) {
                if self.pdt[part][i].set_raw("[]".to_string(), &self.cluster_memory) {
                    self.pdt[part][i].set_recombined(
                        self.input_mode,
                        self.bpm,
                        self.tick_for_onemsr,
                        self.tick_for_beat,
                    );
                }
            }
        }
    }
    pub fn set_raw_composition(&mut self, part: usize, input_text: String) -> bool {
        if part < MAX_COMPOSITION_PART && self.cdt[part].set_raw(input_text) {
            self.cdt[part].set_recombined(self.tick_for_onemsr, self.tick_for_beat);
            return true;
        }
        false
    }
    pub fn change_beat(&mut self, numerator: i16, denomirator: i16) {
        #[cfg(feature = "verbose")]
        println!("beat: {}/{}", numerator, denomirator);
        self.tick_for_onemsr =
            DEFAULT_TICK_FOR_ONE_MEASURE * (numerator as i32) / (denomirator as i32);
        self.tick_for_beat = DEFAULT_TICK_FOR_QUARTER * 4 / (denomirator as i32);
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
                epd.set_recombined(
                    self.input_mode,
                    self.bpm,
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                );
            }
        }
        update
    }
    pub fn change_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }
    pub fn check_if_additional_phrase(&mut self, raw: String) -> Option<String> {
        let strlen = raw.len();
        if strlen >= 9 && raw.contains("].rpt(") && &raw[(strlen - 2)..] == ")+" {
            let input_txt = split_by('.', raw);
            let rpt_cnt;
            if let Some(r) = extract_number_from_parentheses(&input_txt[1]) {
                rpt_cnt = r;
            } else {
                rpt_cnt = 1;
            }
            let plen = input_txt[0].len();
            for i in 0..(rpt_cnt + 1) {
                if i == 0 && self.raw_additional.is_empty() {
                    // 1st time
                    self.raw_additional = input_txt[0][0..(plen - 1)].to_string();
                } else {
                    // 2nd and more time
                    self.raw_additional += &input_txt[0][1..(plen - 1)];
                }
            }
            None
        } else if strlen >= 2 && &raw[(strlen - 2)..] == "]+" {
            if self.raw_additional.is_empty() {
                // 1st time
                self.raw_additional = raw[0..(strlen - 2)].to_string();
            } else {
                // 2nd and more time
                self.raw_additional += &raw[1..(strlen - 2)];
            }
            // additional (..]+) なら None を返す
            None
        } else {
            let mut newraw = raw.clone();
            if !self.raw_additional.is_empty() {
                // last time
                newraw = self.raw_additional.clone() + &raw[1..];
                #[cfg(feature = "verbose")]
                println!("Additional Phrase: {:?}", newraw);
                self.raw_additional = String::from("");
            }
            Some(newraw)
        }
    }
    fn recombine_phr_all(&mut self) {
        for pd in self.pdt.iter_mut() {
            for epd in pd.iter_mut() {
                epd.set_recombined(
                    self.input_mode,
                    self.bpm,
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                );
            }
        }
    }
    fn recombine_all(&mut self) {
        for (i, pd) in self.pdt.iter_mut().enumerate() {
            for epd in pd.iter_mut() {
                epd.set_recombined(
                    self.input_mode,
                    self.bpm,
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                );
            }
            self.cdt[i].set_recombined(self.tick_for_onemsr, self.tick_for_beat);
        }
    }
    fn default_base_note(part_num: usize) -> i32 {
        (DEFAULT_NOTE_NUMBER as i32) + 12 * ((part_num as i32) - 2)
    }
}

//*******************************************************************
//          Phrase Data Stock Struct
//*******************************************************************
#[derive(Debug)]
pub struct PhraseDataStock {
    base_note: i32,
    raw: String,
    cmpl_nt: Vec<String>,
    cmpl_ex: Vec<String>,
    phr: Vec<PhrEvt>,
    ana: Vec<AnaEvt>,
    atrb: Vec<Option<i16>>,
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
            atrb: vec![None],
            do_loop: true,
            whole_tick: 0,
        }
    }
    pub fn _get_cmpl_nt(&self) -> &Vec<String> {
        // for test
        &self.cmpl_nt
    }
    pub fn get_phr(&self) -> &Vec<PhrEvt> {
        &self.phr
    }
    pub fn get_final(&self, part: i16, vari: PhraseAs) -> ElpsMsg {
        let do_loop = vari == PhraseAs::Normal && self.do_loop;
        ElpsMsg::Phr(
            part,
            PhrData {
                whole_tick: self.whole_tick as i16,
                do_loop,
                evts: self.phr.clone(),
                ana: self.ana.clone(),
                vari,
                auftakt: if self.atrb[0].is_some() {
                    self.atrb[0].unwrap_or(0)
                } else {
                    0
                },
            },
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
        #[cfg(feature = "verbose")]
        println!(
            "complement_phrase: {:?} exp: {:?} atrb: {:?}",
            cmpl.0, cmpl.1, cmpl.2
        );
        true
    }
    pub fn set_recombined(
        &mut self,
        input_mode: InputMode,
        bpm: i16,
        tick_for_onemsr: i32,
        tick_for_beat: i32,
    ) {
        if self.cmpl_nt == [""] {
            println!("no_phrase...");
            //  clear
            self.phr = Vec::new();
            self.ana = Vec::new();
            self.whole_tick = 0;
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
        self.phr = beat_filter(&self.phr, bpm, tick_for_onemsr, tick_for_beat);
        #[cfg(feature = "verbose")]
        {
            println!("final_phrase: {:?}", self.phr);
            println!(
                "whole_tick: {:?} do_loop: {:?}",
                self.whole_tick, self.do_loop
            );
            println!("analyse: {:?}", self.ana);
        }
    }
}

//*******************************************************************
//          Composition Data Stock Struct
//*******************************************************************
#[derive(Debug)]
pub struct CompositionDataStock {
    raw: String,
    cmpl_cd: Vec<String>,
    chord: Vec<ChordEvt>,
    do_loop: bool,
    whole_tick: i32,
}
impl Default for CompositionDataStock {
    fn default() -> Self {
        Self {
            raw: "".to_string(),
            cmpl_cd: vec!["".to_string()],
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
                measure: NOTHING,
            },
        )
    }
    pub fn set_raw(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        if let Some(cmpl) = complement_composition(input_text) {
            self.cmpl_cd = cmpl.clone();
            #[cfg(feature = "verbose")]
            println!("complement_composition: {:?}", cmpl);
            true
        } else {
            println!("Composition input failed!");
            false
        }
    }
    pub fn set_recombined(&mut self, tick_for_onemsr: i32, tick_for_beat: i32) {
        if self.cmpl_cd == [""] {
            // clear
            self.chord = Vec::new();
            println!("no_composition...");
            return;
        }

        // 3.recombined data
        let (whole_tick, do_loop, rcmb) =
            recombine_to_chord_loop(&self.cmpl_cd, tick_for_onemsr, tick_for_beat);
        self.chord = rcmb;
        self.do_loop = do_loop;
        self.whole_tick = whole_tick;
        #[cfg(feature = "verbose")]
        println!(
            "final_composition: {:?} whole_tick: {:?}",
            self.chord, self.whole_tick
        );
    }
}
