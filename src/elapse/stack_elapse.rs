//  Created by Hasebe Masahiko on 2023/01/22.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::{Instant, Duration};

use super::tickgen::TickGen;
use super::midi::MidiTx;
use super::elapse::Elapse;
use super::elapse_part::Part;
use crate::lpnlib::MAX_PART_COUNT;

//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
//  3. MIDI Out の生成と管理
pub struct ElapseStack {
    ui_hndr: mpsc::Sender<String>,
    mdx: MidiTx,
    start_time: Instant,
    crnt_time: Instant,
    count: u32,
    during_play: bool,
    display_time: Instant,
    tg: TickGen,
    elapseVec: Vec<Box<dyn Elapse>>,
}

impl ElapseStack {
    pub fn new(ui_hndr: mpsc::Sender<String>) -> Option<Self> {
        match MidiTx::connect() {
            Ok(c)   => {
                let mut vp = Vec::new();
                for _ in 0..MAX_PART_COUNT {
                    vp.push(Part::new())
                }
                Some(Self {
                    ui_hndr,
                    mdx: c,
                    start_time: Instant::now(),
                    crnt_time: Instant::now(),
                    count: 0,
                    during_play: false,
                    display_time: Instant::now(),
                    tg: TickGen::new(),
                    elapseVec: vp,
                })
            }
            Err(_e) => None,
        } 
    }
    pub fn add_elapse(&mut self, elps: Box<dyn Elapse>) {
        self.elapseVec.push(elps);
    }
    pub fn periodic(&mut self, msg: Result<String, TryRecvError>) -> bool {
        self.crnt_time = Instant::now();
        match msg {
            Ok(n)  => {
                if n == "quit" {return true;}
                else {self.parse_msg(n);}
            },
            Err(TryRecvError::Disconnected) => return true,// Wrong!
            Err(TryRecvError::Empty) => {},      // No event
        }

        // play 中でなければ return
        if !self.during_play {return false;}

        //  新tick計算
        let crnt_msr_tick = self.tg.get_crnt_msr_tick(self.crnt_time);
        if crnt_msr_tick.new_msr {  // 小節を跨いだ場合
            // change beat event

            // change bpm event

            // fine
        }

        loop {
            /*let et = crnt_time-self.start_time;
            if et > Duration::from_secs(1) {
                self.start_time = crnt_time;
                self.count += 1;
                if self.count%2 == 1 {
                    self.mdx.midi_out(0x90,0x40,0x60);
                    self.send_msg_to_ui(&self.count.to_string());
                }
                else {
                    self.mdx.midi_out(0x80,0x40,0x40);
                }
            }*/
            break;
        }

        //  for GUI
        let elapse_time = self.crnt_time-self.display_time;
        if elapse_time > Duration::from_millis(50) {
            self.display_time = self.crnt_time;
            let (m,b,t,_c) = self.tg.get_tick();
            let beat_disp = "3".to_owned() + &m.to_string() + " : " + &b.to_string() + " : " + &t.to_string();
            self.send_msg_to_ui(&beat_disp);
        }

        return false
    }
    fn send_msg_to_ui(&self, msg: &str) {
        match self.ui_hndr.send(msg.to_string()) {
            Err(e) => println!("Something happened on MPSC! {}",e),
            _ => {},
        }
    }
    fn start(&mut self) {
        self.during_play = true;
        self.tg.start(self.crnt_time);
    }
    fn stop(&mut self) {
        self.during_play = false;
    }
    fn parse_msg(&mut self, msg: String) {
        println!("msg is {}", msg);
        if msg == "start" {self.start();}
        else if msg == "play" {self.start();}
        else if msg == "stop" {self.stop();}
    }
}