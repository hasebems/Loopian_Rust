//  Created by Hasebe Masahiko on 2024/07/15.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use crate::file::settings::Settings;
use crate::lpnlib::*;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};

#[cfg(feature = "raspi")]
use rppal::uart::{Parity, Uart};
#[cfg(feature = "raspi")]
use std::time::Duration;

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
        if self.buf.is_empty() {
            None
        } else {
            self.buf.pop()
        }
    }
}

//*******************************************************************
//          MIDI Rx
//*******************************************************************
pub struct MidiRx {
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    _conn_in: [Option<MidiInputConnection<()>>; 2],
    mdr_buf: [Option<Arc<Mutex<MidiRxBuf>>>; 2],
    rx_cnct_num: [usize; 2],
    tx_hndr: mpsc::Sender<ElpsMsg>,
    midi_stream_status: u8,
    midi_stream_data1: u8,
    keynote: u8,
    #[cfg(feature = "raspi")]
    pub uart: Option<Uart>,
}
impl MidiRx {
    pub fn new(tx_hndr: mpsc::Sender<ElpsMsg>) -> Option<MidiRx> {
        let mut this = Self {
            _conn_in: [None, None],
            mdr_buf: [None, None],
            rx_cnct_num: [NONE_NUM, NONE_NUM],
            tx_hndr,
            midi_stream_status: INVALID,
            midi_stream_data1: INVALID,
            keynote: 0,
            #[cfg(feature = "raspi")]
            uart: None,
        };
        if this.set_connect() {
            Some(this)
        } else {
            None
        }
    }
    fn set_connect(&mut self) -> bool {
        // USB MIDI 変数初期化
        self.mdr_buf = [None, None];
        self.rx_cnct_num = [NONE_NUM, NONE_NUM];

        self.connect_uart();
        self.display_usb_midi_list();
        let mut num_to_avoid = NONE_NUM;
        for i in 0..2 {
            let mdr_buf = Arc::new(Mutex::new(MidiRxBuf::new()));
            match self.connect(Arc::clone(&mdr_buf), i, num_to_avoid) {
                Ok(num) => {
                    self.mdr_buf[i] = Some(mdr_buf);
                    self.rx_cnct_num[i] = num;
                    num_to_avoid = num;
                }
                Err(err) => {
                    println!("{}", err);
                    return false;
                }
            };
        }
        println!("MIDI receive Connection OK.");
        true
    }
    fn display_usb_midi_list(&mut self) {
        let mut midi_in = MidiInput::new("midir reading input").unwrap();
        midi_in.ignore(Ignore::None);

        let in_ports = midi_in.ports();
        if in_ports.is_empty() {
            return;
        }
        // 全inputを表示
        for (i, p) in in_ports.iter().enumerate() {
            let drv_name = midi_in.port_name(p).unwrap();
            println!("[MIDI Input] No.{}: {}", i, drv_name);
        }
    }
    fn connect_uart(&mut self) {
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
                    println!("UART MIDI connection failed.");
                }
            }
        }
    }
    fn connect(
        &mut self,
        mdr_buf: Arc<Mutex<MidiRxBuf>>,
        idx_num: usize,
        num_to_avoid: usize,
    ) -> Result<usize, &str> {
        let midi_in = MidiInput::new("midir reading input").unwrap();
        let in_ports = midi_in.ports();
        if in_ports.is_empty() {
            return Err("no input port found");
        }
        let mut in_port: Option<&MidiInputPort> = None;
        let mut ret_num = NONE_NUM;
        for (i, p) in in_ports.iter().enumerate() {
            let drv_name = midi_in.port_name(p).unwrap();
            let dev_name = &Settings::load_settings().midi.midi_device;
            if drv_name.contains(dev_name) && i != num_to_avoid {
                println!(
                    "{}: {} <as Flow{}>",
                    i,
                    midi_in.port_name(p).unwrap(),
                    idx_num + 1 // 1ori
                );
                in_port = in_ports.get(i);
                ret_num = i;
                break;
            }
        }
        if let Some(port) = in_port {
            let port_name = &format!("loopian_rx{}", idx_num);
            self._conn_in[idx_num] = Some(
                midi_in
                    .connect(
                        port,
                        port_name,
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
        Ok(ret_num)
    }
    fn send_msg_to_elapse(&self, msg: ElpsMsg) {
        if let Err(e) = self.tx_hndr.send(msg) {
            println!("Something happened on MPSC from MIDIRx! {}", e);
        }
    }
    pub fn periodic(&mut self, rx_ctrlmsg: Result<ElpsMsg, TryRecvError>) -> bool {
        self.receive_midi_event();
        match rx_ctrlmsg {
            // 制御用メッセージ
            Ok(n) => {
                if let ElpsMsg::Ctrl(m) = n {
                    if m == MSG_CTRL_QUIT {
                        return true;
                    } else if m == MSG_CTRL_START {
                        for i in 0..2 {
                            if let Ok(mut mb) = self.mdr_buf[i].as_ref().unwrap().lock() {
                                mb.flush(); // MIDI In Buffer をクリア
                            }
                        }
                    } else if m == MSG_CTRL_MIDI_RECONNECT {
                        let _b = self.set_connect();
                    }
                }
            }
            Err(TryRecvError::Disconnected) => return true, // Wrong!
            Err(TryRecvError::Empty) => return false,       // No event
        }
        false
    }
    fn receive_midi_event(&mut self) {
        for i in 0..2 {
            if self.mdr_buf[i].is_some() {
                if let Some(msg_ext) = self.mdr_buf[i].as_ref().unwrap().lock().unwrap().take() {
                    let msg = msg_ext.1;
                    #[cfg(feature = "verbose")]
                    {
                        let length = msg.len();
                        println!(
                            "MIDI{} Received >{}: {:x}-{:x}-{:x} (len = {})",
                            i + 1,
                            msg_ext.0,
                            msg[0],
                            msg[1],
                            if length > 2 { msg[2] } else { 0 },
                            length
                        );
                    }
                    // midi ch=12,13 のみ受信 (Loopian::ORBIT)
                    let input_ch = msg[0] & 0x0f;
                    if input_ch != 0x0b && input_ch != 0x0c {
                        return;
                    }
                    if msg.len() == 2 {
                        self.send_msg_to_elapse(ElpsMsg::MIDIRx(msg[0], msg[1], 0, 0));
                    } else {
                        self.send_msg_to_elapse(ElpsMsg::MIDIRx(msg[0], msg[1], msg[2], 0));
                    }
                }
            }
        }
        #[cfg(feature = "raspi")]
        if let Some(ref mut urx) = self.uart {
            let mut byte = [0];
            match urx.read(&mut byte) {
                Ok(c) => {
                    if c == 1 {
                        self.parse_1byte_midi(byte[0]);
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
                            0,
                        ));
                        println!(
                            "ExtLoopian: {}-{}-{}",
                            self.midi_stream_status, dt1, input_data
                        );
                        self.midi_stream_data1 = INVALID;
                    } else if (MIN_NOTE_NUMBER..=MAX_NOTE_NUMBER).contains(&input_data) {
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
                            0,
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
                        0,
                    ));
                    self.midi_stream_status = INVALID;
                    self.midi_stream_data1 = INVALID;
                }
                0xa0 => {
                    if self.midi_stream_data1 != INVALID {
                        // Chord from External Loopian
                        let dt1 = self.midi_stream_data1;
                        if dt1 == 0x7f {
                            self.keynote = input_data; // 一旦保持しておく
                        } else {
                            self.send_msg_to_elapse(ElpsMsg::MIDIRx(
                                self.midi_stream_status,
                                dt1,
                                input_data,
                                self.keynote,
                            ));
                            println!(
                                "Chord from ExtLoopian: root:{},ctbl:{},key:{}",
                                dt1, input_data, self.keynote
                            );
                        }
                        self.midi_stream_data1 = INVALID;
                    } else {
                        self.midi_stream_data1 = input_data;
                    }
                }
                _ => {}
            }
        }
    }
}
