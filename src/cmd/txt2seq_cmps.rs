//  Created by Hasebe Masahiko on 2023/02/24.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use crate::lpnlib::*;

//*******************************************************************
//          Chord Tables and IF
//*******************************************************************
const ROOT_NAME: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];
struct ChordTable {
    name: &'static str,
    table: &'static [i16],
}

#[rustfmt::skip]
const CHORD_TABLE: [ChordTable; 58] = [
    ChordTable {name: "X",      table: &THRU,}, // noped
    ChordTable {name: "O",      table: &THRU,},
    ChordTable {name: "_",      table: &MAJOR,},
    ChordTable {name: "_m",     table: &MINOR,},
    ChordTable {name: "_7",     table: &M7TH,},
    ChordTable {name: "_m7",    table: &MIN7TH,},
    ChordTable {name: "_6",     table: &MAJ6TH,},
    ChordTable {name: "_m6",    table: &MIN6TH,},
    ChordTable {name: "_M7",    table: &MAJ7TH,},
    ChordTable {name: "_maj7",  table: &MAJ7TH,},

    ChordTable {name: "_mM7",   table: &MINMAJ7TH,},
    ChordTable {name: "_add9",  table: &ADD9TH,},
    ChordTable {name: "_9",     table: &M9TH,},
    ChordTable {name: "_m9",    table: &MIN9TH,},
    ChordTable {name: "_M9",    table: &MAJ9TH,},
    ChordTable {name: "_mM9",   table: &MINMAJ9TH,},
    ChordTable {name: "_maj9",  table: &MAJ9TH,},
    ChordTable {name: "_+5",    table: &AUG5TH,},
    ChordTable {name: "_aug",   table: &AUG5TH,},
    ChordTable {name: "_7+5",   table: &AUG57TH,},

    ChordTable {name: "_aug7",  table: &AUG7TH,},
    ChordTable {name: "_7-9",   table: &M7MNS9,},
    ChordTable {name: "_7+9",   table: &M7PLS9,},
    ChordTable {name: "_M96",   table: &MAJ9ADD6,},
    ChordTable {name: "_dim",   table: &DIM,},
    ChordTable {name: "_dim7",  table: &DIM7,},
    ChordTable {name: "_m7-5",  table: &MIN7M5,},
    ChordTable {name: "_sus4",  table: &SUS4,},
    ChordTable {name: "_7sus4", table: &M7SUS4,},
    // parasc(29-34): para() を付けなくても、para機能
    ChordTable {name: "_chr",   table: &THRU,}, // Iのとき音程そのまま。音程関係を保持したまま並行移動

    ChordTable {name: "_ion",   table: &IONIAN,}, // Iが音程そのまま。Iとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name: "_dor",   table: &IONIAN,}, // IIが音程そのまま。IIとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name: "_lyd",   table: &IONIAN,}, // IVが音程そのまま。IVとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name: "_mix",   table: &IONIAN,}, // Vが音程そのまま。Vとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name: "_aeo",   table: &IONIAN,}, // VIが音程そのまま。VIとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name: "diatonic",table: &IONIAN,},
    ChordTable {name: "dorian", table: &DORIAN,},
    ChordTable {name: "lydian", table: &LYDIAN,},
    ChordTable {name: "mixolydian",table: &MIXOLYDIAN,},
    ChordTable {name: "aeolian",table: &AEOLIAN,},

    ChordTable {name: "comdim", table: &COMDIM,},
    ChordTable {name: "pentatonic",table: &PENTATONIC,},
    ChordTable {name: "blues",  table: &BLUES,},
    // scale n(38-49): n半音分上の diatonic scale
    ChordTable {name: "sc0",    table: &IONIAN,},
    ChordTable {name: "sc1",    table: &SC1,},
    ChordTable {name: "sc2",    table: &SC2,},
    ChordTable {name: "sc3",    table: &SC3,},
    ChordTable {name: "sc4",    table: &SC4,},
    ChordTable {name: "sc5",    table: &MIXOLYDIAN,},
    ChordTable {name: "sc6",    table: &SC6,},

    ChordTable {name: "sc7",    table: &LYDIAN,},
    ChordTable {name: "sc8",    table: &SC8,},
    ChordTable {name: "sc9",    table: &SC9,},
    ChordTable {name: "sc10",   table: &SC10,},
    ChordTable {name: "sc11",   table: &SC11,},
    ChordTable {name: "Err",    table: &ERR,},
    ChordTable {name: "None",   table: &NONE,},
    ChordTable {name: "LPEND",  table: &NONE,}, // elapse では、再生が止まる
];

