//  Created by Hasebe Masahiko on 2023/01/28
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use crate::setting::*;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use midir::{MidiOutput, /*MidiOutputPort,*/ MidiOutputConnection};
#[cfg(feature = "raspi")]
use rppal::uart::{Parity, Uart};
use std::error::Error;
use std::sync::{Arc, Mutex};
#[cfg(feature = "raspi")]
use std::time::Duration;

pub struct MidiTx {
    connection_tx: Option<Box<MidiOutputConnection>>,
    connection_tx_led: Option<Box<MidiOutputConnection>>,
    //connection_rx_orbit: Box<MidiOutputConnection>,
}

impl MidiTx {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        // Port が二つとも見つからなければ、コネクトできなければエラー
        let mut me = MidiTx {
            connection_tx: None,
            connection_tx_led: None,
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
                match driver.connect(p, "loopian_tx") {
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
                match driver.connect(p, "loopian_tx") {
                    Ok(c) => {
                        me.connection_tx_led = Some(Box::new(c));
                        an_least_one = true;
                        println!("{}: {} <as LED>", i, drv_name);
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
    pub fn midi_out(&mut self, status: u8, dt1: u8, dt2: u8) {
        if let Some(cnct) = self.connection_tx.as_mut() {
            let status_with_ch = status & 0xf0; // ch.1
            let _ = cnct.send(&[status_with_ch, dt1, dt2]);
        }
        if let Some(cnctl) = self.connection_tx_led.as_mut() {
            let midi_cmnd = status & 0xf0;
            if midi_cmnd == 0x90 || midi_cmnd == 0x80 {
                let status_with_ch = midi_cmnd | 0x0f; // ch.16
                let _ = cnctl.send(&[status_with_ch, dt1, dt2]);
            }
        }
    }
}

pub struct MidiRxBuf {
    buf: Vec<(u64, Vec<u8>)>,
}
impl MidiRxBuf {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
    pub fn flush(&mut self) {
        self.buf.clear();
    }
    pub fn put(&mut self, tm: u64, msg: Vec<u8>) {
        self.buf.insert(0, (tm, msg));
    }
    pub fn take(&mut self) -> Option<(u64, Vec<u8>)> {
        if self.buf.len() > 0 {
            self.buf.pop()
        } else {
            None
        }
    }
}

pub struct MidiRx {
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    _conn_in: Option<MidiInputConnection<()>>,
    #[cfg(feature = "raspi")]
    pub uart: Option<Uart>,
}

impl MidiRx {
    pub fn new() -> Self {
        Self {
            _conn_in: None,
            #[cfg(feature = "raspi")]
            uart: None,
        }
    }
    pub fn connect(&mut self, mdr_buf: Arc<Mutex<MidiRxBuf>>) -> Result<(), &str> {
        let mut midi_in = MidiInput::new("midir reading input").unwrap();
        midi_in.ignore(Ignore::None);
        let in_ports = midi_in.ports();
        if in_ports.len() == 0 {
            return Err("no input port found");
        }

        let mut in_port: Option<&MidiInputPort> = None;
        // 全inputを表示
        for (i, p) in in_ports.iter().enumerate() {
            let drv_name = midi_in.port_name(p).unwrap();
            println!("[MIDI Input] No.{}: {}", i, drv_name);
        }
        for (i, p) in in_ports.iter().enumerate() {
            let drv_name = midi_in.port_name(p).unwrap();
            if drv_name.find(MIDI_DEVICE).is_some() {
                println!("{}: {} <as Flow>", i, midi_in.port_name(p).unwrap());
                in_port = in_ports.get(i);
                break;
            }
        }
        if let Some(port) = in_port {
            self._conn_in = Some(
                midi_in
                    .connect(
                        port,
                        "midir-read-input",
                        move |stamp, message, _| {
                            let msg = message.iter().fold(Vec::new(), |mut s, i| {
                                s.push(*i);
                                s
                            });
                            mdr_buf.lock().unwrap().put(stamp, msg);
                        },
                        (),
                    )
                    .unwrap(),
            );
        }
        #[cfg(feature = "raspi")]
        {
            // UARTポートを38400 bpsで設定
            match Uart::with_path("/dev/ttyAMA0", 38400, Parity::None, 8, 1) {
                Ok(mut u) => {
                    let _ = u.set_read_mode(0, Duration::ZERO);
                    self.uart = Some(u);
                }
                Err(_e) => {
                    self.uart = None;
                    return Err("UART MIDI connection failed.");
                }
            }
        }
        Ok(())
    }
}
