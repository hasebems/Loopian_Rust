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
use super::note_translation::*;

//*******************************************************************
//          Flow Struct
//*******************************************************************
//  動作イメージ
//  ・elapse object であると同時に、Part にも集約される
//  ・Part から、制御命令（生成・消滅）、MIDI Inメッセージを受け取る
//      Part は、ElapseStack から、MIDI In メッセージを受け取る
//
//  ・MIDI In は 90 nn vv / 80 nn vv のみ。nn の等しいものが対となる
//      0  -  95 : 触った位置(MIDI In)
//      0  -  71 : 対応するノート番号
//
//  ・Event Stock
//      GenStock (note:u8, vel:u8, org_locate:u8) : 実際に鳴っている原因のイベントを保持する
//      gen_stock: Vec<GenStock>
//
//  ・Event State
//      raw_state[95] : Index は触った位置。イベントがあったタイミングが記載、ないときは NO_DATA

pub const LOCATION_ALL: usize = 96;
pub const _FLOWNOTE_ALL: usize = 72;
pub const TICK_RESOLUTION: i32 = 120;

struct RawEv(i32, i32, u8, u8, u8); //  msr, tick, status, locate, vel
struct GenStock(u8, u8, u8); // note, vel, locate

pub struct Flow {
    id: ElapseId,
    priority: u32,

    old_msr_tick: CrntMsrTick,
    raw_state: [i32; LOCATION_ALL], // tickを格納 同じ場所に複数のイベントが来た場合に排除
    raw_ev: Vec<RawEv>,             // 外部からの MIDI In Ev 受信時に格納し、処理後に削除
    gen_stock: Vec<GenStock>,       // MIDI In Ev処理し、外部音源発音時に生成される

    // for super's member
    during_play: bool,
    destroy: bool,
    next_msr: i32,   //   次に呼ばれる小節番号が保持される
    next_tick: i32,  //   次に呼ばれるTick数が保持される
}

impl Flow {
    pub fn new(sid: u32, pid: u32, during_play: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {pid, sid, elps_type: ElapseType::TpFlow,},
            priority: PRI_FLOW,
            old_msr_tick: CrntMsrTick {msr:0,tick:0,tick_for_onemsr:0,},
            raw_state: [NO_DATA; LOCATION_ALL],
            raw_ev: Vec::new(),
            gen_stock: Vec::new(),

            // for super's member
            during_play,
            destroy: false,
            next_msr: FULL, // not called process()
            next_tick: 0,
        }))
    }
    /// Flow オブジェクトを消滅させ、MIDI IN による発音を終了
    pub fn deactivate(&mut self) {
        // 発音中の音をキャンセル
        self.destroy = true;
        self.during_play = false;
    }
    pub fn rcv_midi(&mut self, crnt_: &CrntMsrTick, status:u8, locate:u8, vel:u8) {
        println!("MIDI IN >> {:x}-{:x}-{:x}", status,locate,vel);
        if !self.during_play {return;}

        self.raw_ev.insert(0,RawEv(crnt_.msr,crnt_.tick,status,locate,vel));
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
    /// 考え方：
    ///  on なら、まずノート変換し、同じ音が現在鳴っていなければ発音
    ///  off なら、この音を鳴らしたイベントを locate から探し、その音を消す
    fn convert_evt(&mut self, estk: &mut ElapseStack) {
        loop {
            if let Some(ev) = self.raw_ev.pop() {
                let ch_status = ev.2 & 0xf0;
                let locate_idx = ev.3 as usize; 
                if ch_status != 0x80 && (ch_status == 0x90 && ev.4 != 0x00) {   // on
                    if self.raw_state[locate_idx] != NO_DATA {break;}
                    self.raw_state[locate_idx] = ev.1;
                    let rnote = self.detect_real_note(estk, ev.3);
                    if !self.same_note_exists(rnote) {
                        estk.midi_out(0x90, rnote, ev.4);
                        println!("MIDI OUT<< 0x90:{:x}:{:x}",rnote,ev.4);
                        self.gen_stock.push(GenStock(rnote, ev.4, ev.3));
                    }
                }
                else {      // off
                    self.raw_state[locate_idx] = NO_DATA;
                    if let Some(idx) = self.same_locate_index(ev.3) {
                        let rnote = self.gen_stock[idx].0;
                        estk.midi_out(0x90, rnote, 0); // test
                        println!("MIDI OUT<< 0x90:{:x}:0",rnote);
                        self.gen_stock.remove(idx);
                    }
                }
                self.next_msr = FULL; // process() は呼ばれないようになる
            }
            else {break;}
        }
    }
    fn detect_real_note(&mut self, estk: &mut ElapseStack, locate: u8) -> u8 {
        let mut real_note = (locate*12)/16;
        if self.id.pid/2 == 0 {real_note += 24} else {real_note += 36}
        if let Some(cmps) = estk.get_cmps(self.id.pid as usize) {
            let (rt, ctbl) = cmps.borrow().get_chord();
            let root: i16 = ROOT2NTNUM[rt as usize];
            real_note = translate_note_com(root, ctbl, real_note as i16) as u8;
        }
        real_note
    }
    fn same_note_exists(&self, rnote: u8) -> bool {
        for x in self.gen_stock.iter() {
            if x.0 == rnote {return true;}
        }
        false
    }
    fn same_locate_index(&self, locate: u8) -> Option<usize> {
        for (i, x) in self.gen_stock.iter().enumerate() {
            if x.2 == locate {return Some(i);}
        }
        None
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
            self.convert_evt(estk);
        }
        self.old_msr_tick = crnt_.clone();
    }
    /// 特定 elapse に message を送る
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    /// 自クラスが役割を終えた時に True を返す
    fn destroy_me(&self) -> bool {self.destroy}
}