pub const NO_LOOP: i16 = (CHORD_TABLE.len() - 1) as i16;
pub const MAX_CHORD_TABLE: usize = CHORD_TABLE.len();
pub const NO_PED_TBL_NUM: usize = 0; // 'X'
const THRU: [i16; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const MAJOR: [i16; 3] = [0, 4, 7];
const MINOR: [i16; 3] = [0, 3, 7];
const M7TH: [i16; 4] = [0, 4, 7, 10];
const MAJ6TH: [i16; 4] = [0, 4, 7, 9];
const MIN6TH: [i16; 4] = [0, 3, 7, 9];
const MIN7TH: [i16; 4] = [0, 3, 7, 10];
const MAJ7TH: [i16; 4] = [0, 4, 7, 11];
const MINMAJ7TH: [i16; 4] = [0, 3, 7, 11];
const ADD9TH: [i16; 4] = [0, 2, 4, 7];
const M9TH: [i16; 5] = [0, 2, 4, 7, 10];
const MIN9TH: [i16; 5] = [0, 2, 3, 7, 10];
const MAJ9TH: [i16; 5] = [0, 2, 4, 7, 11];
const MINMAJ9TH: [i16; 5] = [0, 2, 3, 7, 11];
const AUG5TH: [i16; 3] = [0, 4, 8];
const AUG57TH: [i16; 4] = [0, 4, 8, 10];
const AUG7TH: [i16; 4] = [0, 4, 8, 11];
const M7MNS9: [i16; 5] = [0, 1, 4, 7, 10];
const M7PLS9: [i16; 5] = [0, 3, 4, 7, 10];
const DIM: [i16; 3] = [0, 3, 6];
const DIM7: [i16; 4] = [0, 3, 6, 9];
const MIN7M5: [i16; 4] = [0, 3, 6, 10];
const SUS4: [i16; 3] = [0, 5, 7];
const M7SUS4: [i16; 4] = [0, 5, 7, 10];
const MAJ9ADD6: [i16; 6] = [0, 2, 4, 7, 9, 11]; // Ionian-F
const IONIAN: [i16; 7] = [0, 2, 4, 5, 7, 9, 11]; // Ionian
const DORIAN: [i16; 7] = [0, 2, 3, 5, 7, 9, 10]; // Dorian
const LYDIAN: [i16; 7] = [0, 2, 4, 6, 7, 9, 11]; // Lydian
const MIXOLYDIAN: [i16; 7] = [0, 2, 4, 5, 7, 9, 10]; // Mixolydian
const AEOLIAN: [i16; 7] = [0, 2, 3, 5, 7, 8, 10]; // Aeolian
const COMDIM: [i16; 8] = [0, 2, 3, 5, 6, 8, 9, 11];
const PENTATONIC: [i16; 5] = [0, 2, 4, 7, 9];
const BLUES: [i16; 6] = [0, 3, 5, 6, 7, 10];
const ERR: [i16; 1] = [0];
const NONE: [i16; 1] = [0];
const SC1: [i16; 7] = [0, 1, 3, 5, 6, 8, 10];
const SC2: [i16; 7] = [1, 2, 4, 6, 7, 9, 11];
const SC3: [i16; 7] = [0, 2, 3, 5, 7, 8, 10];
const SC4: [i16; 7] = [1, 3, 4, 6, 8, 9, 11];
const SC6: [i16; 7] = [1, 3, 5, 6, 8, 10, 11];
const SC8: [i16; 7] = [0, 1, 3, 5, 7, 8, 10];
const SC9: [i16; 7] = [1, 2, 4, 6, 8, 9, 11];
const SC10: [i16; 7] = [0, 2, 3, 5, 7, 9, 10];
const SC11: [i16; 7] = [1, 3, 4, 6, 8, 10, 11];

pub fn get_root_name(idx_num: usize) -> &'static str {
    assert!(idx_num < ROOT_NAME.len());
    ROOT_NAME[idx_num]
}
pub fn get_table(idx_num: usize) -> (&'static [i16], bool) {
    let mut idx = idx_num;
    let mut upper = false;
    if idx > UPPER as usize {
        idx -= UPPER as usize;
        upper = true;
    }
    assert!(idx < MAX_CHORD_TABLE);
    (CHORD_TABLE[idx].table, upper)
}
pub fn get_table_name(idx_num: i16) -> &'static str {
    if idx_num == NO_TABLE {
        return "";
    }
    let idx: usize = if idx_num > UPPER {
        (idx_num - UPPER) as usize
    } else {
        idx_num as usize
    };
    if idx >= MAX_CHORD_TABLE {
        eprintln!("Error: idx_num out of bounds: {}", idx); // idx_num を表示
        panic!("Assertion failed: idx_num < MAX_CHORD_TABLE");
    }
    CHORD_TABLE[idx].name
}
pub fn get_table_num(kind: &str) -> i16 {
    let mut table: i16 = (MAX_CHORD_TABLE - 2) as i16;

    for (i, tp) in CHORD_TABLE.iter().enumerate() {
        if tp.name == kind {
            table = i as i16;
            break;
        }
    }
    table
}
pub fn is_movable_scale(mut idx_num: i16, root: i16) -> (bool, i16) {
    if idx_num > UPPER {
        idx_num -= UPPER;
    }
    const CHURCH_SCALE_BASE_NOTE: [i16; 6] = [0, 0, 2, 5, 7, 9]; //parasc()使用table分
    let lo_num = get_table_num("_chr");
    let hi_num = get_table_num("_aeo");
    let mut rt: i16 = 0;
    if idx_num >= lo_num && idx_num <= hi_num {
        let idx = (idx_num - lo_num) as usize;
        if idx < CHURCH_SCALE_BASE_NOTE.len() {
            rt = CHURCH_SCALE_BASE_NOTE[idx];
        }
        rt = (root - rt) % 12;
        (true, rt)
    } else {
        (false, rt)
    }
}

