//  Created by Hasebe Masahiko on 2023/01/28
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use std::error::Error;
use midir::{MidiOutput, /*MidiOutputPort,*/ MidiOutputConnection};

pub struct MidiTx {
    connection_tx: Option<Box<MidiOutputConnection>>,
    connection_tx_led: Option<Box<MidiOutputConnection>>,
    //connection_rx_orbit: Box<MidiOutputConnection>,
}

pub const MIDI_OUT: &str = "IACdriver";
pub const MIDI_OUT_LED: &str = "Arduino";

impl MidiTx {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        // Port が二つとも見つからなければ、コネクトできなければエラー
        let mut me = MidiTx {connection_tx: None, connection_tx_led: None,};

        // Get an output port (read from console if multiple are available)
        let driver = MidiOutput::new("Loopian_tx")?;
        let out_ports = driver.ports();
        if out_ports.len() == 0 {return Err("no output port found".into());}

        let mut an_least_one = false;
        for (i, p) in out_ports.iter().enumerate() {
            let driver = MidiOutput::new("Loopian_tx")?;
            let drv_name = driver.port_name(p).unwrap();
            if drv_name.find(MIDI_OUT) != None {
                match driver.connect(p, "loopian_tx") {
                    Ok(c) => {
                        me.connection_tx = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as Piano>", i, drv_name);
                    },
                    Err(_e) => {
                        println!("Connection Failed! for No.{}",i);
                    },
                }
            }
            else if drv_name.find(MIDI_OUT_LED) != None {
                match driver.connect(p, "loopian_tx") {
                    Ok(c) => {
                        me.connection_tx_led = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as LED>", i, drv_name);
                    },
                    Err(_e) => {
                        println!("Connection Failed! for No.{}",i);
                    },
                }
            }
            else {
                println!("[no connect]{}: {}", i, drv_name);
            }
        }
        if !an_least_one {return Err("port not connected!".into())}
        Ok(me)
    }
    pub fn midi_out(&mut self, status: u8, dt1: u8, dt2: u8) {
        if let Some(cnct) = self.connection_tx.as_mut() {
            let _ = cnct.send(&[status, dt1, dt2]);
        }
    }
}