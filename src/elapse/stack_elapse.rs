//  Created by Hasebe Masahiko on 2023/01/22.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::vec::Vec;

use super::elapse_base::*;
use super::elapse_damper::DamperPart;
use super::elapse_flow::Flow;
use super::elapse_loop_phr::PhraseLoop;
use super::elapse_part::Part;
use super::tickgen::{CrntMsrTick, RitType, TickGen};
use crate::lpnlib::{ElpsMsg::*, *};
use crate::midi::midirx::MidiRx;
use crate::midi::miditx::MidiTx;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SameKeyState {
    More,    //  まだある
    Last,    //  これが最後
    Nothing, //  もうない
}

//*******************************************************************
//          Elapse Stack Struct
//*******************************************************************
//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
//  3. MIDI Out の生成と管理
pub struct ElapseStack {
    ui_hndr: mpsc::Sender<UiMsg>,
    rx_hndr: mpsc::Receiver<ElpsMsg>,
    tx_ctrl: mpsc::Sender<ElpsMsg>,
    mdx: MidiTx,

    crnt_time: Instant,
    last_msr_tick: CrntMsrTick,
    bpm_stock: i16,
    beat_stock: Meter,
    fine_stock: bool,

    during_play: bool,
    display_time: Instant,
    tg: TickGen,
    flac: u64,
    part_vec: Vec<Rc<RefCell<Part>>>, // Part Instance が繋がれた Vec
    damper_part: Rc<RefCell<DamperPart>>,
    elapse_vec: Vec<Rc<RefCell<dyn Elapse>>>, // dyn Elapse Instance が繋がれた Vec
    key_map: [i32; (MAX_NOTE_NUMBER - MIN_NOTE_NUMBER + 1) as usize],
    limit_for_deb: i32,
}
//*******************************************************************
//          Public Method for Elapse Stack Struct
//*******************************************************************
fn gen_midirx_thread() -> (Receiver<ElpsMsg>, Sender<ElpsMsg>) {
    //  create new thread & channel
    let (txmsg, rxmsg) = mpsc::channel();
    let (txctrl, rxctrl) = mpsc::channel();
    thread::spawn(move || match MidiRx::new(txmsg /* , rxctrl*/) {
        Some(mut rx) => loop {
            if rx.periodic(rxctrl.try_recv()) {
                break;
            }
        },
        None => {
            println!("MIDI Rx thread does't work")
        }
    });
    (rxmsg, txctrl)
}
impl ElapseStack {
    pub fn new(ui_hndr: mpsc::Sender<UiMsg>) -> Self {
        let (c, e) = MidiTx::connect();
        if let Some(err) = e {
            println!("{}", err);
        }
        let mut part_vec = Vec::new();
        let mut elapse_vec = Vec::new();

        // Keyboard Part
        for i in 0..MAX_KBD_PART {
            // 同じ Part を part_vec, elapse_vec 両方に繋げる
            let pt = Part::new(i as u32, None);
            part_vec.push(Rc::clone(&pt));
            elapse_vec.push(pt as Rc<RefCell<dyn Elapse>>);
        }
        // Flow Part
        let flow = Flow::new(0, FLOW_PART as u32, false);
        elapse_vec.push(flow.clone() as Rc<RefCell<dyn Elapse>>);
        let pt = Part::new(FLOW_PART as u32, Some(flow));
        part_vec.push(Rc::clone(&pt));
        elapse_vec.push(pt as Rc<RefCell<dyn Elapse>>);
        // Damper Part
        let damper_part = DamperPart::new(DAMPER_PEDAL_PART as u32);
        elapse_vec.push(Rc::clone(&damper_part) as Rc<RefCell<dyn Elapse>>);

        let (rx_hndr, tx_ctrl) = gen_midirx_thread();
        Self {
            ui_hndr,
            rx_hndr,
            tx_ctrl,
            mdx: c,
            crnt_time: Instant::now(),
            last_msr_tick: CrntMsrTick::new(),
            bpm_stock: DEFAULT_BPM,
            beat_stock: Meter(4, 4),
            fine_stock: false,
            during_play: false,
            display_time: Instant::now(),
            tg: TickGen::new(RitType::Sigmoid),
            flac: 0,
            part_vec: part_vec.clone(),
            damper_part,
            elapse_vec,
            key_map: [0; (MAX_NOTE_NUMBER - MIN_NOTE_NUMBER + 1) as usize],
            limit_for_deb: 0,
        }
    }
    pub fn add_elapse(&mut self, elps: Rc<RefCell<dyn Elapse>>) {
        self.elapse_vec.push(elps);
    }
    pub fn _del_elapse(&mut self, search_id: ElapseId) {
        // 呼ぶとエラーが出る
        if let Some(remove_index) = self
            .elapse_vec
            .iter()
            .position(|x| x.borrow().id() == search_id)
        {
            self.elapse_vec.remove(remove_index);
        }
    }
    pub fn part(&mut self, part_num: u32) -> Option<Rc<RefCell<Part>>> {
        if let Some(index) = self
            .part_vec
            .iter()
            .position(|x| x.borrow().id().sid == part_num)
        {
            let part = Rc::clone(&self.part_vec[index]);
            Some(part)
        } else {
            None
        }
    }
    pub fn get_phr(&self, part_num: usize) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.part_vec[part_num].borrow().get_phr().cloned()
    }
    pub fn get_flow(&self) -> Option<Rc<RefCell<Flow>>> {
        self.part_vec[FLOW_PART].borrow().get_flow()
    }
    pub fn tg(&self) -> &TickGen {
        &self.tg
    }
    pub fn inc_key_map(&mut self, key_num: u8, vel: u8, pt: u8) {
        self.key_map[(key_num - MIN_NOTE_NUMBER) as usize] += 1;
        self.send_msg_to_ui(UiMsg::NoteUi(NoteUiEv { key_num, vel, pt }));
    }
    pub fn dec_key_map(&mut self, key_num: u8) -> SameKeyState {
        let idx = (key_num - MIN_NOTE_NUMBER) as usize;
        match self.key_map[idx].cmp(&1) {
            Ordering::Greater => {
                self.key_map[idx] -= 1;
                SameKeyState::More
            }
            Ordering::Equal => {
                self.key_map[idx] = 0;
                SameKeyState::Last
            }
            Ordering::Less => SameKeyState::Nothing,
        }
    }
    //    pub fn set_phrase_vari(&self, part_num: usize, vari_num: usize) {
    //        self.part_vec[part_num]
    //            .borrow_mut()
    //            .set_phrase_vari(vari_num);
    //    }
    //    pub fn set_loop_end(&self, part_num: usize) {
    //        self.part_vec[part_num].borrow_mut().set_loop_end();
    //    }
    pub fn midi_out(&mut self, status: u8, data1: u8, data2: u8) {
        self.mdx.midi_out(status, data1, data2, true);
    }
    pub fn midi_out_flow(&mut self, status: u8, data1: u8, data2: u8) {
        self.mdx.midi_out(status, data1, data2, false);
    }
    pub fn midi_out_ext(&mut self, status: u8, data1: u8, data2: u8) {
        // DIN MIDI OUT
        self.mdx.midi_out_only_for_another(status, data1, data2);
        // IAC MIDI OUT
        //self.mdx.midi_out(status, data1, data2, true);
    }
    //*******************************************************************
    //      Periodic
    //*******************************************************************
    pub fn periodic(&mut self, msg: Result<ElpsMsg, TryRecvError>) -> bool {
        self.crnt_time = Instant::now();

        // message 受信処理
        if self.handle_msg(msg) {
            self.send_msg_to_rx(ElpsMsg::Ctrl(MSG_CTRL_QUIT));
            return true;
        }

        //  新tick計算
        let mut crnt_ = CrntMsrTick::new();
        let mut beattop_ev: bool = false;
        if self.during_play {
            //  再生中ならば、現在の tick を更新
            let (msrtop, beattop, beatnum) = self.tg.gen_tick(self.crnt_time);
            beattop_ev = beattop;
            crnt_ = self.tg.get_crnt_msr_tick();
            self.last_msr_tick = crnt_;
            // 小節先頭、Beat 先頭の処理
            if msrtop {
                if self.fine_stock {
                    self.stop();
                    self.fine_stock = false;
                } else {
                    self.measure_top(&mut crnt_);
                }
            }
            if beattop {
                self.send_msg_to_ui(UiMsg::NewBeat(beatnum));
            }
        };

        //  for GUI
        self.update_gui();

        //　MIDI Rx処理
        self.check_rcv_midi(&crnt_);

        if self.during_play {
            //  Elapse の処理
            self.play_elapse(&crnt_);

            if beattop_ev {
                // Flow Part の和音を MIDI OUT する
                self.midi_chord_out(&crnt_); // process の後でコールしないとハングする
            }

            // remove ended obj
            self.destroy_finished_elps();
        }

        // play 中でなければ return
        false
    }
    fn measure_top(&mut self, crnt_: &mut CrntMsrTick) {
        // デバッグ用表示
        println!(
            "<New measure! in stack_elapse> Msr: {} Max Debcnt: {}/{} Time: {:?}",
            crnt_.msr,
            self.limit_for_deb,
            self.elapse_vec.len(),
            self.tg.get_origin_time().elapsed()
        );
        #[cfg(feature = "verbose")]
        println!("  All Elapse Obj. Num: {:?}", self.elapse_vec.len());

        // 小節先頭ならば、beat/bpm のイベント調査
        self.limit_for_deb = 0;
        // change beat event
        if self.beat_stock != self.tg.get_meter() {
            let tick_for_onemsr =
                (DEFAULT_TICK_FOR_ONE_MEASURE / self.beat_stock.1) * self.beat_stock.0;
            self.tg.change_beat_event(tick_for_onemsr, self.beat_stock);
            *crnt_ = self.tg.get_crnt_msr_tick(); //再設定
        }
        // for GUI(8indicator)
        self.update_gui_at_msrtop();
    }
    fn play_elapse(&mut self, crnt_: &CrntMsrTick) {
        // すべての Elapse Obj. のうち、現在の measure/tick より前のイベントを持つものを処理する
        let mut debcnt = 0;
        while let Some(felps) = self.pick_up_first(crnt_) {
            // 現measure/tick より前のイベントを持つ obj を返す
            #[cfg(feature = "verbose")]
            {
                let et = felps.borrow().id();
                let mt = felps.borrow().next();
                println!(
                    "<{:>02}:{:>04}> pid: {:?}, sid: {:?}, type: {:?}, nmsr: {:?}, ntick: {:?}",
                    crnt_.msr, crnt_.tick, et.pid, et.sid, et.elps_type, mt.0, mt.1
                );
            }
            felps.borrow_mut().process(crnt_, self);
            debcnt += 1;
            if debcnt > 80 {
                // 50個以上のイベントがあったら、何かがおかしい
                println!(
                    "Too many events! cnt: {}, id: {:?}",
                    debcnt,
                    felps.borrow().id()
                );
            }
            assert!(debcnt < 100, "Last ID:{:?}", felps.borrow().id()); // ERR_SE10
            if self.limit_for_deb < debcnt {
                self.limit_for_deb = debcnt;
            }
        }
    }
    //*******************************************************************
    //      handle message
    //*******************************************************************
    fn handle_msg(&mut self, msg: Result<ElpsMsg, TryRecvError>) -> bool {
        match msg {
            Ok(n) => {
                match n {
                    Ctrl(m) => {
                        if m == MSG_CTRL_QUIT {
                            return true;
                        } else {
                            self.parse_elps_msg(n)
                        }
                    }
                    _ => self.parse_elps_msg(n),
                }
                //if n[0] == MSG_QUIT {return true;}
                //else {self.parse_msg(n);}
            }
            Err(TryRecvError::Disconnected) => return true, // Wrong!
            Err(TryRecvError::Empty) => return false,       // No event
        }
        false
    }
    fn parse_elps_msg(&mut self, msg: ElpsMsg) {
        match msg {
            Ctrl(m) => self.ctrl_msg(m),
            Sync(m) => self.sync(m),
            Rit(m) => self.rit(m),
            Set(m) => self.setting_cmnd(m),
            Efct(m) => self.efct(m),
            SetMeter(m) => self.set_meter(m),
            Phr(m0, mv) => self.phrase(m0, mv),
            Cmp(m0, mv) => self.composition(m0, mv),
            PhrX(m) => self.del_phrase(m),
            CmpX(m) => self.del_composition(m),
            _ => (),
        }
    }
    fn ctrl_msg(&mut self, msg: i16) {
        if msg == MSG_CTRL_START {
            self.start(false);
        } else if msg == MSG_CTRL_STOP {
            self.stop();
        } else if msg == MSG_CTRL_FINE {
            self.fine(msg);
        } else if msg == MSG_CTRL_PANIC {
            self.panic();
        } else if msg == MSG_CTRL_RESUME {
            self.start(true);
        } else if msg == MSG_CTRL_CLEAR {
            self.clear_elapse();
        } else if msg == MSG_CTRL_MIDI_RECONNECT {
            self.reconnect();
        }
    }
    fn midi_chord_out(&mut self, crnt_: &CrntMsrTick) {
        // Flow Part の和音を MIDI OUT する
        let mut keynote = 0;
        if let Some(fl) = self.part_vec[FLOW_PART].borrow_mut().get_flow() {
            keynote = fl.borrow().get_keynote();
        }
        let (root, tbl) = {
            let mut pt_borrow = self.part_vec[FLOW_PART].borrow_mut();
            let cmp_med = pt_borrow.get_cmps_med();
            cmp_med.get_chord(crnt_)
        };
        self.midi_out_ext(0xa0, 0x7f, keynote);
        self.midi_out_ext(0xa0, root as u8, tbl as u8);
    }
    fn send_msg_to_ui(&self, msg: UiMsg) {
        if let Err(e) = self.ui_hndr.send(msg) {
            println!("Something happened on MPSC for UI! {}", e);
        }
    }
    fn send_msg_to_rx(&self, msg: ElpsMsg) {
        if let Err(e) = self.tx_ctrl.send(msg) {
            println!("Something happened on MPSC To MIDIRx! {}", e);
        }
    }
    fn check_rcv_midi(&mut self, crnt_: &CrntMsrTick) {
        match self.rx_hndr.try_recv() {
            Ok(rxmsg) => {
                if let MIDIRx(sts, nt, vel, extra) = rxmsg {
                    self.rcv_midi_msg(crnt_, sts, nt, vel, extra);
                }
            }
            Err(TryRecvError::Disconnected) => {} // Wrong!
            Err(TryRecvError::Empty) => {}
        }
    }
    fn rcv_midi_msg(&mut self, crnt_: &CrntMsrTick, sts: u8, nt: u8, vel: u8, ex: u8) {
        if sts & 0x0f == 0x0a {
            // 0a ch <from another loopian>
            if !self.during_play {
                // pattern 再生中は、External Loopian とは繋がない
                if sts & 0xe0 == 0x80 {
                    // LED を光らせる
                    self.mdx.midi_out_for_led(sts, nt, vel);
                } else if sts & 0xf0 == 0xa0 {
                    // Flow Part に和音を設定する
                    if let Some(fl) = self.part_vec[FLOW_PART].borrow_mut().get_flow() {
                        fl.borrow_mut().set_chord_for_noplay(nt, vel, ex);
                    }
                }
            }
        } else {
            // 0b/0c ch <from ORBIT>
            if (sts & 0xe0) == 0x80 {
                // 再生中 & Note Message
                let pt = self.part_vec[FLOW_PART].clone();
                pt.borrow_mut()
                    .rcv_midi_in(self, crnt_, sts & 0xf0, nt, vel);
            } else if (sts & 0xf0) == 0xc0 {
                // PCN は Pattern 切り替えに使用する
                self.send_msg_to_ui(UiMsg::ChangePtn(nt));
            }
        }
    }
    //*******************************************************************
    //      Control Message
    //*******************************************************************
    fn start(&mut self, resume: bool) {
        if self.during_play && !resume {
            return;
        }

        // すべての Part の開始beatを調べる
        let mut first_beat = FULL;
        self.part_vec.iter_mut().for_each(|x| {
            if let Some(auf) = x.borrow().get_start_beat() {
                if auf < first_beat {
                    first_beat = auf;
                }
            }
        });

        self.during_play = true;
        self.tg.start(self.crnt_time, self.bpm_stock, resume);
        let start_msr = if resume {
            self.tg.get_crnt_msr_tick().msr
        } else {
            0
        };
        for elps in self.elapse_vec.iter() {
            elps.borrow_mut().start(start_msr);
        }
        self.send_msg_to_rx(ElpsMsg::Ctrl(MSG_CTRL_START));
        #[cfg(feature = "verbose")]
        println!("<Start Playing! in stack_elapse> M:{}", start_msr);
    }
    fn panic(&mut self) {
        self.midi_out(0xb0, 0x78, 0x00);
    }
    fn stop(&mut self) {
        if !self.during_play {
            return;
        }
        self.during_play = false;
        self.last_msr_tick = CrntMsrTick::reset(self.last_msr_tick.tick_for_onemsr);
        let stop_vec = self.elapse_vec.to_vec();
        for elps in stop_vec.iter() {
            elps.borrow_mut().stop(self);
        }
        // destroy flag の立った elapse obj.を回収
        self.destroy_finished_elps();
    }
    fn clear_elapse(&mut self) {
        let clear_vec = self.elapse_vec.to_vec();
        for elps in clear_vec.iter() {
            elps.borrow_mut().clear(self);
        }
    }
    fn reconnect(&mut self) {
        let (_c, e) = MidiTx::connect();
        if let Some(err) = e {
            println!("{}", err);
        } else {
            self.send_msg_to_rx(Ctrl(MSG_CTRL_MIDI_RECONNECT));
        }
    }
    fn fine(&mut self, _msg: i16) {
        if self.tg().get_bpm() == 0 {
            self.stop();
        } else {
            self.fine_stock = true;
        }
    }
    fn sync(&mut self, part: i16) {
        let mut sync_part = [false; MAX_KBD_PART];
        if part < MAX_KBD_PART as i16 {
            sync_part[part as usize] = true;
        } else if part == MSG_SYNC_LFT {
            sync_part[LEFT1] = true;
            sync_part[LEFT2] = true;
        } else if part == MSG_SYNC_RGT {
            sync_part[RIGHT1] = true;
            sync_part[RIGHT2] = true;
        } else if part == MSG_SYNC_ALL {
            for pt in sync_part.iter_mut() {
                *pt = true;
            }
        }
        for (i, pt) in sync_part.iter().enumerate() {
            if *pt {
                let part = self.part_vec[i].clone();
                let crnt_ = self.last_msr_tick;
                part.borrow_mut().set_sync(&crnt_, self);
            }
        }
    }
    fn rit(&mut self, msg: [i16; 2]) {
        let strength_set: [(i16, i32); 3] =
            [(MSG_RIT_POCO, 80), (MSG_RIT_NRM, 60), (MSG_RIT_MLT, 40)];
        let strength_msg = msg[0] % 10;
        let bar = (msg[0] / 10) as i32;
        let target_bpm: i16;
        let strength = strength_set
            .into_iter()
            .find(|x| x.0 == strength_msg)
            .unwrap_or(strength_set[0]);
        if msg[1] == MSG2_RIT_ATMP {
            target_bpm = self.tg.get_bpm();
        } else if msg[1] == MSG2_RIT_FERMATA {
            target_bpm = 0;
        } else {
            target_bpm = msg[1];
        }
        self.tg.prepare_rit(strength.1, bar, target_bpm);
    }
    fn setting_cmnd(&mut self, msg: [i16; 2]) {
        if msg[0] == MSG_SET_BPM {
            self.bpm_stock = msg[1];
            self.tg.change_bpm(msg[1])
        } else if msg[0] == MSG_SET_KEY {
            self.part_vec
                .iter()
                .for_each(|x| x.borrow_mut().change_key(msg[1] as u8));
        } else if msg[0] == MSG_SET_TURN {
            self.part_vec
                .iter_mut()
                .for_each(|x| x.borrow_mut().set_turnnote(msg[1]));
        } else if msg[0] == MSG_SET_CRNT_MSR {
            if self.during_play {
                self.stop();
            }
            self.tg.set_crnt_msr(msg[1] as i32);
        }
    }
    fn efct(&mut self, msg: [i16; 2]) {
        if msg[0] == MSG_EFCT_DMP {
            self.damper_part.borrow_mut().set_position(msg[1]);
        } else if msg[0] == MSG_EFCT_CC70 {
            let val = if msg[1] > 127 { 127 } else { msg[1] as u8 };
            self.midi_out(0xb0, 70, val);
        }
    }
    fn set_meter(&mut self, msg: [i16; 2]) {
        self.beat_stock = Meter(msg[0] as i32, msg[1] as i32);
        self.sync(MSG_SYNC_ALL);
        if !self.during_play {
            let tick_for_onemsr = (DEFAULT_TICK_FOR_ONE_MEASURE / msg[1] as i32) * msg[0] as i32;
            self.last_msr_tick.tick_for_onemsr = tick_for_onemsr;
            self.tg.change_beat_event(tick_for_onemsr, self.beat_stock);
        }
    }
    fn phrase(&mut self, part_num: i16, evts: PhrData) {
        println!("Received Phrase Message! Part: {}", part_num);
        let crnt_ = self.last_msr_tick;
        let pt = self.part_vec[part_num as usize].clone();
        pt.borrow_mut().rcv_phr_msg(evts, &crnt_, self);
    }
    fn composition(&mut self, part_num: i16, evts: ChordData) {
        println!("Received Composition Message! Part: {}", part_num);
        self.part_vec[part_num as usize]
            .borrow_mut()
            .rcv_cmps_msg(evts, self.tg().get_beat_tick());
    }
    #[allow(dead_code)]
    fn del_phrase(&mut self, part_num: i16) {
        println!("Deleted Phrase Message! Part: {}", part_num);
        self.part_vec[part_num as usize].borrow_mut().del_phr();
    }
    fn del_composition(&mut self, part_num: i16) {
        println!("Deleted Composition Message! Part: {}", part_num);
        self.part_vec[part_num as usize]
            .borrow_mut()
            .rcv_cmps_msg(ChordData::empty(), self.tg().get_beat_tick());
    }
    //*******************************************************************
    //      Pick out playable
    //*******************************************************************
    fn pick_up_first(&self, crnt_: &CrntMsrTick) -> Option<Rc<RefCell<dyn Elapse>>> {
        let mut first: Option<Rc<RefCell<dyn Elapse>>> = None;
        for elps in self.elapse_vec.iter() {
            let (msr, tick) = elps.borrow().next();
            if (msr == crnt_.msr && tick <= crnt_.tick) || msr < crnt_.msr {
                // 現在のタイミングより前のイベントがあれば
                if let Some(felps) = first.clone() {
                    let (msrx, tickx) = felps.borrow().next();
                    if (msr < msrx)
                        || ((msr == msrx) && (tick < tickx))
                        || ((msr == msrx)
                            && (tick == tickx)
                            && (felps.borrow().prio() > elps.borrow().prio()))
                    {
                        first = Some(elps.clone());
                    }
                } else {
                    first = Some(elps.clone());
                }
            }
        }
        first
    }
    fn _pick_out_playable(&self, crnt_: &CrntMsrTick) -> Vec<Rc<RefCell<dyn Elapse>>> {
        let mut playable: Vec<Rc<RefCell<dyn Elapse>>> = Vec::new();
        for elps in self.elapse_vec.iter() {
            let (msr, tick) = elps.borrow().next();
            if (msr == crnt_.msr && tick <= crnt_.tick) || msr < crnt_.msr {
                // 現在のタイミングより前のイベントがあれば
                if playable.is_empty() {
                    // playable にまだ何も無ければ、普通に push
                    playable.push(Rc::clone(elps));
                } else {
                    // playable に、時間順になるように挿入
                    let mut after_break = false;
                    for (i, one_plabl) in playable.iter().enumerate() {
                        let (msrx, tickx) = one_plabl.borrow().next();
                        if (msr < msrx)
                            || ((msr == msrx)
                                && ((tick < tickx)
                                    || ((tick == tickx)
                                        && (one_plabl.borrow().prio() > elps.borrow().prio()))))
                        {
                            playable.insert(i, Rc::clone(elps));
                            after_break = true;
                            break;
                        }
                    }
                    if !after_break {
                        // 条件にはまらなければ最後に入れる
                        playable.push(Rc::clone(elps));
                    }
                }
            }
        }
        playable
    }
    fn destroy_finished_elps(&mut self) {
        loop {
            let mut removed_num: i32 = -1;
            for (i, elps) in self.elapse_vec.iter().enumerate() {
                if elps.borrow().destroy_me() {
                    self.elapse_vec.remove(i);
                    removed_num = i as i32;
                    break;
                }
            }
            if removed_num == -1 {
                break;
            }
        }
    }
    //*******************************************************************
    //      Update GUI
    //*******************************************************************
    fn update_gui_at_msrtop(&mut self) {
        if self.during_play {
            // key
            self.send_msg_to_ui(UiMsg::NewMeasure);
        }
    }
    /// 50-60msec に一度、表示更新のイベントを Main Thread に送る
    fn update_gui(&mut self) {
        let diff = self.crnt_time - self.display_time;
        if diff > Duration::from_millis(50 + self.flac) {
            // 表示が周期的にならないように、間隔をバラす
            self.display_time = self.crnt_time;
            // beat
            let beat = self.tg.get_meter();
            self.send_msg_to_ui(UiMsg::Meter(beat.0, beat.1));
            // bpm
            self.send_msg_to_ui(UiMsg::BpmUi(self.get_bpm()));
            // tick
            let (m, b, t, _c) = self.tg.get_tick();
            self.send_msg_to_ui(UiMsg::TickUi(self.during_play, m, b, t));
            // part
            let crnt_ = self.tg.get_crnt_msr_tick();
            for i in 0..MAX_KBD_PART {
                let part = Rc::clone(&self.part_vec[i]);
                let part_ui = part.borrow_mut().gen_part_indicator(&crnt_);
                self.send_msg_to_ui(UiMsg::PartUi(i, part_ui));
            }
            self.flac = (t % 10) as u64;
        }
    }
    pub fn get_bpm(&self) -> i16 {
        if self.during_play {
            self.tg.get_real_bpm()
        } else {
            self.bpm_stock
        }
    }
}
