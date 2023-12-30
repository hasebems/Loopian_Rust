//  Created by Hasebe Masahiko on 2023/12/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::sync::mpsc;
use crate::lpnlib::*;
use super::seq_stock::SeqDataStock;

pub struct MessageSender {
    msg_hndr: mpsc::Sender<Vec<i16>>,
}

impl MessageSender {
    pub fn new(msg_hndr: mpsc::Sender<Vec<i16>>) -> Self {
        Self {
            msg_hndr,
        }
    }
    pub fn send_msg_to_elapse(&self, msg: Vec<i16>) {
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
        let (mut pdt, mut ana): (Vec<i16>, Vec<i16>) = gdt.get_pdstk(part, vari).get_final();
        let msg_pv = (part as i16) + 10*(vari as i16);
        if pdt.len() > 1 {
            let mut msg: Vec<i16> = vec![MSG_PHR + msg_pv];
            msg.append(&mut pdt);
            //println!("msg check: {:?}",msg);
            self.send_msg_to_elapse(msg);
            if ana.len() > 1 {
                let mut msgana: Vec<i16> = vec![MSG_ANA + msg_pv];
                msgana.append(&mut ana);
                //println!("msg check ana: {:?}",msgana);
                self.send_msg_to_elapse(msgana);                
            }
        }
        else {
            self.send_msg_to_elapse(vec![MSG_PHR_X + msg_pv]);
            if ana.len() == 0 {
                self.send_msg_to_elapse(vec![MSG_ANA_X + msg_pv]);
            }
            println!("Part {} Phrase: No Data!",part);
        }
    }
    pub fn send_composition_to_elapse(&self, part: usize, gdt: &SeqDataStock) {
        let mut cdt: Vec<i16> = gdt.get_cdstk(part).get_final();
        if cdt.len() > 1 {
            let mut msg: Vec<i16> = vec![MSG_CMP+part as i16];
            msg.append(&mut cdt);
            //println!("msg check: {:?}",msg);
            self.send_msg_to_elapse(msg);
        }
        else {
            self.send_msg_to_elapse(vec![MSG_CMP_X+part as i16]);
            println!("Part {} Composition: No Data!",part)
        }
    }
}