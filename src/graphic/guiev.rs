//  Created by Hasebe Masahiko on 2024/09/29.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::fs;
use std::sync::{mpsc, mpsc::*};

use crate::cmd::cmdparse::LoopianCmd;
use crate::elapse::tickgen::CrntMsrTick;
use crate::lpnlib::*;

pub struct GuiEv {
    ui_hndr: mpsc::Receiver<UiMsg>,
    has_gui: bool,
    indicator: Vec<String>,
    graphic_ev: Vec<NoteUiEv>,
    crnt_msr: CrntMsrTick,
    numerator: i32,
    denomirator: i32,
    during_play: bool,
}
impl GuiEv {
    pub fn new(ui_hndr: mpsc::Receiver<UiMsg>, has_gui: bool) -> Self {
        let mut indicator = vec![String::from("---"); MAX_INDICATOR];
        indicator[0] = "C".to_string();
        indicator[1] = DEFAULT_BPM.to_string();
        Self {
            ui_hndr,
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
    pub fn get_graphic_ev(&self) -> Option<Vec<NoteUiEv>> {
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
    /// Play Thread からの、8indicator表示/PC時のFile Loadメッセージを受信する処理
    pub fn read_from_ui_hndr(&mut self, cmd: &mut LoopianCmd) -> u8 {
        loop {
            match self.ui_hndr.try_recv() {
                Ok(msg) => {
                    if let Some(ptn) = self.set_indicator(msg, cmd) {
                        return ptn;
                    }
                }
                Err(TryRecvError::Disconnected) => break, // Wrong!
                Err(TryRecvError::Empty) => break,
            }
        }
        NO_MIDI_VALUE
    }
    fn set_indicator(&mut self, msg: UiMsg, cmd: &mut LoopianCmd) -> Option<u8> {
        match msg {
            UiMsg::NewMeasure => {
                self.indicator[0] = cmd.get_indicator_key_stock();
            }
            UiMsg::BpmUi(bpm) => {
                self.indicator[1] = format!("{}", bpm);
            }
            UiMsg::Beat(nume, denomi) => {
                self.indicator[2] = format!("{}/{}", nume, denomi);
                self.numerator = nume;
                self.denomirator = denomi;
            }
            UiMsg::TickUi(during_play, m, b, t) => {
                let p = if during_play { ">" } else { "" };
                self.indicator[3] = format!("{} {} : {} : {:>03}", p.to_string(), m, b, t);
                self.during_play = during_play;
                self.crnt_msr.msr = m;
                let base_tick = DEFAULT_TICK_FOR_ONE_MEASURE / self.denomirator;
                self.crnt_msr.tick = b * base_tick + t;
            }
            UiMsg::PartUi(pnum, pui) => {
                if pui.exist {
                    let loop_msr = format!("{}/{}", pui.msr_in_loop, pui.all_msrs);
                    self.indicator[4 + pnum] = format!("{} {}", loop_msr, pui.chord_name);
                } else if pui.flow {
                    let loop_msr = "FLOW".to_string();
                    self.indicator[4 + pnum] = format!("{} {}", loop_msr, pui.chord_name);
                } else {
                    self.indicator[4 + pnum] = "---".to_string();
                }
            }
            UiMsg::NoteUi(note_ev) => {
                self.graphic_ev.push(note_ev);
            }
            UiMsg::ChangePtn(ptn) => {
                self.get_pcmsg_from_midi(ptn, cmd);
                return Some(ptn);
            }
        }
        None
    }
    fn get_pcmsg_from_midi(&mut self, pc_num: u8, cmd: &mut LoopianCmd) {
        // MIDI PC Message (1-128)
        println!("Get Command!: {:?}", pc_num);
        if pc_num < MAX_PATTERN_NUM {
            let fname = format!("{}.lpn", pc_num);
            let command_stk = self.load_lpn_when_pc(fname);
            for one_cmd in command_stk.iter() {
                let _answer = cmd.set_and_responce(one_cmd);
            }
        }
    }
    fn load_lpn_when_pc(&mut self, fname: String) -> Vec<String> {
        let mut command: Vec<String> = Vec::new();
        let path = "pattern/".to_owned() + &fname;
        println!("Pattern File: {}", path);
        match fs::read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    let mut comment = false;
                    if line.len() > 1 {
                        // コメントでないか、過去の 2023.. が書かれてないか
                        let notxt = line[0..2].to_string();
                        if notxt == "//" || notxt == "20" {
                            comment = true;
                        }
                    }
                    if line.len() > 0 && !comment {
                        command.push(line.to_string());
                    }
                }
            }
            Err(_err) => println!("Can't open a file"),
        };
        command
    }
}
