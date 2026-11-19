//  Created by Hasebe Masahiko on 2024/07/15.
//  Copyright (c) 2024 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
extern crate midir;

use crate::common::lpnlib::*;
use crate::file::settings::Settings;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc;

#[cfg(feature = "raspi")]
use rppal::uart::{Parity, Uart};
#[cfg(feature = "raspi")]
use std::time::Duration;

//*******************************************************************
//          MIDI Rx
//*******************************************************************
/// USB MIDI は midir のコールバックで受信し、その場で ElpsMsg に変換して Elapse に送る。
/// MidiRxThread のスレッドは、制御用メッセージ（Quit / 再接続など）が来るまで眠っている。
/// （raspi の UART は通知の仕組みがないため、1ms ごとに見に行く）
pub struct MidiRxThread {
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    _conn_in: [Option<MidiInputConnection<()>>; 2],
    rx_cnct_num: [usize; 2],
    tx_hndr: mpsc::Sender<ElpsMsg>,
    midi_stream_status: u8,
    midi_stream_data1: u8,
    keynote: u8,
    midi_input_ch: Arc<AtomicU8>, // コールバックからも参照するため共有する
    #[cfg(feature = "raspi")]
    pub uart: Option<Uart>,
}
impl MidiRxThread {
    #[cfg(feature = "raspi")]
    const UART_POLLING_INTERVAL: Duration = Duration::from_millis(1);

    /// MidiRx のインスタンスを生成し、MIDI IN ポートに接続する
    pub fn new(tx_hndr: mpsc::Sender<ElpsMsg>) -> Option<MidiRxThread> {
        let mut this = Self {
            _conn_in: [None, None],
            rx_cnct_num: [NONE_NUM, NONE_NUM],
            tx_hndr,
            midi_stream_status: INVALID,
            midi_stream_data1: INVALID,
            keynote: 0,
            midi_input_ch: Arc::new(AtomicU8::new(0)),
            #[cfg(feature = "raspi")]
            uart: None,
        };
        if this.set_connect() { Some(this) } else { None }
    }
    fn set_connect(&mut self) -> bool {
        // USB MIDI 変数初期化
        self.rx_cnct_num = [NONE_NUM, NONE_NUM];

        self.connect_uart();
        self.display_usb_midi_list();
        let mut num_to_avoid = NONE_NUM;
        for i in 0..2 {
            match self.connect(i, num_to_avoid) {
                Ok(num) => {
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
            println!("--MIDI Input List-- No.{}: {}", i, drv_name);
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
    fn connect(&mut self, idx_num: usize, num_to_avoid: usize) -> Result<usize, &str> {
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
                    "<<Input Connected!>> No.{}: {} <as Flow{}>",
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
            let tx_hndr = self.tx_hndr.clone();
            let midi_input_ch = Arc::clone(&self.midi_input_ch);
            self._conn_in[idx_num] = Some(
                midi_in
                    .connect(
                        port,
                        port_name,
                        move |stamp, message, _| {
                            // MIDI を受信したときに midir から呼ばれる
                            Self::receive_midi_event(
                                idx_num,
                                stamp,
                                message,
                                &tx_hndr,
                                midi_input_ch.load(Ordering::Relaxed),
                            );
                        },
                        (),
                    )
                    .unwrap(),
            );
        }
        Ok(ret_num)
    }
    /// ElpsMsg を Elapse に送信する
    fn send_msg_to_elapse(&self, msg: ElpsMsg) {
        Self::send_to(&self.tx_hndr, msg);
    }
    fn send_to(tx_hndr: &mpsc::Sender<ElpsMsg>, msg: ElpsMsg) {
        if let Err(e) = tx_hndr.send(msg) {
            println!("Something happened on MPSC from MIDIRx! {}", e);
        }
    }
    /// stack_elapse::gen_midirx_thread() で生成したスレッドで呼び出される
    /// 制御用メッセージを待ち、ElpsMsg::Ctrl(MsgCtrl::Quit) を受け取るか、
    /// 送信側がなくなったら戻る
    pub fn run(&mut self, rx_ctrl: mpsc::Receiver<ElpsMsg>) {
        loop {
            #[cfg(not(feature = "raspi"))]
            let msg = match rx_ctrl.recv() {
                Ok(msg) => msg,
                Err(_) => return, // 送信側がなくなった
            };
            #[cfg(feature = "raspi")]
            let msg = {
                self.receive_uart_event();
                match rx_ctrl.recv_timeout(Self::UART_POLLING_INTERVAL) {
                    Ok(msg) => msg,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            };
            if self.handle_ctrl_msg(msg) {
                return;
            }
        }
    }
    /// 制御用メッセージの処理。Quit なら true を返す
    fn handle_ctrl_msg(&mut self, msg: ElpsMsg) -> bool {
        if let ElpsMsg::Ctrl(m) = msg {
            if m == MsgCtrl::Quit {
                return true;
            } else if m == MsgCtrl::MidiReconnect {
                let _b = self.set_connect();
            }
            // MsgCtrl::Start: 受信したものはすぐ送っているので、クリアするバッファはない
        } else if let ElpsMsg::Set(MsgSet::MidiInputCh(ch)) = msg {
            println!("Set Flow MIDI Input Ch(MIDI Rx): {}", ch);
            self.midi_input_ch.store(ch as u8, Ordering::Relaxed);
        }
        false
    }
    /// USB MIDI のイベントを ElpsMsg::MIDIRx に変換して Elapse に送信する
    /// （midir のコールバックから呼ばれる）
    fn receive_midi_event(
        idx: usize,
        stamp: u64,
        msg: &[u8],
        tx_hndr: &mpsc::Sender<ElpsMsg>,
        midi_input_ch: u8,
    ) {
        if msg.is_empty() {
            return;
        }
        let data1 = msg.get(1).copied().unwrap_or(0);
        let data2 = msg.get(2).copied().unwrap_or(0);
        if cfg!(feature = "verbose") {
            println!(
                "MIDI{} Received >{}: {:x}-{:x}-{:x} (len = {})",
                idx + 1,
                stamp,
                msg[0],
                data1,
                data2,
                msg.len()
            );
        } else {
            println!("{:x}-{:x}-{:x} ", msg[0], data1, data2);
        }
        // midi ch=12,13 のみ受信 (Loopian::ORBIT/CUBIT)
        let status = msg[0] & 0xf0;
        let input_ch = msg[0] & 0x0f;
        if input_ch != 0x0b && input_ch != 0x0c && midi_input_ch != input_ch + 1 {
            return;
        }
        if status == 0xc0 || status == 0xd0 {
            Self::send_to(tx_hndr, ElpsMsg::MIDIRx(msg[0], data1, 0, 0));
        } else if status == 0x90
            || status == 0x80
            || status == 0xa0
            || status == 0xb0
            || status == 0xe0
        {
            Self::send_to(tx_hndr, ElpsMsg::MIDIRx(msg[0], data1, data2, 0));
        }
    }
    /// UART MIDI のイベントをチェックし、届いているバイトをすべて処理する
    #[cfg(feature = "raspi")]
    fn receive_uart_event(&mut self) {
        loop {
            let Some(ref mut urx) = self.uart else {
                return;
            };
            let mut byte = [0];
            match urx.read(&mut byte) {
                Ok(1) => self.parse_1byte_midi(byte[0]),
                Ok(_) => return, // 届いているバイトはもうない
                Err(e) => {
                    println!("{}", e);
                    return;
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
