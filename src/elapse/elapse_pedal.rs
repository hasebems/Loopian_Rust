//  Created by Hasebe Masahiko on 2025/09/28
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cell::RefCell;
use std::rc::Rc;

use super::elapse_base::*;
use super::floating_tick::*;
use super::stack_elapse::ElapseStack;
use super::tickgen::CrntMsrTick;
use crate::lpnlib::*;

//*******************************************************************
//          Damper Event Struct
//*******************************************************************
pub struct Damper {
    id: ElapseId,
    priority: u32,
    position: u8,
    damper_started: bool,
    destroy: bool,
    next_msr: i32,
    next_tick: i32,
}
impl Damper {
    pub fn new(sid: u32, pid: u32, position: u8, msr: i32, tick: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpNote,
            },
            priority: PRI_NOTE,
            position,
            damper_started: false,
            destroy: false,
            next_msr: msr,
            next_tick: tick,
        }))
    }
    fn damper_off(&mut self, estk: &mut ElapseStack) {
        self.destroy = true;
        self.next_msr = FULL;
        // midi damper off
        estk.midi_out(0xb0, 0x40, 0);
        #[cfg(feature = "verbose")]
        println!("Damper-Off");
    }
    fn damper_evt(&mut self, estk: &mut ElapseStack) {
        self.destroy = true;
        self.next_msr = FULL;
        let pos = if self.position > 127 {
            127
        } else {
            self.position
        };
        estk.midi_out(0xb0, 0x40, pos);
        #[cfg(feature = "verbose")]
        println!("Damper-Event: {}", self.position);
    }
}
impl Elapse for Damper {
    /// id を得る
    fn id(&self) -> ElapseId {
        self.id
    }
    /// priority を得る
    fn prio(&self) -> u32 {
        self.priority
    }
    /// 次に呼ばれる小節番号、Tick数を返す
    fn next(&self) -> (i32, i32, bool) {
        (self.next_msr, self.next_tick, false)
    }
    /// User による start/play 時にコールされる
    fn start(&mut self, _msr: i32) {}
    /// User による stop 時にコールされる
    fn stop(&mut self, estk: &mut ElapseStack) {
        if self.damper_started {
            self.damper_off(estk);
        }
    }
    /// 再生データを消去
    fn clear(&mut self, estk: &mut ElapseStack) {
        if self.damper_started {
            self.damper_off(estk);
        }
        self.destroy = true;
    }
    /// 再生 msr/tick に達したらコールされる
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if (crnt_.msr == self.next_msr && crnt_.tick >= self.next_tick)
            || (crnt_.msr > self.next_msr)
        {
            self.damper_evt(estk);
        }
    }
    fn rcv_sp(&mut self, msg: ElapseMsg, _msg_data: u8) {
        let _ = msg;
    }
    fn destroy_me(&self) -> bool {
        self.destroy
    } // 自クラスが役割を終えた時に True を返す
}

//*******************************************************************
//          Pedal Loop Struct
//*******************************************************************
pub struct PedalLoop {
    id: ElapseId,
    priority: u32,
    during_play: bool,
    start_flag: bool,

    // Pedal Message を受けた場合
    damper_msg: Vec<PedalEvt>,
    tick_for_onebeat: i32,
    play_counter: usize,
    next_tick_in_phrase: i32,
    flt: FloatingTick, //  FloatingTick を保持する