//*******************************************************************
//          complement_composition
//*******************************************************************
pub fn complement_composition(input_text: String) -> Option<Vec<String>> {
    // 1. {} を抜き出し、２つ分の brackets を Vec に入れて戻す
    if let Some(cd) = divide_brace(input_text) {
        // 2. 重複補填と ',' で分割
        let cmps_vec = fill_omitted_chord_data(cd);

        Some(cmps_vec)
    } else {
        None
    }
}
pub fn divide_brace(input_text: String) -> Option<String> {
    let isx: &str = &input_text;
    isx.find('}').map(|n2| isx[1..n2].to_string())
}
fn fill_omitted_chord_data(mut cmps: String) -> Vec<String> {
    let cmp_len = cmps.len();
    if cmp_len == 0 {
        return vec!["".to_string()];
    } else if cmp_len >= 2 && cmps.ends_with("//") {
        cmps.pop();
        cmps += "LPEND";
    }

    const NO_CHORD: &str = "X"; // 省略を X で補填
    let mut fill: String = "".to_string(); // cmps に補填して fill に入れる
    let mut chord: String = NO_CHORD.to_string(); // 補填用の chord
    let mut end_flag: bool = true; // 補填して区切られ済み

    for ltr in cmps.chars() {
        if ltr == ',' {
            fill += &chord;
            fill += ",";
            chord = NO_CHORD.to_string();
            end_flag = true;
        } else if ltr == '/' || ltr == '|' {
            fill += &chord;
            fill += "|,";
            chord = NO_CHORD.to_string();
            end_flag = true;
        } else if end_flag {
            chord = ltr.to_string(); // 最初の文字を chord に入れる
            end_flag = false;
        } else {
            chord.push(ltr); // 文字を chord に追加
        }
    }
    if !chord.is_empty() {
        fill += &chord; // 最後の文字
    }
    fill += "|";

    // space を削除
    fill.retain(|c| !c.is_whitespace());

    // ',' で分割
    split_by(',', fill)
}

