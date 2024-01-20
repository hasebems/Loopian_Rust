//  Created by Hasebe Masahiko on 2023/01/22.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::{Instant, Duration};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::vec::Vec;

use crate::lpnlib::{*, ElpsMsg::*};
use super::tickgen::{TickGen, CrntMsrTick};
use super::midi::{MidiTx, MidiRx, MidiRxBuf};
use super::elapse::*;
use super::elapse_part::Part;
use super::elapse_flow::Flow;
use super::elapse_loop::{PhraseLoop, CompositionLoop};

#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum SameKeyState {
    MORE,       //  まだある
    LAST,       //  これが最後
    NOTHING,    //  もうない
}

//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
//  3. MIDI Out の生成と管理
pub struct ElapseStack {
    ui_hndr: mpsc::Sender<String>,
    mdx: MidiTx,
    _mdrx: MidiRx,
    mdr_buf: Arc<Mutex<MidiRxBuf>>,
    _start_time: Instant,
    crnt_time: Instant,
    bpm_stock: i16,
    fermata_stock: bool,
    beat_stock: Beat,

    during_play: bool,
    display_time: Instant,
    tg: TickGen,
    part_vec: Vec<Rc<RefCell<Part>>>,           // Part Instance が繋がれた Vec
    elapse_vec: Vec<Rc<RefCell<dyn Elapse>>>,   // dyn Elapse Instance が繋がれた Vec
    key_map: [i32; (MAX_NOTE_NUMBER-MIN_NOTE_NUMBER+1) as usize],

    limit_for_deb: i32,
}

