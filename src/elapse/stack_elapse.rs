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

use super::tickgen::{TickGen, CrntMsrTick};
use super::midi::MidiTx;
use super::elapse::Elapse;
use super::elapse_part::Part;
use crate::lpnlib::MAX_PART_COUNT;

//  ElapseStack の責務
//  1. Elapse Object の生成と集約
//  2. Timing/Tempo の生成とtick管理
//  3. MIDI Out の生成と管理
pub struct ElapseStack {
    ui_hndr: mpsc::Sender<String>,
    mdx: MidiTx,
    start_time: Instant,
    crnt_time: Instant,
    count: u32,
    bpm_stock: u32,
    during_play: bool,
    display_time: Instant,
    tg: TickGen,
    elapse_vec: Vec<Rc<RefCell<dyn Elapse>>>,
}

impl ElapseStack {
    pub fn new(ui_hndr: mpsc::Sender<String>) -> Option<Self> {
        match MidiTx::connect() {
            Ok(c)   => {
                let mut vp = Vec::new();
                for i in 0..MAX_PART_COUNT {
                    vp.push(Part::new(i as u32))
                }
                Some(Self {
                    ui_hndr,
                    mdx: c,
                    start_time: Instant::now(),
                    crnt_time: Instant::now(),
                    count: 0,
                    bpm_stock: 120,
                    during_play: false,
                    display_time: Instant::now(),
                    tg: TickGen::new(),
                    elapse_vec: vp,
                })
            }
            Err(_e) => None,
        } 
    }
    pub fn add_elapse(&mut self, elps: Rc<RefCell<dyn Elapse>>) {
        self.elapse_vec.push(elps);
    }
    pub fn del_elapse(&mut self, search_id: u32) {
        if let Some(remove_index) = self.elapse_vec.iter().position(|x| x.borrow().id() == search_id) {
            self.elapse_vec.remove(remove_index);
        }
    }
    pub fn periodic(&mut self, msg: Result<String, TryRecvError>) -> bool {
        self.crnt_time = Instant::now();
        match msg {
            Ok(n)  => {
                if n == "quit" {return true;}
                else {self.parse_msg(n);}
            },
            Err(TryRecvError::Disconnected) => return true,// Wrong!
            Err(TryRecvError::Empty) => {},      // No event
        }

        // play 中でなければ return
        if !self.during_play {return false;}

        //  新tick計算
        let crnt_msr_tick = self.tg.get_crnt_msr_tick(self.crnt_time);
        if crnt_msr_tick.new_msr {  // 小節を跨いだ場合
            // change beat event

            // change bpm event
            if self.bpm_stock != self.tg.get_bpm() {
                self.tg.change_bpm_event(self.bpm_stock);
            }
            // fine
        }

        loop {
            // 現measure/tick より前のイベントを持つ obj を拾い出し、リストに入れて返す
            let playable = self.pick_out_playable(&crnt_msr_tick);
            if playable.len() == 0 {
                break;
            }
            // 再生 obj. をリスト順にコール（processの中で、self.elapse_vec がupdateされる可能性がある）
            for elps in playable {
                elps.borrow_mut().process(&crnt_msr_tick);
            }
        }

        // remove ended obj
        self.destroy_finished_elps();

        //  for GUI
        self.update_gui();

        return false
    }
    fn send_msg_to_ui(&self, msg: &str) {
        match self.ui_hndr.send(msg.to_string()) {
            Err(e) => println!("Something happened on MPSC! {}",e),
            _ => {},
        }
    }
    fn start(&mut self) {
        self.during_play = true;
        self.tg.start(self.crnt_time);
    }
    fn stop(&mut self) {
        self.during_play = false;
    }
    fn setting_cmnd(&mut self, msg: &str) {
        if &msg[0..4] == "bpm=" {
            self.bpm_stock = msg[4..].parse::<u32>().unwrap();
        }
    }
    fn set_phrase(&mut self, _msg: &str) {}
    fn set_composition(&mut self, _msg: &str) {}
    fn parse_msg(&mut self, msg: String) {
        println!("msg is {}", msg);
        if msg == "start" {self.start();}
        else if msg == "play" {self.start();}
        else if msg == "stop" {self.stop();}
        else if &msg[0..3] == "set" {self.setting_cmnd(&msg[3..]);}
        else if &msg[0..3] == "phr" {self.set_phrase(&msg[3..]);}
        else if &msg[0..3] == "cmp" {self.set_composition(&msg[3..]);}
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
                    for (i, one_plabl) in playable.iter().enumerate() {
                        let (msrx, tickx) = one_plabl.borrow().next();
                        if (msr < msrx) || 
                           ((msr == msrx) &&
                            ((tick < tickx) ||
                             (tick == tickx && one_plabl.borrow().prio() > elps.borrow().prio()))){
                            playable.insert(i, Rc::clone(&elps));
                            break;
                        }
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
    fn update_gui(&mut self) {
        if self.crnt_time-self.display_time > Duration::from_millis(50) {
            self.display_time = self.crnt_time;
            // tick
            let (m,b,t,_c) = self.tg.get_tick();
            let beat_disp = "3".to_owned() + &m.to_string() + " : " + &b.to_string() + " : " + &t.to_string();
            self.send_msg_to_ui(&beat_disp);
            // bpm
            let bpm_num = self.tg.get_bpm();
            let bpm_disp = "1".to_owned() + &bpm_num.to_string();
            self.send_msg_to_ui(&bpm_disp);
        }
    }
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