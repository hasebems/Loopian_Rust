//  Created by Hasebe Masahiko on 2023/06/05.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use std::cmp::Ordering;

use crate::cmd::txt2seq_cmps;
use crate::lpnlib::*;

//*******************************************************************
//          Func
//*******************************************************************
pub fn translate_note_parascl(para_note: i16, ctbl: i16, ev_note: u8) -> u8 {
    let input_nt = (ev_note as i16) + para_note;
    let input_doremi = input_nt % 12;
    let input_oct = input_nt / 12;
    let mut output_doremi = 0;
    let mut former_nt = 0;
    let (tbl_cow, take_upper) = txt2seq_cmps::get_table(ctbl as usize);
    let tbl: &[i16] = tbl_cow.as_ref();
    if tbl == txt2seq_cmps::THRU {
        // THRU の場合はそのまま返す
        return ev_note;
    }
    for ntx in tbl.iter() {
        match ntx.cmp(&input_doremi) {
            Ordering::Equal => {
                output_doremi = input_doremi;
                break;
            }
            Ordering::Greater => {
                if (input_doremi - former_nt > *ntx - input_doremi)
                    || ((input_doremi - former_nt == *ntx - input_doremi) && take_upper)
                {
                    // 等距離なら
                    output_doremi = *ntx;
                }
                break;
            }
            Ordering::Less => {
                former_nt = *ntx;
                output_doremi = former_nt;
            }
        }
    }
    output_doremi += input_oct * 12;
    while output_doremi < 0 {
        output_doremi += 12;
    }
    while output_doremi >= 128 {
        output_doremi -= 12;
    }
    output_doremi as u8
}
pub fn translate_note_com(root: i16, ctbl: i16, tgt_nt: u8) -> u8 {
    let tgt_nt = tgt_nt as i16;
    let mut proper_nt = tgt_nt;
    let (tbl_cow, take_upper) = txt2seq_cmps::get_table(ctbl as usize);
    let tbl: &[i16] = tbl_cow.as_ref();
    if tbl == txt2seq_cmps::THRU {
        // THRU の場合はそのまま返す
        return tgt_nt as u8;
    }
    let real_root = root + DEFAULT_NOTE_NUMBER as i16;
    let mut former_nt: i16 = 0;
    let mut found = false;
    let oct_adjust = if tgt_nt - real_root >= 0 {
        (tgt_nt - (real_root + tbl[0])) / 12
    } else {
        ((tgt_nt - 12) - (real_root + tbl[0])) / 12
    };
    for ntx in tbl.iter() {
        proper_nt = *ntx + real_root + oct_adjust * 12;
        match proper_nt.cmp(&tgt_nt) {
            Ordering::Equal => {
                found = true;
                break;
            }
            Ordering::Greater => {
                match (tgt_nt - former_nt).cmp(&(proper_nt - tgt_nt)) {
                    Ordering::Less | Ordering::Equal if !take_upper => {
                        proper_nt = former_nt;
                    }
                    _ => {}
                }
                found = true;
                break;
            }
            Ordering::Less => {
                former_nt = proper_nt;
            }
        }
    }
    if !found {
        proper_nt = tbl[0] + real_root + (oct_adjust + 1) * 12;
        match (tgt_nt - former_nt).cmp(&(proper_nt - tgt_nt)) {
            Ordering::Less | Ordering::Equal if !take_upper => proper_nt = former_nt,
            _ => {}
        }
    }
    while proper_nt < 0 {
        proper_nt += 12;
    }
    while proper_nt >= 128 {
        proper_nt -= 12;
    }
    proper_nt as u8
}
pub fn _translate_note_arp(root: i16, ctbl: i16, nt_diff: i16, last_note: i16) -> i16 {
    // nt_diff: User Input による、前に発音したノートとの差分
    // arp_nt: 前回発音したノートに nt_diff を足したもの
    let arp_nt = last_note + nt_diff;
    let mut nty = DEFAULT_NOTE_NUMBER as i16;
    let (tbl_cow, _take_upper) = txt2seq_cmps::get_table(ctbl as usize);
    let tbl: &[i16] = tbl_cow.as_ref();
    match nt_diff.cmp(&0) {
        Ordering::Equal => arp_nt,
        Ordering::Greater => {
            let mut ntx = last_note + 1;
            ntx = search_scale_nt_just_above(root, tbl, ntx);
            if ntx >= arp_nt {
                return ntx;
            }
            while nty < 128 {
                nty = ntx + 1;
                nty = search_scale_nt_just_above(root, tbl, nty);
                if nty >= arp_nt {
                    if nty - arp_nt > arp_nt - ntx {
                        nty = ntx;
                    }
                    break;
                }
                ntx = nty;
            }
            nty
        }
        Ordering::Less => {
            let mut ntx = last_note - 1;
            ntx = search_scale_nt_just_below(root, tbl, ntx);
            if ntx <= arp_nt {
                return ntx;
            }
            while nty >= 0 {
                nty = ntx - 1;
                nty = search_scale_nt_just_below(root, tbl, nty);
                if nty <= arp_nt {
                    if arp_nt - nty > ntx - arp_nt {
                        nty = ntx;
                    }
                    break;
                }
                ntx = nty;
            }
            nty
        }
    }
}
pub fn translate_note_arp2(root: i16, ctbl: i16, tgt_nt: u8, nt_diff: i16, last_note: i16) -> u8 {
    let tgt_nt = tgt_nt as i16;
    let mut proper_nt = tgt_nt;
    let (tbl_cow, take_upper) = txt2seq_cmps::get_table(ctbl as usize);
    let tbl: &[i16] = tbl_cow.as_ref();
    if tbl == txt2seq_cmps::THRU {
        // THRU の場合はそのまま返す
        return tgt_nt as u8;
    }
    let real_root = root + DEFAULT_NOTE_NUMBER as i16;
    let mut former_nt: i16 = 0;
    let mut found = false;
    let oct_adjust = if tgt_nt - real_root >= 0 {
        (tgt_nt - (real_root + tbl[0])) / 12
    } else {
        ((tgt_nt - 11) - (real_root + tbl[0])) / 12
    };
    for ntx in tbl.iter() {
        proper_nt = *ntx + real_root + oct_adjust * 12;
        match proper_nt.cmp(&tgt_nt) {
            Ordering::Equal => {
                found = true;
                break;
            }
            Ordering::Greater => {
                if (tgt_nt - former_nt < proper_nt - tgt_nt)
                    || ((tgt_nt - former_nt == proper_nt - tgt_nt) && !take_upper)
                {
                    // 等距離なら
                    proper_nt = former_nt;
                }
                found = true;
                break;
            }
            Ordering::Less => {}
        }
        former_nt = proper_nt;
    }
    if !found {
        proper_nt = tbl[0] + real_root + (oct_adjust + 1) * 12;
        if (tgt_nt - former_nt < proper_nt - tgt_nt)
            || ((tgt_nt - former_nt == proper_nt - tgt_nt) && !take_upper)
        {
            // 等距離なら
            proper_nt = former_nt
        }
    }
    match (proper_nt.cmp(&last_note), nt_diff.cmp(&0)) {
        (Ordering::Equal, _)
        | (Ordering::Greater, Ordering::Less)
        | (Ordering::Less, Ordering::Greater) => {
            // 前回と同じ音か、アルペジオの方向が逆のとき、方向が同じ別の音を探す
            proper_nt = if nt_diff > 0 {
                search_scale_nt_just_above(root, tbl, proper_nt + 1)
            } else {
                search_scale_nt_just_below(root, tbl, proper_nt - 1)
            };
        }
        _ => {}
    }
    while proper_nt < 0 {
        proper_nt += 12;
    }
    while proper_nt >= 128 {
        proper_nt -= 12;
    }
    proper_nt as u8
}
fn search_scale_nt_just_above(root: i16, tbl: &[i16], nt: i16) -> i16 {
    // nt の音程より上にある(nt含む)、一番近い root/tbl の音程を探す
    let mut scale_nt: i16 = 0;
    let mut octave: i16 = -1;
    while nt > scale_nt {
        // Octave 判定
        octave += 1;
        scale_nt = root + octave * 12;
    }
    scale_nt = 0;
    octave -= 1;
    let mut cnt: i16 = -1;
    while nt > scale_nt {
        //Table index 判定
        cnt += 1;
        if cnt >= tbl.len() as i16 {
            octave += 1;
            cnt = 0;
        }
        scale_nt = root + tbl[cnt as usize] + octave * 12;
    }
    scale_nt
}
fn search_scale_nt_just_below(root: i16, tbl: &[i16], nt: i16) -> i16 {
    // nt の音程から下にある(nt含む)、一番近い root/tbl の音程を探す
    let mut scale_nt: i16 = 0;
    let mut octave: i16 = -1;
    while nt > scale_nt {
        // Octave 判定
        octave += 1;
        scale_nt = root + octave * 12;
    }
    scale_nt = MAX_NOTE_NUMBER as i16;
    octave -= 1;
    let mut cnt = tbl.len() as i16;
    while nt < scale_nt {
        // Table index 判定
        cnt -= 1;
        if cnt < 0 {
            octave -= 1;
            cnt = tbl.len() as i16 - 1;
        }
        scale_nt = root + tbl[cnt as usize] + octave * 12;
    }
    scale_nt
}