impl ElapseStack {
    pub fn new(ui_hndr: mpsc::Sender<String>) -> Option<Self> {
        match MidiTx::connect() {
            Ok(c)   => {
                let mut vp = Vec::new();
                let mut velps = Vec::new();
                for i in 0..ALL_PART_COUNT {
                    // 同じ Part を part_vec, elapse_vec 両方に繋げる
                    let pt = Part::new(i as u32);
                    vp.push(Rc::clone(&pt));
                    velps.push(pt as Rc<RefCell<dyn Elapse>>);
                }
                let mut _mdrx = MidiRx::new();
                let mdr_buf = Arc::new(Mutex::new(MidiRxBuf::new()));
                match _mdrx.connect(Arc::clone(&mdr_buf)) {
                    Ok(()) => (),
                    Err(err) => println!("{}",err),
                };
                Some(Self {
                    ui_hndr,
                    mdx: c,
                    _mdrx,
                    mdr_buf,
                    _start_time: Instant::now(),
                    crnt_time: Instant::now(),
                    bpm_stock: DEFAULT_BPM,
                    fermata_stock: false,
                    beat_stock: Beat(4,4),
                    during_play: false,
                    display_time: Instant::now(),
                    tg: TickGen::new(),
                    part_vec: vp,
                    elapse_vec: velps,
                    key_map: [0; (MAX_NOTE_NUMBER-MIN_NOTE_NUMBER+1) as usize],
                    limit_for_deb: 0,
                })
            }
            Err(_e) => None,
        } 
    }
    pub fn add_elapse(&mut self, elps: Rc<RefCell<dyn Elapse>>) {
        self.elapse_vec.push(elps);
    }
    pub fn _del_elapse(&mut self, search_id: ElapseId) {    // 呼ぶとエラーが出る
        if let Some(remove_index) = self.elapse_vec.iter().position(|x| x.borrow().id() == search_id) {
            self.elapse_vec.remove(remove_index);
        }
    }
    pub fn _get_part(&mut self, id: ElapseId) -> Option<Rc<RefCell<Part>>> {
        if let Some(index) = self.part_vec.iter().position(|x| x.borrow().id() == id) {
            let part = Rc::clone(&self.part_vec[index]);
            Some(part)
        }
        else {None}
    }
    pub fn periodic(&mut self, msg: Result<ElpsMsg, TryRecvError>) -> bool {
        self.crnt_time = Instant::now();

        // message 受信処理
        match msg {
            Ok(n)  => {
                match n {
                    Ctrl(m) => {
                        if m==MSG_CTRL_QUIT {return true;}
                        else {self.parse_emsg(n)}
                    }
                    _ => self.parse_emsg(n),
                }
                //if n[0] == MSG_QUIT {return true;}
                //else {self.parse_msg(n);}
            },
            Err(TryRecvError::Disconnected) => return true, // Wrong!
            Err(TryRecvError::Empty) => {},                 // No event
        }

        //  for GUI
        self.update_gui();

        // play 中でなければ return
        if !self.during_play {return false;}

        // 小節先頭ならば、beat/bpm のイベント調査
        if self.tg.gen_tick(self.crnt_time) { 
            println!("<New measure! in stack_elapse> Max Debcnt: {}",self.limit_for_deb);
            self.limit_for_deb = 0;
            // change beat event
            if self.beat_stock != self.tg.get_beat() {
                let tick_for_onemsr = (DEFAULT_TICK_FOR_ONE_MEASURE/self.beat_stock.1)*self.beat_stock.0;
                self.tg.change_beat_event(tick_for_onemsr, self.beat_stock);
            }
            // change bpm event
            if self.bpm_stock != self.tg.get_bpm() {
                self.tg.change_bpm_event(self.bpm_stock);
            }
            if self.fermata_stock {
                self.fermata_stock = false;
                self.tg.change_fermata_event();
            }
            // for GUI(8indicator)
            self.update_gui_at_msrtop();
        }
        //  新tick計算
        let crnt_ = self.tg.get_crnt_msr_tick();

        // MIDI IN 受信処理
        if let Some(msg_ext) = self.mdr_buf.lock().unwrap().take() {
            //println!("{}: {:?} (len = {})", msg_ext.0, msg_ext.1, msg_ext.1.len());
            self.part_vec.iter().for_each(|x| {
                x.borrow_mut().rcv_midi_in(&crnt_, msg_ext.1[0], msg_ext.1[1], msg_ext.1[2]);
            });
        }

        let mut debcnt = 0;
        loop {
            // 現measure/tick より前のイベントを持つ obj を拾い出し、リストに入れて返す
            let playable = self.pick_out_playable(&crnt_);
            if playable.len() == 0 {
                break;
            }
            else {
                //println!("$$$deb:{},{},{},{:?}",limit_for_deb,crnt_.msr,crnt_.tick,self.crnt_time);
                assert!(debcnt < 100);
                debcnt += 1;
            }
            // 再生 obj. をリスト順にコール（processの中で、self.elapse_vec がupdateされる可能性がある）
            for elps in playable {
                elps.borrow_mut().process(&crnt_, self);
            }
        }
        if self.limit_for_deb < debcnt {
            self.limit_for_deb = debcnt;
        } 

        // remove ended obj
        self.destroy_finished_elps();

        return false
    }
    fn parse_emsg(&mut self, msg: ElpsMsg) {
        match msg {
            Ctrl(m) => self.ctrl_msg(m),
            Sync(m) => self.sync(m),
            Rit(m) => self.rit(m),
            Set(m) => self.setting_cmnd(m),
            SetBeat(m) => self.set_beat(m),
            Phr(m0, m1, m2, mv) => self.phrase(m0, m1, m2, mv),
            Cmp(m0, m1, mv) => self.composition(m0, m1, mv),
            Ana(m0, m1, mv) => self.ana(m0, m1, mv),
            PhrX(m0, m1) => self.del_phrase(m0, m1),
            CmpX(m) => self.del_composition(m),
            AnaX(m0, m1) => self.del_ana(m0, m1),
            //_ => {}
        }
    }
    fn ctrl_msg(&mut self, msg: i16) {
        if msg == MSG_CTRL_START {self.start(false);}
        else if msg == MSG_CTRL_STOP {self.stop();}
        else if msg == MSG_CTRL_PANIC {self.panic();}
        else if msg == MSG_CTRL_RESUME {self.start(true);}
        else if msg >= MSG_CTRL_FLOW && msg < MSG_CTRL_FLOW+5 {self.flow(vec![msg-MSG_CTRL_FLOW]);}
        else if msg >= MSG_CTRL_ENDFLOW && msg < MSG_CTRL_ENDFLOW+5 {self.endflow(vec![msg-MSG_CTRL_ENDFLOW]);}
    }
    pub fn midi_out(&mut self, status: u8, data1: u8, data2: u8) {
        self.mdx.midi_out(status, data1, data2);
    }
    pub fn get_phr(&self, part_num: usize) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.part_vec[part_num].borrow().get_phr()
    }
    pub fn get_cmps(&self, part_num: usize) -> Option<Rc<RefCell<CompositionLoop>>> {
        self.part_vec[part_num].borrow().get_cmps()
    }
    pub fn get_flow(&self, part_num: usize) -> Option<Rc<RefCell<Flow>>> {
        self.part_vec[part_num].borrow().get_flow()
    }
    pub fn tg(&self) -> &TickGen {&self.tg}
    pub fn inc_key_map(&mut self, key_num: u8, vel: u8) {
        self.key_map[(key_num-MIN_NOTE_NUMBER) as usize] += 1;
        let key_disp = format!("9{}/{}", key_num, vel);
        self.send_msg_to_ui(&key_disp);
    }
    pub fn dec_key_map(&mut self, key_num: u8) -> SameKeyState {
        let idx = (key_num-MIN_NOTE_NUMBER) as usize;
        if self.key_map[idx] > 1 {
            self.key_map[idx] -= 1;
            SameKeyState::MORE
        }
        else if self.key_map[idx] == 1 {
            self.key_map[idx] = 0;
            SameKeyState::LAST
        }
        else {SameKeyState::NOTHING}
    }
    pub fn set_phrase_vari(&self, part_num: usize, vari_num: usize) {
        self.part_vec[part_num].borrow_mut().set_phrase_vari(vari_num);
    }
    fn send_msg_to_ui(&self, msg: &str) {
        match self.ui_hndr.send(msg.to_string()) {
            Err(e) => println!("Something happened on MPSC for UI! {}",e),
            _ => {},
        }
    }
    fn start(&mut self, resume: bool) {
        if self.during_play && !resume {return;}
        self.during_play = true;
        self.tg.start(self.crnt_time, self.bpm_stock, resume);
        for elps in self.elapse_vec.iter() {
            elps.borrow_mut().start();
        }
        if let Ok(mut mb) = self.mdr_buf.lock() {
            mb.flush(); // MIDI In Buffer をクリア
        }
    }
    fn panic(&mut self) {
        self.midi_out(0xb0, 0x78, 0x00);
    }
    fn stop(&mut self) {
        if !self.during_play {return;}
        self.during_play = false;
        let stop_vec = self.elapse_vec.to_vec();
        for elps in stop_vec.iter() {
            elps.borrow_mut().stop(self);
        }
    }
    //fn fermata(&mut self, _msg: Vec<i16>) {self.fermata_stock = true;}
    fn sync(&mut self, part: i16) {
        let mut sync_part = [false; MAX_USER_PART];
        if part < MAX_USER_PART as i16 {sync_part[part as usize] = true;}
        else if part == MSG_SYNC_LFT {
            sync_part[LEFT1] = true;
            sync_part[LEFT2] = true;
        }
        else if part == MSG_SYNC_RGT {
            sync_part[RIGHT1] = true;
            sync_part[RIGHT2] = true;
        }
        else if part == MSG_SYNC_ALL {
            for pt in sync_part.iter_mut() {*pt=true;}
        }
        for (i, pt) in sync_part.iter().enumerate() {
            if *pt {self.part_vec[i].borrow_mut().set_sync();}
        }
    }
    fn flow(&mut self, msg: Vec<i16>) {
        let ptnum = msg[1] as usize;
        if ptnum < MAX_USER_PART {
            let pt = self.part_vec[ptnum].clone();
            pt.borrow_mut().activate_flow(self);
        }
    }
    fn endflow(&mut self, msg: Vec<i16>) {
        let ptnum = msg[1] as usize;
        if ptnum < MAX_USER_PART {
            self.part_vec[ptnum].borrow_mut().deactivate_flow();
        }
    }
    fn rit(&mut self, msg: [i16; 2]) {
        let strength_set: [(i16, i32);3] = [(MSG_RIT_POCO, 95),(MSG_RIT_NRM, 80),(MSG_RIT_MLT, 75)];
        let strength_msg =  msg[0]%10;
        let bar = (msg[0]/10) as i32;
        let strength = strength_set.into_iter()
            .find(|x| x.0==strength_msg)
            .unwrap_or(strength_set[0]);
        if msg[1] == MSG2_RIT_ATMP {self.bpm_stock = self.tg.get_bpm();}
        else if msg[1] == MSG2_RIT_FERMATA {self.fermata_stock = true;}
        else {
            self.bpm_stock = msg[1];
        }
        self.tg.start_rit(self.crnt_time, strength.1, bar);
    }
    fn setting_cmnd(&mut self, msg: [i16; 2]) {
        if msg[0] == MSG_SET_BPM {
            self.bpm_stock = msg[1];
        }
        else if msg[0] == MSG_SET_KEY {
            self.part_vec.iter().for_each(|x| x.borrow_mut().change_key(msg[1] as u8));
        }
        else if msg[0] == MSG_SET_TURN {
            self.part_vec.iter_mut().for_each(|x| x.borrow_mut().set_turnnote(msg[1]));
        }
    }
    fn set_beat(&mut self, msg: [i16; 2]) {
        self.beat_stock = Beat(msg[0] as i32, msg[1] as i32);
        self.sync(MSG_SYNC_ALL);
    }
    fn phrase(&mut self, part_num: i16, vari_num: i16, tick: i16, evts: Vec<PhrEvt>) {
        println!("Received Phrase Message! Part: {}, variation: {}", part_num, vari_num);
        self.part_vec[part_num as usize].borrow_mut().rcv_phr_msg(evts, tick, vari_num as usize);
    }
    fn composition(&mut self, part_num: i16, tick: i16, evts: Vec<ChordEvt>) {
        println!("Received Composition Message! Part: {}", part_num);
        self.part_vec[part_num as usize].borrow_mut().rcv_cmps_msg(evts, tick);
    }
    fn ana(&mut self, part_num: i16, vari_num: i16, evts: Vec<AnaEvt>) {
        println!("Received Analysis Message! Part: {}, variation: {}", part_num, vari_num);
        self.part_vec[part_num as usize].borrow_mut().rcv_ana_msg(evts, vari_num as usize);
    }
    fn del_phrase(&mut self, part_num: i16, vari_num: i16) {
        self.part_vec[part_num as usize].borrow_mut().rcv_phr_msg(Vec::new(), 0, vari_num as usize);
        self.part_vec[part_num as usize].borrow_mut().rcv_ana_msg(Vec::new(), vari_num as usize);
    }
    fn del_composition(&mut self, part_num: i16) {
        self.part_vec[part_num as usize].borrow_mut().rcv_cmps_msg(Vec::new(), 0);
    }
    fn del_ana(&mut self, part_num: i16, vari_num: i16) {
        self.part_vec[part_num as usize].borrow_mut().rcv_ana_msg(Vec::new(), vari_num as usize);
    }
    fn pick_out_playable(&self, crnt_: &CrntMsrTick) -> Vec<Rc<RefCell<dyn Elapse>>> {
        let mut playable: Vec<Rc<RefCell<dyn Elapse>>> = Vec::new();
        for elps in self.elapse_vec.iter() {
            let (msr, tick) = elps.borrow().next();
            if (msr == crnt_.msr && tick <= crnt_.tick) || msr < crnt_.msr {
                // 現在のタイミングより前のイベントがあれば
                if playable.len() == 0 {
                    // playable にまだ何も無ければ、普通に push
                    playable.push(Rc::clone(&elps));
                }
                else {
                    // playable に、時間順になるように挿入
                    let mut after_break = false; 
                    for (i, one_plabl) in playable.iter().enumerate() {
                        let (msrx, tickx) = one_plabl.borrow().next();
                        if (msr < msrx) || 
                           ((msr == msrx) &&
                            ((tick < tickx) ||
                             ((tick == tickx) && (one_plabl.borrow().prio() > elps.borrow().prio())))){
                            playable.insert(i, Rc::clone(&elps));
                            after_break = true;
                            break;
                        }
                    }
                    if !after_break { // 条件にはまらなければ最後に入れる
                        playable.push(Rc::clone(&elps));
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
            if removed_num == -1 {break;}
        }
    }
    fn update_gui_at_msrtop(&mut self) {
        // key
        let key_disp = "0_".to_string();
        self.send_msg_to_ui(&key_disp);
        // beat
        let beat = self.tg.get_beat();
        let beat_disp = format!("2{}/{}",beat.0,beat.1);
        self.send_msg_to_ui(&beat_disp);
    }
    fn update_gui(&mut self) {
        if self.crnt_time-self.display_time > Duration::from_millis(80) {
            self.display_time = self.crnt_time;
            // bpm
            let bpm_num = self.tg.get_real_bpm();
            let bpm_disp = format!("1{}",bpm_num);
            self.send_msg_to_ui(&bpm_disp);
            // tick
            let (m,b,t,_c) = self.tg.get_tick();
            let tick_disp = format!("3{} : {} : {:>03}",m,b,t);
            self.send_msg_to_ui(&tick_disp);
            // part
            self.part_vec.iter().for_each(|x| {
                let crnt_ = self.tg.get_crnt_msr_tick();
                let part_ind = x.borrow().gen_part_indicator(&crnt_);
                self.send_msg_to_ui(&part_ind);
            });
        }
    }
}