    // for super's member
    whole_tick: i32,
    destroy: bool,
    first_msr_num: i32,
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl PedalLoop {
    pub fn new(sid: u32, pid: u32, msg: &PhrData) -> Self {
        let damper_msg = msg
            .evts
            .iter()
            .filter_map(|e| {
                if let PhrEvt::Damper(p) = e {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect();
        Self {
            id: ElapseId {
                pid,
                sid,
                elps_type: ElapseType::TpPedalLoop,
            },
            priority: PRI_PHR_LOOP,
            during_play: false,
            start_flag: false,
            damper_msg,
            tick_for_onebeat: 0,
            play_counter: 0,
            next_tick_in_phrase: 0,
            flt: FloatingTick::new(false),
            whole_tick: msg.whole_tick as i32,
            destroy: false,
            first_msr_num: 0,
            next_msr: 0,
            next_tick: 0,
        }
    }
    fn play_damper_msg(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.destroy {
            return;
        }

        // crnt_ を記譜上の小節数に変換し、その後 elapsed_tick を計算する
        let ntcrnt_ = self.flt.convert_to_notational(crnt_);

        let elapsed_tick = self.calc_serial_tick(&ntcrnt_);
        if elapsed_tick > self.whole_tick {
            self.next_msr = FULL;
            self.destroy = true;
        } else if elapsed_tick >= self.next_tick_in_phrase {
            let next_tick = self.generate_damper_event(crnt_, estk);
            self.next_tick_in_phrase = next_tick;
            if next_tick == END_OF_DATA {
                self.next_msr = FULL;
                self.destroy = true;
            } else {
                let (msr, tick) = self.gen_msr_tick(&ntcrnt_, self.next_tick_in_phrase);
                let mt = CrntMsrTick {
                    msr,
                    tick,
                    tick_for_onemsr: ntcrnt_.tick_for_onemsr,
                    ..Default::default()
                };
                // FloatingTick を使って、次に呼ばれる実際の小節とTickを計算する
                if let Some(rlcrnt_) = self.flt.convert_to_real(&mt) {
                    // FloatingTick の変換結果を使って、次の小節とTickを設定する
                    self.next_msr = rlcrnt_.msr;
                    self.next_tick = rlcrnt_.tick;
                } else {
                    // FloatingTick の変換結果が None の場合は、現在の crnt_ をそのまま使用
                    self.next_msr = mt.msr;
                    self.next_tick = mt.tick;
                }
            }
        }
    }
    fn generate_damper_event(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) -> i32 {
        let evt = &self.damper_msg[self.play_counter];
        let val = Self::convert_pos_to_value(evt.position);
        let dmpr: Rc<RefCell<dyn Elapse>> = Damper::new(
            self.play_counter as u32, //  msr&read pointer
            self.id.sid,              //  pedal_loop.sid -> damper.pid
            val,                      //&evt[trace],
            self.next_msr,
            self.real_tick(evt) as i32,
        );
        estk.add_elapse(Rc::clone(&dmpr));

        self.play_counter += 1;
        if self.play_counter < self.damper_msg.len() {
            let evt = &self.damper_msg[self.play_counter];
            evt.msr as i32 * crnt_.tick_for_onemsr + evt.beat as i32 * self.tick_for_onebeat
        } else {
            END_OF_DATA
        }
    }
    fn real_tick(&self, evt: &PedalEvt) -> i16 {
        if evt.front {
            evt.beat * self.tick_for_onebeat as i16 + 30
        } else {
            evt.beat * self.tick_for_onebeat as i16 + (self.tick_for_onebeat as i16 - 30)
        }
    }
    /// Pedal Part に PhrData メッセージを受信する
    /*pub fn rcv_phr_msg(&mut self, msg: PhrData, _crnt_: &CrntMsrTick, _estk_: &mut ElapseStack) {
        let evts = msg.evts;
        let mut damper_flag: bool = false;
        let mut sostenuto_flag: bool = false;
        let mut shift_flag: bool = false;
        for e in evts {
            match e {
                PhrEvt::Damper(p) => {
                    if !damper_flag {
                        self.damper_msg.clear();
                        damper_flag = true;
                    }
                    self.damper_msg.push(p);
                }
                PhrEvt::Sostenuto(_p) => {
                    if !sostenuto_flag {
                        sostenuto_flag = true;
                    }
                }
                PhrEvt::Shift(_p) => {
                    if !shift_flag {
                        shift_flag = true;
                    }
                }
                _ => {
                    // ignore other events
                }
            }
        }
        if damper_flag {
            // Damper 再生の初期化
            println!("PedalPart: Damper PhrData received, {:?}", self.damper_msg);
        }
    }*/
    fn convert_pos_to_value(pos: PedalPos) -> u8 {
        match pos {
            PedalPos::NoEvt => 0,
            PedalPos::Off => 0,
            PedalPos::Half => 64,
            PedalPos::Full => 127,
        }
    }
}
impl Elapse for PedalLoop {
    fn id(&self) -> ElapseId {
        self.id
    } // id を得る
    fn prio(&self) -> u32 {
        self.priority
    } // priority を得る
    fn next(&self) -> (i32, i32, bool) {
        // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick, false)
    }
    fn start(&mut self, msr: i32) {
        // User による start/play 時にコールされる
        self.during_play = true;
        self.start_flag = true;
        self.next_msr = msr;
        self.first_msr_num = msr;
        self.next_tick = 0;
    }
    fn stop(&mut self, _estk: &mut ElapseStack) {
        // User による stop 時にコールされる
        self.during_play = false;
        self.destroy = true;
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.damper_msg = Vec::new();
        self.next_msr = 0;
        self.next_tick = 0;
    }
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        // Damper PhrData メッセージがある場合の処理
        if self.play_counter == 0 {
            self.tick_for_onebeat = estk.tg().get_beat_tick().1;
            self.first_msr_num = crnt_.msr;
        }
        self.play_damper_msg(crnt_, estk);
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {
        // 自クラスが役割を終えた時に True を返す
        self.destroy
    }
}
impl Loop for PedalLoop {
    fn destroy(&self) -> bool {
        self.destroy
    }
    fn set_destroy(&mut self) {
        self.destroy = true;
    }
    fn first_msr_num(&self) -> i32 {
        self.first_msr_num
    }
    /// Loopの途中から再生するための小節数を設定
    fn set_forward(&mut self, crnt_: &CrntMsrTick, elapsed_msr: i32) {
        self.first_msr_num = crnt_.msr - elapsed_msr;
    }
}

//*******************************************************************
//          Pedal Part Struct
//*******************************************************************
pub struct PedalPart {
    id: ElapseId,
    priority: u32,
    during_play: bool,
    start_flag: bool,
    position: i16,
    pedal_msg: Option<PhrData>,
    pedal_loop: Option<Rc<RefCell<PedalLoop>>>,
    elapsed_tick: i32,
    loop_whole_tick: i32,
    do_loop: bool,
    sync: bool,

    // for super's member
    next_msr: i32,  //   次に呼ばれる小節番号が保持される
    next_tick: i32, //   次に呼ばれるTick数が保持される
}
impl PedalPart {
    pub fn new(num: u32) -> Rc<RefCell<PedalPart>> {
        let new_id = ElapseId {
            pid: 0,
            sid: num,
            elps_type: ElapseType::TpPedalPart,
        };
        Rc::new(RefCell::new(Self {
            id: new_id,
            priority: PRI_DMPR,
            during_play: false,
            start_flag: false,
            position: 127,
            pedal_msg: None,
            pedal_loop: None,
            elapsed_tick: 0,
            loop_whole_tick: 0,
            do_loop: false,
            sync: false,

            // for super's member
            next_msr: 0,  //   次に呼ばれる小節番号が保持される
            next_tick: 0, //   次に呼ばれるTick数が保持される
        }))
    }
    pub fn set_position(&mut self, pos: i16) {
        self.position = pos;
    }
    /// Damper Event を ElapseStack に追加する
    fn push_pedal_event(&mut self, estk: &mut ElapseStack, sid: u32, position: u8, tick: i16) {
        let dmpr: Rc<RefCell<dyn Elapse>> = Damper::new(
            sid,         //  msr&read pointer
            self.id.sid, //  pedal part.sid -> damper.pid
            position,    //&evt[trace],
            self.next_msr,
            tick as i32,
        );
        estk.add_elapse(Rc::clone(&dmpr));
    }
    /// 1小節分の Damper Event を生成する。小節頭でコールされる
    /// 返り値: (Damper Event List, 次の Tick)
    fn gen_events_in_msr(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        let (tick_for_onemsr, tick_for_onebeat) = estk.tg().get_beat_tick();
        let beat_num: usize = (tick_for_onemsr / tick_for_onebeat) as usize;

        // Damper PhrData メッセージが無い場合、各パートの Chord 情報から生成する
        let mut chord_map = vec![PedalPos::NoEvt; beat_num];
        if let Some(_fl) = estk.get_flow() {
            chord_map = PedalPart::merge_chord_map(crnt_, estk, FLOW_PART, chord_map, beat_num);
        }
        for i in 0..MAX_KBD_PART {
            if let Some(phr) = estk.get_phr(i) {
                if phr.borrow().get_noped() {
                    // 一パートでも noped 指定があれば
                    chord_map = vec![PedalPos::Off; beat_num];
                    break;
                } else {
                    chord_map = PedalPart::merge_chord_map(crnt_, estk, i, chord_map, beat_num);
                }
            } else {
                continue;
            }
        }
        self.gen_damper_event_from_chord_map(estk, chord_map, tick_for_onebeat, beat_num);
    }
    /// 各パートのChord情報より、Damper 情報を beat にどんどん足していく
    fn merge_chord_map(
        crnt_: &CrntMsrTick,
        estk: &mut ElapseStack,
        part_num: usize,
        mut chord_map: Vec<PedalPos>,
        beat_num: usize,
    ) -> Vec<PedalPos> {
        if let Some(pt) = estk.part(part_num as u32) {
            let mut pt_borrowed = pt.borrow_mut();
            let cmp_med = pt_borrowed.get_cmps_med();
            let ba = cmp_med.get_damper_ev_map(crnt_, beat_num);
            if ba.len() != chord_map.len() {
                // もし長さが違ったら、エラー
                println!(
                    "<<< part{}/beat{}: {}->{}",
                    part_num,
                    beat_num,
                    chord_map.len(),
                    ba.len()
                );
                panic!("DamperPart::merge_chord_map: length mismatch");
            }
            for (i, x) in chord_map.iter_mut().enumerate() {
                match *x {
                    // 新しいイベントのマージ方法
                    PedalPos::Full => {
                        continue; // すでに Full なら、変更しない
                    }
                    PedalPos::Half => {
                        if ba[i] == PedalPos::Full {
                            *x = PedalPos::Full; // Full が来たら、Full にする
                        }
                    }
                    PedalPos::Off => {
                        if ba[i] == PedalPos::Full || ba[i] == PedalPos::Half {
                            *x = ba[i]; // Full or Half が来たら、変更する
                        }
                    }
                    PedalPos::NoEvt => {
                        *x = ba[i];
                    }
                }
            }
        }
        chord_map
    }
    /// Damper Event を、chord map から生成する
    fn gen_damper_event_from_chord_map(
        &mut self,
        estk: &mut ElapseStack,
        chord_map: Vec<PedalPos>,
        tick_for_onebeat: i32,
        beat_num: usize,
    ) {
        let mut keep: usize = beat_num;
        let mut idx = 0;
        const PDL_MARGIN_TICK: i32 = 60;
        for (j, k) in chord_map.iter().enumerate() {
            if *k == PedalPos::Full {
                if keep != beat_num {
                    let tick = ((keep as i32) * tick_for_onebeat + PDL_MARGIN_TICK) as i16;
                    self.push_pedal_event(
                        estk,
                        (self.next_msr as u32) * 100 + idx,
                        self.position as u8,
                        tick,
                    );
                    idx += 1;
                    self.push_pedal_event(
                        estk,
                        (self.next_msr as u32) * 100 + idx,
                        0,
                        tick + (((j - keep) as i32) * tick_for_onebeat - PDL_MARGIN_TICK) as i16,
                    );
                    idx += 1;
                }
                keep = j;
            }
        }
        if keep != beat_num {
            let tick = ((keep as i32) * tick_for_onebeat + PDL_MARGIN_TICK) as i16;
            self.push_pedal_event(
                estk,
                (self.next_msr as u32) * 100 + idx,
                self.position as u8,
                tick,
            );
            idx += 1;
            self.push_pedal_event(
                estk,
                (self.next_msr as u32) * 100 + idx,
                0,
                tick + (((beat_num - keep) as i32) * tick_for_onebeat - PDL_MARGIN_TICK) as i16,
            );
        }
    }
    /// Pedal Part に PhrData メッセージを受信する
    pub fn rcv_phr_msg(&mut self, msg: PhrData, _crnt_: &CrntMsrTick, _estk: &mut ElapseStack) {
        self.pedal_msg = Some(msg);
    }
    fn begin_new_loop(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack, msg: &PhrData) {
        let ploop = Rc::new(RefCell::new(PedalLoop::new(
            crnt_.msr as u32,
            self.id.sid,
            msg,
        )));
        self.pedal_loop = Some(Rc::clone(&ploop));
        estk.add_elapse(ploop);
        self.loop_whole_tick = msg.whole_tick as i32;
        self.do_loop = msg.do_loop;
        if self.pedal_msg.is_none() {
            self.elapsed_tick = 0; // 次の小節から再生
        }
        self.elapsed_tick = 0;
    }
    /// sync command 発行時にコールされる
    pub fn set_sync(&mut self) {
        self.sync = true;
    }
}
impl Elapse for PedalPart {
    fn id(&self) -> ElapseId {
        self.id
    } // id を得る
    fn prio(&self) -> u32 {
        self.priority
    } // priority を得る
    fn next(&self) -> (i32, i32, bool) {
        // 次に呼ばれる小節番号、Tick数を返す
        (self.next_msr, self.next_tick, false)
    }
    fn start(&mut self, msr: i32) {
        // User による start/play 時にコールされる
        self.during_play = true;
        self.start_flag = true;
        self.next_msr = msr;
        self.next_tick = 0;
    }
    fn stop(&mut self, estk: &mut ElapseStack) {
        // User による stop 時にコールされる
        self.during_play = false;
        // midi damper off
        estk.midi_out(0xb0, 0x40, 0);
    }
    /// 再生データを消去
    fn clear(&mut self, _estk: &mut ElapseStack) {
        self.next_msr = 0;
        self.next_tick = 0;
    }
    fn process(&mut self, crnt_: &CrntMsrTick, estk: &mut ElapseStack) {
        if self.pedal_msg.is_none() {
            // Damper Event を生成
            self.gen_events_in_msr(crnt_, estk);
        } else if let Some(pmsg) = &self.pedal_msg {
            if self.loop_whole_tick != 0 {
                self.elapsed_tick += crnt_.tick_for_onemsr;
            }
            if self.elapsed_tick >= self.loop_whole_tick && crnt_.msr >= 1 || self.sync {
                // Loop 開始
                let pmsg_clone = pmsg.clone();
                self.begin_new_loop(crnt_, estk, &pmsg_clone);
                if self.do_loop {
                    self.elapsed_tick = 0;
                } else {
                    self.pedal_msg = None; // 1回限りの再生
                }
                self.sync = false;
            }
        }
        // 次の小節の先頭をセット
        self.next_msr = crnt_.msr + 1;
        self.next_tick = 0;
    }
    fn rcv_sp(&mut self, _msg: ElapseMsg, _msg_data: u8) {}
    fn destroy_me(&self) -> bool {
        // 自クラスが役割を終えた時に True を返す
        false
    }
}
