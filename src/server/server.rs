//  Created by Hasebe Masahiko on 2024/07/12.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
#[cfg(feature = "raspi")]
use rppal::gpio::{Gpio, InputPin, Level};
#[cfg(feature = "raspi")]
use std::error::Error;
use std::io;

use crate::cmd::cmdparse;
use crate::gen_elapse_thread;
use crate::graphic::guiev::GuiEv;
use crate::lpnlib::*;

//Raspberry Pi5 pin
#[cfg(feature = "raspi")]
pub const RASPI_PIN_FOR_QUIT: u8 = 26;
#[cfg(feature = "raspi")]
pub const RASPI_PIN_FOR_RECONNECT: u8 = 16;

pub struct LoopianServer {
    //input_text: String,
    guiev: GuiEv,
    cmd: cmdparse::LoopianCmd,
    cui_mode: bool,
}
impl LoopianServer {
    pub fn new() -> Self {
        let (txmsg, rxui) = gen_elapse_thread();
        Self {
            //input_text: "".to_string(),
            guiev: GuiEv::new(rxui, false),
            cmd: cmdparse::LoopianCmd::new(txmsg),
            cui_mode: false,
        }
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
            if let Some(answer) = srv.cmd.set_and_responce(&input) {
                println!("{}", answer.0);
            }
        } else {
            //  Read imformation from StackElapse/Gpio
            let rtn = srv.guiev.read_from_ui_hndr(&mut srv.cmd);
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
