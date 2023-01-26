//  Created by Hasebe Masahiko on 2023/0x/xx.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
//use std::thread;
//use std::time::Duration;

//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
pub struct ElapseStack {
    _ui_hndr: mpsc::Sender<String>,
}

impl ElapseStack {
    pub fn new(_ui_hndr: mpsc::Sender<String>) -> Self {
        Self {
            _ui_hndr,
        }
    }
    pub fn periodic(&mut self, msg: Result<String, TryRecvError>) -> bool {
        //thread::sleep(Duration::from_millis(500));
        match msg {
            Ok(n)  => {
                println!("msg is {}", n);
                n == "quit"
            },
            Err(TryRecvError::Disconnected) => true,// Wrong!
            Err(TryRecvError::Empty) => false,      // No event
        }
    }
}