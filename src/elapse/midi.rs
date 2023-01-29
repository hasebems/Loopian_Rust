//  Created by Hasebe Masahiko on 2023/01/28
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use std::error::Error;
use std::io::{stdin, stdout, Write};
use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};

pub struct MidiTx {
    connection: Box<MidiOutputConnection>,
}

impl MidiTx {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        let driver = MidiOutput::new("Loopian_tx")?;
        // Get an output port (read from console if multiple are available)
        let out_ports = driver.ports();
        let out_port: &MidiOutputPort = match out_ports.len() {
            0 => return Err("no output port found".into()),
            1 => {
                println!("Choosing the only available output port: {}", driver.port_name(&out_ports[0]).unwrap());
                &out_ports[0]
            },
            _ => {
                println!("\nAvailable output ports:");
                for (i, p) in out_ports.iter().enumerate() {
                    println!("{}: {}", i, driver.port_name(p).unwrap());
                }
                print!("Please select output port: ");
                stdout().flush()?;
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                out_ports.get(input.trim().parse::<usize>()?)
                         .ok_or("invalid output port selected")?
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