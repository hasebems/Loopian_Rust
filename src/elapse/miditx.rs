//  Created by Hasebe Masahiko on 2023/01/28
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use crate::setting::*;
use midir::{MidiOutput, /*MidiOutputPort,*/ MidiOutputConnection};
use std::error::Error;

pub struct MidiTx {
    connection_tx: Option<Box<MidiOutputConnection>>,
    connection_tx_led: Option<Box<MidiOutputConnection>>,
    connection_ext_loopian: Option<Box<MidiOutputConnection>>,
}

impl MidiTx {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        // Port が二つとも見つからなければ、コネクトできなければエラー
        let mut me = MidiTx {
            connection_tx: None,
            connection_tx_led: None,
            connection_ext_loopian: None,
        };

        // Get an output port (read from console if multiple are available)
        let driver = MidiOutput::new("Loopian_tx")?;
        let out_ports = driver.ports();
        if out_ports.is_empty() {
            return Err("no output port found".into());
        }

        // 全outputを表示
        for (i, p) in out_ports.iter().enumerate() {
            let driver = MidiOutput::new("Loopian_tx")?;
            let drv_name = driver.port_name(p).unwrap();
            println!("[MIDI Output] No.{}: {}", i, drv_name);
        }
        let mut an_least_one = false;
        for (i, p) in out_ports.iter().enumerate() {
            let driver = MidiOutput::new("Loopian_tx")?;
            let drv_name = driver.port_name(p).unwrap();
            if drv_name.find(MIDI_OUT).is_some() {
                match driver.connect(p, "loopian_tx1") {
                    Ok(c) => {
                        me.connection_tx = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as Piano>", i, drv_name);
                    }
                    Err(_e) => {
                        println!("Connection Failed! for No.{}", i);
                    }
                }
            } else if drv_name.find(MIDI_DEVICE).is_some() {
                match driver.connect(p, "loopian_tx2") {
                    Ok(c) => {
                        me.connection_tx_led = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as LED>", i, drv_name);
                    }
                    Err(_e) => {
                        println!("Connection Failed! for No.{}", i);
                    }
                }
            } else if drv_name.find(MIDI_EXT_OUT).is_some() {
                match driver.connect(p, "loopian_tx3") {
                    Ok(c) => {
                        me.connection_ext_loopian = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as Ext>", i, drv_name);
                    }
                    Err(_e) => {
                        println!("Connection Failed! for No.{}", i);
                    }
                }
            } else {
                println!("[no connect]: {}", drv_name);
            }
        }
        if !an_least_one {
            return Err("port not connected!".into());
        }
        Ok(me)
    }
    pub fn midi_out(&mut self, status: u8, dt1: u8, dt2: u8, to_led: bool) {
        if let Some(cnct) = self.connection_tx.as_mut() {
            let status_with_ch = status & 0xf0; // ch.1
            let _ = cnct.send(&[status_with_ch, dt1, dt2]);
        }
        if let Some(cnct) = self.connection_ext_loopian.as_mut() {
            let status_with_ch = (status & 0xf0) + 10; // ch.11
            let _ = cnct.send(&[status_with_ch, dt1, dt2]);
        }
        if to_led {
            self.midi_out_for_led(status, dt1, dt2);
        }
    }
    pub fn midi_out_for_led(&mut self, status: u8, dt1: u8, dt2: u8) {
        if let Some(cnctl) = self.connection_tx_led.as_mut() {
            let midi_cmnd = status & 0xf0;
            if midi_cmnd == 0x90 || midi_cmnd == 0x80 {
                let status_with_ch = midi_cmnd | 0x0f; // ch.16
                let _ = cnctl.send(&[status_with_ch, dt1, dt2]);
            }
        }
    }
}
