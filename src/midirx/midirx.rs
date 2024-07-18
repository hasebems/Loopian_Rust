//  Created by Hasebe Masahiko on 2024/07/15.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
//use std::sync::mpsc::{Receiver, Sender};
use crate::lpnlib::{ElpsMsg::*, *};
use crate::setting::MIDI_DEVICE;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex};

#[cfg(feature = "raspi")]
use std::time::Duration;
#[cfg(feature = "raspi")]
use rppal::uart::{Parity, Uart};

//*******************************************************************
//          MIDI Rx Buffer
//*******************************************************************
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

//*******************************************************************
//          MIDI Rx
//*******************************************************************
pub struct MidiRx {
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    _conn_in: Option<MidiInputConnection<()>>,
    tx_hndr: mpsc::Sender<ElpsMsg>,
    mdr_buf: Option<Arc<Mutex<MidiRxBuf>>>,
    midi_stream_status: u8,
    midi_stream_data1: u8,
    #[cfg(feature = "raspi")]
    pub uart: Option<Uart>,
}
impl MidiRx {
    pub fn new(tx_hndr: mpsc::Sender<ElpsMsg>) -> Option<MidiRx> {
        let mut this = Self {
            _conn_in: None,
            tx_hndr,
            mdr_buf: None,
            midi_stream_status: INVALID,
            midi_stream_data1: INVALID,
            #[cfg(feature = "raspi")]
            uart: None,
        };

        let mdr_buf = Arc::new(Mutex::new(MidiRxBuf::new()));
        match this.connect(Arc::clone(&mdr_buf)) {
            Ok(()) => {
                println!("MIDI receive Connection OK.");
                this.mdr_buf = Some(mdr_buf);
                return Some(this);
            }
            Err(err) => {
                println!("{}", err);
                return None;
            }
        };
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
                        "loopian_rx1",
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
                    println!("Uart MIDI available, now!");
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
    pub fn send_msg_to_elapse(&self, msg: ElpsMsg) {
        match self.tx_hndr.send(msg) {
            Err(e) => println!("Something happened on MPSC from MIDIRx! {}", e),
            _ => {}
        }
    }
    pub fn periodic(&mut self, rx_ctrlmsg: Result<ElpsMsg, TryRecvError>) -> bool {
        self.receive_midi_event();
        match rx_ctrlmsg {
            // 制御用メッセージ’
            Ok(n) => {
                match n {
                    Ctrl(m) => {
                        if m == MSG_CTRL_QUIT {
                            return true;
                        } else if m == MSG_CTRL_START {
                            if let Ok(mut mb) = self.mdr_buf.as_ref().unwrap().lock() {
                                mb.flush(); // MIDI In Buffer をクリア
                            }
                        }
                    }
                    _ => (),
                }
            }
            Err(TryRecvError::Disconnected) => return true, // Wrong!
            Err(TryRecvError::Empty) => return false,       // No event
        }
        false
    }
    fn receive_midi_event(&mut self) {
        if let Some(msg_ext) = self.mdr_buf.as_ref().unwrap().lock().unwrap().take() {
            let msg = msg_ext.1;
            println!(
                "MIDI Received >{}: {:?} (len = {})",
                msg_ext.0,
                msg,
                msg.len()
            );
            // midi ch=12,13 のみ受信 (Loopian::ORBIT)
            let input_ch = msg[0] & 0x0f;
            if input_ch != 0x0b && input_ch != 0x0c {
                return;
            }
            self.send_msg_to_elapse(ElpsMsg::MIDIRx(msg[0], msg[1], msg[2]))
        }
        #[cfg(feature = "raspi")]
        if let Some(ref mut urx) = self.uart {
            let mut byte = [0];
            match urx.read(&mut byte) {
                Ok(c) => {
                    if c == 1 {
                        self.parse_1byte_midi(self.tx_hndr, byte[0]);
                    }
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }
    #[allow(dead_code)]
    fn parse_1byte_midi(&mut self, input_data: u8) {
        if input_data & 0x80 == 0x80 {
            match input_data {
                0xfa => {}
                0xf8 => {}
                0xfc => {}
                _ => {
                    if input_data & 0x0f == 0x0a {
                        self.midi_stream_status = input_data;
                    }
                }
            }
        } else {
            match self.midi_stream_status & 0xf0 {
                0x90 => {
                    if self.midi_stream_data1 != INVALID {
                        // LED
                        let dt1 = self.midi_stream_data1;
                        self.send_msg_to_elapse(ElpsMsg::MIDIRx(
                            self.midi_stream_status,
                            dt1,
                            input_data,
                        ));
                        println!(
                            "ExtLoopian: {}-{}-{}",
                            self.midi_stream_status, dt1, input_data
                        );
                        self.midi_stream_data1 = INVALID;
                    } else if input_data >= MIN_NOTE_NUMBER && input_data <= MAX_NOTE_NUMBER {
                        // note num がピアノ鍵盤範囲外なら受け付けない
                        self.midi_stream_data1 = input_data;
                    } else {
                        self.midi_stream_data1 = INVALID;
                    }
                }
                0x80 => {
                    if self.midi_stream_data1 != INVALID {
                        // LED
                        let dt1 = self.midi_stream_data1;
                        self.send_msg_to_elapse(ElpsMsg::MIDIRx(
                            self.midi_stream_status,
                            dt1,
                            input_data,
                        ));
                        println!(
                            "ExtLoopian: {}-{}-{}",
                            self.midi_stream_status, dt1, input_data
                        );
                        self.midi_stream_data1 = INVALID;
                    } else {
                        self.midi_stream_data1 = input_data;
                    }
                }
                0xc0 => {
                    self.send_msg_to_elapse(ElpsMsg::MIDIRx(
                        self.midi_stream_status,
                        input_data,
                        0,
                    ));
                    self.midi_stream_status = INVALID;
                    self.midi_stream_data1 = INVALID;
                }
                0xb0 => {}
                _ => {}
            }
        }
    }
}
