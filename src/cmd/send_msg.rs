//  Created by Hasebe Masahiko on 2023/12/30.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::seq_stock::SeqDataStock;
use crate::lpnlib::*;
use std::sync::mpsc;

pub struct MessageSender {
    msg_hndr: mpsc::Sender<ElpsMsg>,
}

impl MessageSender {
    pub fn new(msg_hndr: mpsc::Sender<ElpsMsg>) -> Self {
        Self { msg_hndr }
    }
    pub fn send_msg_to_elapse(&self, msg: ElpsMsg) {
        if let Err(e) = self.msg_hndr.send(msg) {
            println!("Something happened on MPSC for Elps! {e}");
        }
    }
    pub fn send_all_vari_and_phrase(&self, part: usize, gdt: &SeqDataStock) {
        for i in 0..MAX_VARIATION {
            if i == 0 {
                // Normal の場合、Phrase が空でない、do_loop の場合のみ送信
                if let Some(pdt) = gdt.get_pdstk(part, PhraseAs::Normal) {
                    if !pdt.get_phr().is_empty() {
                        self.send_phrase_to_elapse(part, PhraseAs::Normal, gdt);
                    }
                }
            } else {
                self.send_phrase_to_elapse(part, PhraseAs::Variation(i), gdt);
            }
        }
    }
    pub fn send_phrase_to_elapse(&self, part: usize, vari: PhraseAs, gdt: &SeqDataStock) {
        if let Some(pdt) = gdt.get_pdstk(part, vari.clone()) {
            self.send_msg_to_elapse(pdt.get_final(part as i16, vari));
        }
    }
    pub fn clear_phrase_to_elapse(&self, part: usize) {
        self.send_msg_to_elapse(ElpsMsg::PhrX(part as i16));
    }
    pub fn send_composition_to_elapse(&self, part: usize, gdt: &SeqDataStock) {
        let cdt = gdt.get_cdstk(part).get_final(part as i16);
        let cmsg = cdt.clone();
        if let ElpsMsg::Cmp(_c0, cv) = cdt {
            if cv.evts.is_empty() {
                self.send_msg_to_elapse(ElpsMsg::CmpX(part as i16));
                println!("Part {part} Composition: No Data!");
            } else {
                self.send_msg_to_elapse(cmsg)
            }
        }
    }
}
