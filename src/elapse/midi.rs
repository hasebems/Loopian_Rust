//  Created by Hasebe Masahiko on 2023/01/28
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use std::error::Error;
//use std::io::{stdin, stdout, Write};
use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};

pub struct MidiTx {
    connection: Box<MidiOutputConnection>,
}

pub const MIDI_OUT: &str = "IACdriver";

impl MidiTx {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        let driver = MidiOutput::new("Loopian_tx")?;
        // Get an output port (read from console if multiple are available)
        let out_ports = driver.ports();
        let out_port: &MidiOutputPort = match out_ports.len() {
            0 => return Err("no output port found".into()),
            _ => {
                println!("\nAvailable output ports:");
                let mut out_port: &MidiOutputPort = &out_ports[0];
                let mut found = false;
                for (i, p) in out_ports.iter().enumerate() {
                    let drv_name = driver.port_name(p).unwrap();
                    let mut selected = "";
                    if drv_name.find(MIDI_OUT) != None {
                        out_port = p;
                        found = true;
                        selected = "<selected>"
                    }
                    println!("{}: {} {}", i, drv_name, selected);
                }
                if found {out_port}
                else {return Err("no output port found".into());}
            }
        };
        match driver.connect(out_port, "loopian_tx") {
            Ok(c) => Ok(Self {connection: Box::new(c),}),
            Err(_e) => return Err("Connection Failed!".into()),
        }
    }
    pub fn midi_out(&mut self, status: u8, dt1: u8, dt2: u8) {
        // We're ignoring errors in here
        let _ = self.connection.send(&[status, dt1, dt2]);
    }
}