//  Created by Hasebe Masahiko on 2024/07/12.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
#[cfg(feature = "raspi")]
use rppal::gpio::{Gpio, InputPin, Level};
#[cfg(feature = "raspi")]
use std::error::Error;
use std::fs;
use std::io;
use std::sync::{mpsc, mpsc::*};

//use crate::cmd::cmdparse;
use crate::gen_elapse_thread;
//use crate::graphic::guiev::GuiEv;
use crate::file::input_txt::InputText;
use crate::lpnlib::*;

//Raspberry Pi5 pin
#[cfg(feature = "raspi")]
pub const RASPI_PIN_FOR_QUIT: u8 = 26;
#[cfg(feature = "raspi")]
pub const RASPI_PIN_FOR_RECONNECT: u8 = 16;

pub struct LoopianServer {
    ui_hndr: mpsc::Receiver<UiMsg>,
    itxt: InputText,
    cui_mode: bool,
}
impl LoopianServer {
    pub fn new() -> Self {
        let (txmsg, rxui) = gen_elapse_thread();
        Self {
            ui_hndr: rxui,
            itxt: InputText::new(txmsg),
            cui_mode: false,
        }
    }
    fn read_from_midi(&mut self) -> u8 {
        loop {
            match self.ui_hndr.try_recv() {
                Ok(msg) => match msg {
                    UiMsg::ChangePtn(ptn) => {
                        self.get_pcmsg_from_midi(ptn);
                        return ptn;
                    }
                    _ => {}
                },
                Err(TryRecvError::Disconnected) => break, // Wrong!
                Err(TryRecvError::Empty) => break,
            }
        }
        NO_MIDI_VALUE
    }
    fn get_pcmsg_from_midi(&mut self, pc_num: u8) {
        // MIDI PC Message (1-128)
        println!("Get Command!: {:?}", pc_num);
        if pc_num < MAX_PATTERN_NUM {
            let fname = format!("{}.lpn", pc_num);
            let command_stk = self.load_lpn_when_pc(fname);
            for one_cmd in command_stk.iter() {
                let _answer = self.itxt.set_and_responce(one_cmd);
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
pub fn cui_loop() {
    let mut srv = LoopianServer::new();
    // Raspberry Pi5 のピン配の初期設定
    #[cfg(feature = "raspi")]
    let pinq = get_rasp_pin(RASPI_PIN_FOR_QUIT);
    #[cfg(feature = "raspi")]
    let pinr = get_rasp_pin(RASPI_PIN_FOR_RECONNECT);
    #[cfg(feature = "raspi")]
    let mut reconnect_sw = false;
    loop {
        if srv.cui_mode {
            // 標準入力から文字列を String で取得
            let mut buf = String::new();
            io::stdin()
                .read_line(&mut buf)
                .expect("Failed to read line.");
            let input = buf.trim().to_string();
            if input == "q" || input == "quit" {
                break; // 終了
            }
            if let Some(answer) = srv.itxt.set_and_responce(&input) {
                println!("{}", answer.0);
            }
        } else {
            //  Read imformation from StackElapse/Gpio
            let rtn = srv.read_from_midi();
            if rtn == MAX_PATTERN_NUM {
                break; // 終了
            } else if rtn == MAX_PATTERN_NUM + 1 {
                srv.cui_mode = true;
            }
            #[cfg(feature = "raspi")]
            {
                if let Ok(ref pin) = pinq {
                    if pin.read() == Level::Low {
                        // Gpio Button を押されたら終了
                        break;
                    }
                }
                if let Ok(ref pin) = pinr {
                    let lvl = pin.read();
                    if lvl == Level::Low && !reconnect_sw {
                        // reconnect
                        srv.cmd.send_reconnect();
                        reconnect_sw = true;
                    } else if lvl == Level::High {
                        reconnect_sw = false;
                    }
                }
            }
        }
    }
}
#[cfg(feature = "raspi")]
pub fn get_rasp_pin(pin: u8) -> Result<InputPin, Box<dyn Error>> {
    let gpio = Gpio::new()?;
    Ok(gpio.get(pin)?.into_input())
}
