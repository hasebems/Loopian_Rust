//  Created by Hasebe Masahiko on 2023/12/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use crate::lpnlib::*;
use super::seq_stock::SeqDataStock;

pub struct MessageSender {
    msg_hndr: mpsc::Sender<ElpsMsg>,
}

impl MessageSender {
    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>) -> Self {
        Self {
            msg_hndr,
        }
    }
    pub fn send_msg_to_elapse(&self, msg: ElpsMsg) {
        match self.msg_hndr.send(msg) {
            Err(e) => println!("Something happened on MPSC for Elps! {}",e),
            _ => {},
        }
    }
    pub fn send_all_vari_and_phrase(&self, part: usize, gdt: &SeqDataStock) {
        for i in 0..MAX_PHRASE {
            self.send_phrase_to_elapse(part, i, gdt);
        }
    }
    pub fn send_phrase_to_elapse(&self, part: usize, vari: usize, gdt: &SeqDataStock) {
        //let msg_pv = (part as i16) + 10*(vari as i16);
        let (pdt, ana) = gdt.get_pdstk(part, vari).get_final(part as i16, vari as i16);
        let msg = pdt.clone();
        match pdt {
            ElpsMsg::Phr(_m0, _m1, mv) => {
                if mv.evts.len() > 0 {
                    self.send_msg_to_elapse(msg);
                    let amsg = ana.clone();
                    match ana {
                        ElpsMsg::Ana(_a0, _a1, av) => {
                            if av.evts.len() > 0 {
                                self.send_msg_to_elapse(amsg);
                            }
                        }
                        _ => {}
                    }
                }
                else {
                    self.send_msg_to_elapse(ElpsMsg::PhrX(part as i16, vari as i16));
                    match ana {
                        ElpsMsg::Ana(_a0, _a1, av) => {
                            if av.evts.len() == 0 {
                                self.send_msg_to_elapse(ElpsMsg::AnaX(part as i16, vari as i16));
                            }
                            println!("Part {} Phrase: No Data!",part);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    pub fn send_composition_to_elapse(&self, part: usize, gdt: &SeqDataStock) {
        let cdt = gdt.get_cdstk(part).get_final(part as i16);
        let cmsg = cdt.clone();
        match cdt {
            ElpsMsg::Cmp(_c0, cv) => {
                if cv.evts.len() > 0 {
                    self.send_msg_to_elapse(cmsg);
                }
                else {
                    self.send_msg_to_elapse(ElpsMsg::CmpX(part as i16));
                    println!("Part {} Composition: No Data!",part)
                }
            }
            _ => {}
        }
    }
}