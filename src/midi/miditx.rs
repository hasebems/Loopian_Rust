//  Created by Hasebe Masahiko on 2023/01/28
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use crate::file::settings::Settings;
use midir::{MidiOutput, /*MidiOutputPort,*/ MidiOutputConnection};

pub struct MidiTx {
    tx_available: bool,
    connection_tx: Option<Box<MidiOutputConnection>>,
    connection_tx_led1: Option<Box<MidiOutputConnection>>,
    connection_tx_led2: Option<Box<MidiOutputConnection>>,
    connection_ext_loopian: Option<Box<MidiOutputConnection>>,
}

impl MidiTx {
    // Port が二つとも見つからなければ、コネクトできなければエラーメッセージを返す
    pub fn connect() -> (Self, Option<String>) {
        let mut this = MidiTx {
            tx_available: false,
            connection_tx: None,
            connection_tx_led1: None,
            connection_tx_led2: None,
            connection_ext_loopian: None,
        };

        // Get an output port (read from console if multiple are available)
        let out_ports;
        match MidiOutput::new("Loopian_tx") {
            Ok(driver) => {
                out_ports = driver.ports();
                if out_ports.is_empty() {
                    return (this, Some("no output port found".into()));
                }
            }
            Err(_e) => {
                return (this, Some("Midi out initialize failed".into()));
            }
        }

        // 全outputを表示
        for (i, p) in out_ports.iter().enumerate() {
            match MidiOutput::new("Loopian_tx") {
                Ok(driver) => {
                    let drv_name = driver.port_name(p).unwrap();
                    println!("[MIDI Output] No.{}: {}", i, drv_name);
                }
                Err(_e) => continue,
            }
        }

        let midi_out = &Settings::load_settings().midi.midi_out;
        let midi_ext_out = &Settings::load_settings().midi.midi_ext_out;
        let midi_device = &Settings::load_settings().midi.midi_device;
        let mut an_least_one = false;
        for (i, p) in out_ports.iter().enumerate() {
            let driver;
            let drv_name;
            match MidiOutput::new("Loopian_tx") {
                Ok(o) => {
                    driver = o;
                    drv_name = driver.port_name(p).unwrap();
                    //println!("[MIDI Output] No.{}: {}", i, drv_name);
                }
                Err(_e) => continue,
            }
            if drv_name.contains(midi_out) {
                match driver.connect(p, "loopian_tx1") {
                    Ok(c) => {
                        this.connection_tx = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as Piano>", i, drv_name);
                    }
                    Err(_e) => {
                        println!("Connection Failed! for No.{}", i);
                    }
                }
            } else if drv_name.contains(midi_device) {
                if this.connection_tx_led1.is_none() {
                    match driver.connect(p, "loopian_tx2") {
                        Ok(c) => {
                            this.connection_tx_led1 = Some(Box::new(c));
                            an_least_one = true;
                            println!("{}: {} <as LED1>", i, drv_name);
                        }
                        Err(_e) => {
                            println!("Connection Failed! for No.{}", i);
                        }
                    }
                } else {
                    match driver.connect(p, "loopian_tx2") {
                        Ok(c) => {
                            this.connection_tx_led2 = Some(Box::new(c));
                            an_least_one = true;
                            println!("{}: {} <as LED2>", i, drv_name);
                        }
                        Err(_e) => {
                            println!("Connection Failed! for No.{}", i);
                        }
                    }
                }
            } else if drv_name.contains(midi_ext_out) {
                match driver.connect(p, "loopian_tx3") {
                    Ok(c) => {
                        this.connection_ext_loopian = Some(Box::new(c));
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
        if an_least_one {
            this.tx_available = true;
            (this, None)
        } else {
            (this, Some("port not connected!".into()))
        }
    }
    pub fn midi_out(&mut self, status: u8, dt1: u8, dt2: u8, to_led: bool) {
        if !self.tx_available {
            return;
        }
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
        if !self.tx_available {
            return;
        }
        let midi_cmnd = status & 0xf0;
        if midi_cmnd == 0x90 || midi_cmnd == 0x80 {
            let status_with_ch = midi_cmnd | 0x0f; // ch.16
            if let Some(cnctl) = self.connection_tx_led1.as_mut() {
                let _ = cnctl.send(&[status_with_ch, dt1, dt2]);
            }
            if let Some(cnctl) = self.connection_tx_led2.as_mut() {
                let _ = cnctl.send(&[status_with_ch, dt1, dt2]);
            }
        }
    }
    pub fn midi_out_only_for_another(&mut self, status: u8, dt1: u8, dt2: u8) {
        if !self.tx_available {
            return;
        }
        if let Some(cnct) = self.connection_ext_loopian.as_mut() {
            let status_with_ch = (status & 0xf0) + 10; // ch.11
            let _ = cnct.send(&[status_with_ch, dt1, dt2]);
        }
    }
}
