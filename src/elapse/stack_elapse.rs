//  Created by Hasebe Masahiko on 2023/01/22.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::{Instant, Duration};
use std::rc::Rc;
use std::cell::RefCell;
use std::vec::Vec;

use crate::lpnlib::*;
use super::tickgen::{TickGen, CrntMsrTick};
use super::midi::MidiTx;
use super::elapse::*;
use super::elapse_part::Part;
use super::elapse_loop::{PhraseLoop, CompositionLoop};

//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
//  3. MIDI Out の生成と管理
pub struct ElapseStack {
    ui_hndr: mpsc::Sender<String>,
    mdx: MidiTx,
    _start_time: Instant,
    crnt_time: Instant,
    _count: u32,
    bpm_stock: i16,
    beat_stock: Beat,
    during_play: bool,
    display_time: Instant,
    tg: TickGen,
    part_vec: Vec<Rc<RefCell<Part>>>,           // Part Instance が繋がれた Vec
    elapse_vec: Vec<Rc<RefCell<dyn Elapse>>>,   // dyn Elapse Instance が繋がれた Vec
    registered_cmnd: Vec<(ElapseMsg, u8, ElapseId)>,
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
                Some(Self {
                    ui_hndr,
                    mdx: c,
                    _start_time: Instant::now(),
                    crnt_time: Instant::now(),
                    _count: 0,
                    bpm_stock: DEFAULT_BPM,
                    beat_stock: Beat(4,4),
                    during_play: false,
                    display_time: Instant::now(),
                    tg: TickGen::new(),
                    part_vec: vp,
                    elapse_vec: velps,
                    registered_cmnd: Vec::new(),
                })
            }
            Err(_e) => None,
        } 
    }
    pub fn add_elapse(&mut self, elps: Rc<RefCell<dyn Elapse>>) {
        self.elapse_vec.push(elps);
    }
    pub fn del_elapse(&mut self, search_id: ElapseId) {
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
    pub fn register_sp_cmnd(&mut self, msg: ElapseMsg, dt: u8, id: ElapseId) {
        self.registered_cmnd.push((msg, dt, id));
    }
    pub fn periodic(&mut self, msg: Result<Vec<i16>, TryRecvError>) -> bool {
        self.crnt_time = Instant::now();
        match msg {
            Ok(n)  => {
                if n[0] == MSG_QUIT {return true;}
                else {self.parse_msg(n);}
            },
            Err(TryRecvError::Disconnected) => return true,// Wrong!
            Err(TryRecvError::Empty) => {},      // No event
        }

        // play 中でなければ return
        if !self.during_play {return false;}

        // 小節先頭ならば、beat/bpm のイベント調査
        if self.tg.new_msr(self.crnt_time) { 
            println!("<New measure! in stack_elapse>");
            // change beat event
            if self.beat_stock != self.tg.get_beat() {
                let tick_for_onemsr = (DEFAULT_TICK_FOR_ONE_MEASURE/self.beat_stock.1)*self.beat_stock.0;
                self.tg.change_beat_event(tick_for_onemsr, self.beat_stock);
            }
            // change bpm event
            if self.bpm_stock != self.tg.get_bpm() {
                self.tg.change_bpm_event(self.bpm_stock);
            }
            // fine //<<DoItLater>>
        }
        //  新tick計算
        let crnt_ = self.tg.get_crnt_msr_tick();

        loop {
            // 現measure/tick より前のイベントを持つ obj を拾い出し、リストに入れて返す
            let playable = self.pick_out_playable(&crnt_);
            if playable.len() == 0 {
                break;
            }
            // 再生 obj. をリスト順にコール（processの中で、self.elapse_vec がupdateされる可能性がある）
            for elps in playable {
                elps.borrow_mut().process(&crnt_, self);
            }
        }

        // registered sp command
        self.call_registered_cmnd();      

        // remove ended obj
        self.destroy_finished_elps();

        //  for GUI
        self.update_gui();

        return false
    }
    pub fn midi_out(&mut self, status: u8, data1: u8, data2: u8) {
        self.mdx.midi_out(status, data1, data2);
        /*let et = crnt_time-self.start_time;
        if et > Duration::from_secs(1) {
            self.start_time = crnt_time;
            self.count += 1;
            if self.count%2 == 1 {
                self.mdx.midi_out(0x90,0x40,0x60);
                self.send_msg_to_ui(&self.count.to_string());
            }
            else {
                self.mdx.midi_out(0x80,0x40,0x40);
            }
        }*/
    }
    pub fn get_phr(&self, part_num: usize) -> Option<Rc<RefCell<PhraseLoop>>> {
        self.part_vec[part_num].borrow().get_phr()
    }
    pub fn get_cmps(&self, part_num: usize) -> Option<Rc<RefCell<CompositionLoop>>> {
        self.part_vec[part_num].borrow().get_cmps()
    }
    pub fn tg(&self) -> &TickGen {&self.tg}
    fn send_msg_to_ui(&self, msg: &str) {
        match self.ui_hndr.send(msg.to_string()) {
            Err(e) => println!("Something happened on MPSC! {}",e),
            _ => {},
        }
    }
    fn start(&mut self) {
        self.during_play = true;
        self.tg.start(self.crnt_time);
        for elps in self.elapse_vec.iter() {
            elps.borrow_mut().start();
        }
    }
    fn stop(&mut self) {
        self.during_play = false;
        let stop_vec = self.elapse_vec.to_vec();
        for elps in stop_vec.iter() {
            elps.borrow_mut().stop(self);
        }
    }
    fn setting_cmnd(&mut self, msg: Vec<i16>) {
        if msg[1] == MSG2_BPM {
            self.bpm_stock = msg[2];
        }
        else if msg[1] == MSG2_BEAT {
            self.beat_stock = Beat(msg[2] as i32, msg[3] as i32);
        }
        else if msg[1] == MSG2_KEY {
            self.part_vec.iter().for_each(|x| x.borrow_mut().change_key(msg[2] as u8));
        }
    }
    fn phrase(&mut self, msg: Vec<i16>) {
        // message の２次元化
        let part_num: usize = pt(msg[0]) as usize;
        let whole_tick: i16 = msg[1];
        let mut phr_vec: Vec<Vec<i16>> = Vec::new();
        let mut msg_cnt: usize = 0;
        let msg_size = msg.len();
        loop {
            let index = |x, cnt| {x+MSG_HEADER+cnt*TYPE_NOTE_SIZE};
            let mut vtmp: Vec<i16> = Vec::new();
            for i in 0..TYPE_NOTE_SIZE {
                vtmp.push(msg[index(i,msg_cnt)]);
            }
            phr_vec.push(vtmp);
            msg_cnt += 1;
            if msg_size <= index(0,msg_cnt) {break;}
        }
        self.part_vec[part_num].borrow_mut().rcv_phr_msg(phr_vec, whole_tick);
    }
    fn composition(&mut self, msg: Vec<i16>) {
        // message の２次元化
        let part_num: usize = pt(msg[0]) as usize;
        let whole_tick: i16 = msg[1];
        let mut cmps_vec: Vec<Vec<i16>> = Vec::new();
        let mut msg_cnt: usize = 0;
        let msg_size = msg.len();
        loop {
            let index = |x, cnt| {x+MSG_HEADER+cnt*TYPE_CHORD_SIZE};
            let mut vtmp: Vec<i16> = Vec::new();
            for i in 0..TYPE_CHORD_SIZE {
                vtmp.push(msg[index(i,msg_cnt)]);
            }
            cmps_vec.push(vtmp);
            msg_cnt += 1;
            if msg_size <= index(0,msg_cnt) {break;}
        }
        self.part_vec[part_num].borrow_mut().rcv_cmps_msg(cmps_vec, whole_tick);
    }
    fn ana(&mut self, msg: Vec<i16>) {
        // message の２次元化
        let part_num: usize = pt(msg[0]) as usize;
        let mut ana_vec: Vec<Vec<i16>> = Vec::new();
        let mut msg_cnt: usize = 0;
        let msg_size = msg.len();
        loop {
            let index = |x, cnt| {x+MSG_HEADER+cnt*TYPE_BEAT_SIZE};
            let mut vtmp: Vec<i16> = Vec::new();
            for i in 0..TYPE_BEAT_SIZE {
                vtmp.push(msg[index(i,msg_cnt)]);
            }
            ana_vec.push(vtmp);
            msg_cnt += 1;
            if msg_size <= index(0,msg_cnt) {break;}
        }
        self.part_vec[part_num].borrow_mut().rcv_ana_msg(ana_vec);
    }
    fn del_phrase(&mut self, msg: Vec<i16>) {
        let part_num: usize = pt(msg[0]) as usize;
        self.part_vec[part_num].borrow_mut().rcv_phr_msg(Vec::new(), 0);
        self.part_vec[part_num].borrow_mut().rcv_ana_msg(Vec::new());
    }
    fn del_composition(&mut self, msg: Vec<i16>) {
        let part_num: usize = pt(msg[0]) as usize;
        self.part_vec[part_num].borrow_mut().rcv_cmps_msg(Vec::new(), 0);
    }
    fn del_ana(&mut self, msg: Vec<i16>) {
        let part_num: usize = pt(msg[0]) as usize;
        self.part_vec[part_num].borrow_mut().rcv_ana_msg(Vec::new());
    }
    fn parse_msg(&mut self, msg: Vec<i16>) {
        println!("msg is {:?}", msg[0]);
        if msg[0] == MSG_START {self.start();}
        else if msg[0] == MSG_START {self.start();}
        else if msg[0] == MSG_STOP {self.stop();}
        else if msg[0] == MSG_SET {self.setting_cmnd(msg);}
        else if msg1st(msg[0]) == MSG_PHR_X {self.del_phrase(msg);}
        else if msg1st(msg[0]) == MSG_CMP_X {self.del_composition(msg);}
        else if msg1st(msg[0]) == MSG_ANA_X {self.del_ana(msg);}
        else if msg1st(msg[0]) == MSG_PHR {self.phrase(msg);}
        else if msg1st(msg[0]) == MSG_CMP {self.composition(msg);}
        else if msg1st(msg[0]) == MSG_ANA {self.ana(msg);}
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
    fn call_registered_cmnd(&mut self) {
        for reg in &self.registered_cmnd {
            //println!("registered cmnd!");
            let (msg, dt, rmv_id) = *reg;
            for elps in self.elapse_vec.iter() {
                if elps.borrow().id() != rmv_id {
                    elps.borrow_mut().rcv_sp(msg, dt);
                }
            }
        }
        self.registered_cmnd = Vec::new(); // Initialize
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
    fn update_gui(&mut self) {
        if self.crnt_time-self.display_time > Duration::from_millis(20) {
            self.display_time = self.crnt_time;
            // bpm
            let bpm_num = self.tg.get_bpm();
            let bpm_disp = format!("1{}",bpm_num);
            self.send_msg_to_ui(&bpm_disp);
            // beat
            let beat = self.tg.get_beat();
            let bpm_disp = format!("2{}/{}",beat.0,beat.1);
            self.send_msg_to_ui(&bpm_disp);
            // tick
            let (m,b,t,_c) = self.tg.get_tick();
            let beat_disp = format!("3{} : {} : {:>03}",m,b,t);
            self.send_msg_to_ui(&beat_disp);
            //<<DoItLater>> その他の表示
        }
    }
}