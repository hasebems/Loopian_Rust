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
use crate::gen_thread;
use crate::setting::*;

pub struct LoopianServer {
    //input_text: String,
    cmd: cmdparse::LoopianCmd,
    cui_mode: bool,
}
impl LoopianServer {
    pub fn new() -> Self {
        let (txmsg, rxui) = gen_thread();
        Self {
            //input_text: "".to_string(),
            cmd: cmdparse::LoopianCmd::new(txmsg, rxui, false),
            cui_mode: false,
        }
    }
}
pub fn cui_loop() {
    let mut srv = LoopianServer::new();
    #[cfg(feature = "raspi")]
    let pin_or = get_rasp_pin(17);

    let _ = srv.cmd.set_and_responce("flow");
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
            let rtn = srv.cmd.read_from_ui_hndr();
            if rtn == MAX_PATTERN_NUM {
                break; // 終了
            } else if rtn == MAX_PATTERN_NUM + 1 {
                srv.cui_mode = true;
            }
            #[cfg(feature = "raspi")] {
                if let Ok(ref pin) = pin_or {
                    if pin.read() == Level::Low {
                        // Gpio Button を押されたら終了
                        //break;
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