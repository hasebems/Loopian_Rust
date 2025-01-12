//  Created by Hasebe Masahiko on 2024/09/29.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::elapse::tickgen::CrntMsrTick;
use crate::lpnlib::*;

pub const INDC_KEY: usize = 0;
pub const INDC_BPM: usize = 1;
pub const INDC_METER: usize = 2;
pub const INDC_TICK: usize = 3;
pub const INDC_PART: usize = 4;
pub const MAX_INDICATOR: usize = 8;

//*******************************************************************
//  Stock GUI Event from Text Input by User,
//      and Indicator Text: 0:Key, 1:BPM, 2:Meter, 3:Tick, 4-7:Part
//*******************************************************************
pub struct GuiEv {
    has_gui: bool,
    indicator: Vec<String>,
    graphic_ev: Vec<GraphicEv>,
    crnt_msr: CrntMsrTick,
    numerator: i32,
    denomirator: i32,
    during_play: bool,
}
impl GuiEv {
    pub fn new(has_gui: bool) -> Self {
        let mut indicator = vec![String::from("---"); MAX_INDICATOR];
        indicator[INDC_KEY] = "C".to_string();
        indicator[INDC_BPM] = DEFAULT_BPM.to_string();
        Self {
            has_gui,
            indicator,
            graphic_ev: Vec::new(),
            crnt_msr: CrntMsrTick::default(),
            numerator: 4,
            denomirator: 4,
            during_play: false,
        }
    }
    pub fn get_part_txt(&self, input_part: usize) -> &str {
        match input_part {
            LEFT1 => "L1",
            LEFT2 => "L2",
            RIGHT1 => "R1",
            RIGHT2 => "R2",
            _ => "__",
        }
    }
    pub fn get_indicator(&self, num: usize) -> &str {
        &self.indicator[num]
    }
    pub fn get_graphic_ev(&self) -> Option<Vec<GraphicEv>> {
        if self.has_gui {
            Some(self.graphic_ev.clone())
        } else {
            None
        }
    }
    pub fn clear_graphic_ev(&mut self) {
        self.graphic_ev.clear();
    }
    pub fn get_msr_tick(&self) -> CrntMsrTick {
        if self.during_play {
            self.crnt_msr
        } else {
            CrntMsrTick::default()
        }
    }
    pub fn set_indicator(&mut self, msg: UiMsg, key: String) {
        match msg {
            UiMsg::NewMeasure => {
                // 小節頭の時のみ、key 表示を更新する
                self.indicator[INDC_KEY] = key.clone();
            }
            UiMsg::NewBeat(beat) => {
                self.graphic_ev.push(GraphicEv::BeatEv(beat));
            }
            UiMsg::BpmUi(bpm) => {
                self.indicator[INDC_BPM] = format!("{}", bpm);
            }
            UiMsg::Meter(nume, denomi) => {
                self.indicator[INDC_METER] = format!("{}/{}", nume, denomi);
                self.numerator = nume;
                self.denomirator = denomi;
            }
            UiMsg::TickUi(during_play, m, b, t) => {
                let p = if during_play { ">" } else { " " };
                let msr = if m != 0 { m } else { 1 };
                self.indicator[INDC_TICK] = format!("{}{}:{}:{:>03}", p, msr, b, t);
                self.during_play = during_play;
                self.crnt_msr.msr = m;
                let base_tick = DEFAULT_TICK_FOR_ONE_MEASURE / self.denomirator;
                self.crnt_msr.tick = (b - 1) * base_tick + t;
                self.crnt_msr.tick_for_onemsr = base_tick * self.numerator;
            }
            UiMsg::PartUi(pnum, pui) => {
                if pui.exist {
                    let loop_msr = format!(" {}/{}", pui.msr_in_loop, pui.all_msrs);
                    self.indicator[INDC_PART + pnum] = format!(" {} {}", loop_msr, pui.chord_name);
                } else if pui.flow {
                    let loop_msr = "FLOW".to_string();
                    self.indicator[INDC_PART + pnum] = format!(" {} {}", loop_msr, pui.chord_name);
                } else {
                    self.indicator[INDC_PART + pnum] = "  ---".to_string();
                }
            }
            UiMsg::NoteUi(note_ev) => {
                self.graphic_ev.push(GraphicEv::NoteEv(note_ev));
            }
            _ => {}
        }

        //  シーケンサが動作していない時は、key変更は即時反映する
        //  この関数は定期的に呼ばれる前提
        if !self.during_play {
            self.indicator[INDC_KEY] = key;
        }
    }
}
