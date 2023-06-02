//  Created by Hasebe Masahiko on 2023/05/18.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::rc::Rc;
use std::cell::RefCell;

use crate::lpnlib::*;
use super::elapse::*;
use super::tickgen::CrntMsrTick;
use super::stack_elapse::ElapseStack;

//*******************************************************************
//          Flow Struct
//*******************************************************************
//  動作イメージ
//  ・elapse object であると同時に、Part にも集約される
//  ・Part から、制御命令（生成・消滅）、MIDI Inメッセージを受け取る
//      Part は、ElapseStack から、MIDI In メッセージを受け取る
//
//  ・MIDI In は 90 nn vv / 80 nn vv のみ。nn vv の等しいものが対となる。nn: 0 - 95
//      0  -  95 : 触った位置(MIDI In)
//      0  -  71 : 対応するノート番号
//  ・Event Flow
//      raw_state[95] : Index は触った位置。イベントがあったタイミングが記載、ないときは NO_DATA
//      gen_state[71] : Index はノート番号。
//                   tick=120 の周期で、raw_ev に値がある位置より、Note を算出し、タイミングを記載。
//                   生成後、過去の gen_state[] の値と比較し、NoteOn されていなければ MIDI OUT 出力する

pub const LOCATION_ALL: usize = 96;
pub const _FLOWNOTE_ALL: usize = 72;

pub const TICK_RESOLUTION: i32 = 120;

pub struct Flow {
    id: ElapseId,
    priority: u32,

    during_play: bool,
    old_msr_tick: CrntMsrTick,
    raw_state: [i32; LOCATION_ALL],    // tick
    //gen_state: [i32; FLOWNOTE_ALL],    // tick
    raw_ev: Vec<(i32, i32, bool, u8, u8)>, // msr, tick, on/off, note, vel

    // for super's member
    //whole_tick: i32,
    destroy: bool,
    _first_msr_num: i32,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
}

impl Flow {
    pub fn new(sid: u32, pid: u32, msr: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpFlow,},
            priority: PRI_FLOW,

            during_play: false,
            old_msr_tick: CrntMsrTick {msr:0,tick:0,tick_for_onemsr:0,},
            raw_state: [NO_DATA; LOCATION_ALL],
            //gen_state: [NO_DATA; FLOWNOTE_ALL],
            raw_ev: Vec::new(),

            // for super's member
            //whole_tick: 0,
            destroy: false,
            _first_msr_num: msr,
            next_msr: FULL, // not called process()
            next_tick: 0,
        }))
    }
    /// Flow オブジェクトを消滅させ、MIDI IN による発音を終了
    pub fn deactivate(&mut self) {
        // 発音中の音をキャンセル
        self.destroy = true;
    }
    pub fn rcv_midi(&mut self, crnt_: &CrntMsrTick, note_on:bool, locate:u8, vel:u8) {
        println!("MIDI IN: {}-{}-{}", if note_on{"On"}else{"Of"},locate,vel);
        if !self.during_play {return;}

        self.raw_ev.insert(0,(crnt_.msr,crnt_.tick,note_on,locate,vel));

        let tk = (crnt_.tick/TICK_RESOLUTION + 1)*TICK_RESOLUTION;
        if tk >= crnt_.tick_for_onemsr {
            self.next_msr = crnt_.msr + 1;
            self.next_tick = tk - crnt_.tick_for_onemsr;
        }
        else {
            self.next_msr = crnt_.msr;
            self.next_tick = tk;
        }
    }
    fn read_midi_evt(&mut self, estk: &mut ElapseStack) {
        loop {
            if let Some(ev) = self.raw_ev.pop() {
                if ev.2 {
                    self.raw_state[ev.3 as usize] = ev.1;
                    estk.midi_out(0x90, ev.3, ev.4); //test
                }
                else {
                    self.raw_state[ev.3 as usize] = NO_DATA;
                    estk.midi_out(0x90, ev.3, 0); // test
                }
                self.next_msr = FULL; // process() は呼ばれないようになる
            }
            else {break;}
        }


        //self.raw_state[locate as usize] = if note_on {crnt_.tick} else {END_OF_DATA};
    }
}

impl Elapse for Flow {
    /// id を得る
    fn id(&self) -> ElapseId {self.id}
    /// priority を得る
    fn prio(&self) -> u32 {self.priority}
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32) {(self.next_msr, self.next_tick)}
    /// User による start/play 時にコールされる
    fn start(&mut self) {self.during_play = true;}
    /// User による stop 時にコールされる
    fn stop(&mut self, _estk: &mut ElapseStack) {self.during_play = false;}
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if (crnt_.msr == self.next_msr &&
            crnt_.tick/TICK_RESOLUTION == self.next_tick/TICK_RESOLUTION) ||
           (crnt_.msr == self.next_msr+1){
            self.read_midi_evt(estk);
        }
        self.old_msr_tick = crnt_.clone();
    }
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {self.destroy}
}