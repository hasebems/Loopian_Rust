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
    pub fn get_pdstk(&self, part: usize, vari: PhraseAs) -> Option<&PhraseDataStock> {
        let num = match vari {
            PhraseAs::Normal => 0,
            PhraseAs::Variation(v) => v,
            PhraseAs::Measure(_m) => MAX_VARIATION,
        };
        if self.pdt[part][num].send_enable {
            Some(&self.pdt[part][num])
        } else {
            None
        }
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
                    false,
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
                        false,
                    );
                }
            }
        }
    }
    pub fn set_raw_composition(&mut self, part: usize, input_text: String) -> bool {
        if part < MAX_COMPOSITION_PART && self.cdt[part].set_raw_cd(input_text) {
            self.cdt[part].set_recombined_cd(self.tick_for_onemsr, self.tick_for_beat);
            return true;
        }
        false
    }
    pub fn change_beat(&mut self, numerator: i16, denomirator: i16) {
        #[cfg(feature = "verbose")]
        println!("beat: {numerator}/{denomirator}");
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
                    true,
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
                println!("Additional Phrase: {newraw:?}");
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
                    true,
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
                    true,
                );
            }
            self.cdt[i].set_recombined_cd(self.tick_for_onemsr, self.tick_for_beat);
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
    cmpl: Option<Box<PhraseComplemented>>,
    phr: Vec<PhrEvt>,
    ana: Vec<AnaEvt>,
    do_loop: bool,
    whole_tick: i32,
    send_enable: bool,
}
impl PhraseDataStock {
    fn new(base_note: i32) -> Self {
        Self {
            base_note,
            raw: "".to_string(),
            cmpl: None,
            phr: Vec::new(),
            ana: Vec::new(),
            do_loop: true,
            whole_tick: 0,
            send_enable: true,
        }
    }
    pub fn _get_cmpl_nt(&self) -> Vec<String> {
        // for test
        if let Some(cmpl) = &self.cmpl {
            cmpl.note_info.clone()
        } else {
            vec!["".to_string()]
        }
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
                auftakt: if self.cmpl.is_some() {
                    self.cmpl.as_ref().unwrap().note_attribute[0].unwrap_or(0)
                } else {
                    0
                },
            },
        )
    }
    pub fn set_raw(&mut self, input_text: String, cluster_word: &str) -> bool {
        // 1.raw
        self.send_enable = true;
        self.raw = input_text.clone();

        // 2.complement data
        self.cmpl = Some(complement_phrase(input_text, cluster_word));
        if cfg!(feature = "verbose") {
            println!(
                "complement_phrase: {:?} exp: {:?} atrb: {:?} accia: {:?}",
                if self.cmpl.is_some() {
                    self.cmpl.as_ref().unwrap().note_info.clone()
                } else {
                    ["-".to_string()].to_vec()
                },
                if self.cmpl.is_some() {
                    self.cmpl.as_ref().unwrap().music_exp.clone()
                } else {
                    ["-".to_string()].to_vec()
                },
                if self.cmpl.is_some() {
                    self.cmpl.as_ref().unwrap().note_attribute.clone()
                } else {
                    [None].to_vec()
                },
                if self.cmpl.is_some() {
                    self.cmpl.as_ref().unwrap().accia_info.clone()
                } else {
                    [None].to_vec()
                }
            );
        }
        true
    }
    pub fn set_recombined(
        &mut self,
        input_mode: InputMode,
        bpm: i16,
        tick_for_onemsr: i32,
        tick_for_beat: i32,
        resend: bool,
    ) {
        if self.cmpl.is_none() {
            //  clear
            self.phr = Vec::new();
            self.ana = Vec::new();
            self.whole_tick = 0;
            return;
        }

        // 3.recombined data
        let (whole_tick, do_loop, rcmb) = recombine_to_internal_format(
            self.cmpl.as_ref().unwrap(),
            input_mode,
            self.base_note,
            tick_for_onemsr,
        );
        if resend && !do_loop {
            // do_loop が false の場合は、再生しない
            #[cfg(feature = "verbose")]
            println!("do_loop is false, not updating phrase data.");
            self.send_enable = false;
            return;
        }
        self.phr = rcmb;
        self.do_loop = do_loop;
        self.whole_tick = whole_tick;

        // 4.analysed data
        self.ana = analyse_data(&self.phr, &self.cmpl.as_ref().unwrap().music_exp);

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
    chord: Vec<CmpEvt>,
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
            CmpData {
                whole_tick: self.whole_tick as i16,
                do_loop: self.do_loop,
                evts: self.chord.clone(),
                measure: NOTHING,
            },
        )
    }
    pub fn set_raw_cd(&mut self, input_text: String) -> bool {
        // 1.raw
        self.raw = input_text.clone();

        // 2.complement data
        if let Some(cmpl) = complement_composition(input_text) {
            self.cmpl_cd = cmpl.clone();
            #[cfg(feature = "verbose")]
            println!("complement_composition: {cmpl:?}");
            true
        } else {
            println!("Composition input failed!");
            false
        }
    }
    pub fn set_recombined_cd(&mut self, tick_for_onemsr: i32, tick_for_beat: i32) {
        if self.cmpl_cd == [""] {
            // clear
            self.chord = Vec::new();
            self.whole_tick = 0;
            self.do_loop = true;
            //println!("no_composition...");
        } else {
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
}
