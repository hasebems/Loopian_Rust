//  Created by Hasebe Masahiko on 2024/07/12.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
#[cfg(feature = "raspi")]
use rppal::gpio::Gpio;
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
pub fn cui_loop() -> Result<(), Box<dyn Error>> {
    let mut srv = LoopianServer::new();
    #[cfg(feature = "raspi")]
    {
        let pin_number = 17;
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_number)?.into_input();
    }
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
            #[cfg(feature = "raspi")]
            {
                let pin_value = pin.read();
                if !pin_value {
                    break;
                } // Gpio Button を押されたら終了
            }
        }
    }
    Ok(())
}