//*******************************************************************
//          recombine_to_chord_loop
//*******************************************************************
pub fn recombine_to_chord_loop(
    comp: &[String],
    tick_for_onemsr: i32,
    tick_for_onebeat: i32,
) -> (i32, bool, Vec<ChordEvt>) {
    if comp.is_empty() {
        return (0, true, Vec::new());
    }
    let max_read_ptr = comp.len();
    let mut read_ptr = 0;

    let mut chord: String;
    let mut dur: i32 = 0;
    let mut tick: i32 = 0;
    let mut msr: i32 = 1;
    let mut rcmb = Vec::new();
    let mut same_chord: String = "path".to_string();

    while read_ptr < max_read_ptr {
        // generate new tick
        if dur != LAST {
            tick += tick_for_onebeat * dur;
        }
        if dur == LAST || tick >= tick_for_onemsr * msr {
            tick = tick_for_onemsr * msr;
            msr += 1;
        }

        let mut msgs = comp[read_ptr].clone();
        if msgs.contains("@") {
            let msgs_sp: Vec<&str> = msgs.split('@').collect();
            let num = msgs_sp[1]
                .chars()
                .next()
                .unwrap_or('0')
                .to_digit(10)
                .unwrap_or(0) as i16;
            if num > 0 && num <= 9 {
                rcmb.push(ChordEvt {
                    mtype: TYPE_VARI,
                    tick: tick as i16,
                    root: num,
                    tbl: 0,
                })
            }
            if !msgs_sp[1][1..].is_empty() {
                let rest = msgs_sp[1][1..].to_string();
                msgs = format!("{}{}", msgs_sp[0], rest);
            } else {
                msgs = msgs_sp[0].to_string();
            }
            if msgs.is_empty() {
                msgs = "X".to_string();
            }
        }

        (chord, dur) = divide_chord_and_dur(msgs);
        if chord.is_empty() {
            chord = same_chord.clone();
        } else {
            same_chord = chord.clone();
        }

        let (root, table) = convert_chord_to_num(chord);
        if table == NO_LOOP {
            rcmb.push(ChordEvt {
                mtype: TYPE_CONTROL,
                tick: tick as i16,
                root: 0,
                tbl: table,
            });
        } else {
            rcmb.push(ChordEvt {
                mtype: TYPE_CHORD,
                tick: tick as i16,
                root,
                tbl: table,
            });
        }

        read_ptr += 1;
    }

    let tmp = ChordEvt::default();
    let last_one = rcmb.last().unwrap_or(&tmp);
    let do_loop = last_one.tbl != NO_LOOP;
    if !do_loop {
        rcmb.pop();
    }
    (msr * tick_for_onemsr, do_loop, rcmb)
}
fn divide_chord_and_dur(mut chord: String) -> (String, i32) {
    let mut dur: i32 = 1;
    let mut ltr_count = chord.len();
    assert!(ltr_count != 0);

    let last_ltr = chord.chars().last().unwrap_or(' ');
    let mut msr_line: bool = false;
    if last_ltr == '|' {
        chord = chord[0..ltr_count - 1].to_string();
        msr_line = true;
    }

    let mut last_ltr = chord.chars().last().unwrap_or(' ');
    while ltr_count >= 1 && last_ltr == '.' {
        dur += 1;
        chord = chord[0..ltr_count - 1].to_string();
        last_ltr = chord.chars().last().unwrap_or(' ');
        ltr_count = chord.len();
    }
    if msr_line {
        dur = LAST;
    }

    (chord, dur)
}
fn convert_chord_to_num(mut chord: String) -> (i16, i16) {
    let mut root: i16 = 2;
    let mut kind: String = "".to_string();
    let mut root_str: String = "".to_string();
    let mut ltr_cnt = 0;
    let length = chord.len();
    let last_ltr = chord.chars().last().unwrap_or(' ');
    let mut take_upper = false;

    //  check up/down translate
    if last_ltr == '!' {
        take_upper = true;
        chord = chord[0..length - 1].to_string();
    }

    // extract root from chord
    loop {
        if length <= ltr_cnt {
            break;
        }
        let ltr = chord.chars().nth(ltr_cnt).unwrap_or(' ');
        if ltr == 'I' || ltr == 'V' {
            root_str.push(ltr);
        } else if ltr == 'b' {
            root = 1;
            ltr_cnt += 1;
            break;
        } else if ltr == '#' {
            root = 3;
            ltr_cnt += 1;
            break;
        } else {
            break;
        }
        ltr_cnt += 1;
    }

    //  separate with chord, decide root number
    if length > ltr_cnt {
        kind = chord[ltr_cnt..].to_string();
    }
    let mut found = false;
    for (i, rn) in ROOT_NAME.iter().enumerate() {
        if rn == &root_str {
            root += 3 * (i as i16);
            kind = "_".to_string() + &kind;
            found = true;
            break;
        }
    }
    if !found {
        root = NO_ROOT;
    }

    //  search chord type from Table
    let table = get_table_num(&kind) + if take_upper { UPPER } else { 0 };

    (root, table)
}
