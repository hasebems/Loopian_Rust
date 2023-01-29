//  Created by Hasebe Masahiko on 2023/01/22.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::{Instant, Duration};

use super::midi::MidiTx;

//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
//  3. MIDI Out の生成と管理
pub struct ElapseStack {
    _ui_hndr: mpsc::Sender<String>,
    mdx: MidiTx,
    start_time: Instant,
    count: u32,
}

impl ElapseStack {
    pub fn new(_ui_hndr: mpsc::Sender<String>) -> Option<Self> {
        match MidiTx::connect() {
            Ok(c)   => Some(Self {
                _ui_hndr,
                mdx: c,
                start_time: Instant::now(),
                count: 0,
            }),
            Err(_e) => None,
        } 
    }
    pub fn periodic(&mut self, msg: Result<String, TryRecvError>) -> bool {
        let mut end = false;
        match msg {
            Ok(n)  => {
                println!("msg is {}", n);
                end = n == "quit";
            },
            Err(TryRecvError::Disconnected) => return true,// Wrong!
            Err(TryRecvError::Empty) => {},      // No event
        }
        let crnt_time = Instant::now();
        let et = crnt_time-self.start_time;
        if et > Duration::from_secs(1) {
            self.start_time = crnt_time;
            self.count += 1;
            if self.count%2 == 1 {
                self.mdx.midi_out(0x90,0x40,0x60);
            }
            else {
                self.mdx.midi_out(0x80,0x40,0x40);
            }
        }
        return end
    }
}