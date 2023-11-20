//  Created by Hasebe Masahiko on 2023/02/24.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;
use crate::elapse::ug_content::*;

//*******************************************************************
//          Chord Tables and IF
//*******************************************************************
const ROOT_NAME: [&'static str; 7] = ["I","II","III","IV","V","VI","VII"];
struct ChordTable {
    name: &'static str,
    table: &'static [i16],
}
const CHORD_TABLE: [ChordTable; 52] = [
    //  No.0-9
    ChordTable {name:   "thru",     table:  &THRU,},    // noped
    ChordTable {name:   "O",        table:  &THRU,},
    ChordTable {name:   "_",        table:  &MAJOR,},
    ChordTable {name:   "_m",       table:  &MINOR,},
    ChordTable {name:   "_7",       table:  &M7TH,},
    ChordTable {name:   "_6",       table:  &MAJ6TH,},
    ChordTable {name:   "_m7",      table:  &MIN7TH,},
    ChordTable {name:   "_M7",      table:  &MAJ7TH,},
    ChordTable {name:   "_maj7",    table:  &MAJ7TH,},
    ChordTable {name:   "_add9",    table:  &ADD9TH,},

    // No.10-19
    ChordTable {name:   "_9",       table:  &M9TH,},
    ChordTable {name:   "_m9",      table:  &MIN9TH,},
    ChordTable {name:   "_M9",      table:  &MAJ9TH,},
    ChordTable {name:   "_maj9",    table:  &MAJ9TH,},
    ChordTable {name:   "_+5",      table:  &AUG5TH,},
    ChordTable {name:   "_aug",     table:  &AUG5TH,},
    ChordTable {name:   "_7+5",     table:  &AUG7TH,},
    ChordTable {name:   "_aug7",    table:  &AUG7TH,},
    ChordTable {name:   "_7-9",     table:  &M7MNS9,},
    ChordTable {name:   "_7+9",     table:  &M7PLS9,},

    // No.20-29
    ChordTable {name:   "_dim",     table:  &DIM,},
    ChordTable {name:   "_m7-5",    table:  &MIN7M5,},
    ChordTable {name:   "_sus4",    table:  &SUS4,},
    ChordTable {name:   "_7sus4",   table:  &M7SUS4,},
    // parasc(24-29): para() を付けなくても、para機能
    ChordTable {name:   "_chr",     table:  &THRU,},    // Iのとき音程そのまま。音程関係を保持したまま並行移動
    ChordTable {name:   "_ion",     table:  &IONIAN,},  // Iが音程そのまま。Iとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name:   "_dor",     table:  &IONIAN,},  // IIが音程そのまま。IIとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name:   "_lyd",     table:  &IONIAN,},  // IVが音程そのまま。IVとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name:   "_mix",     table:  &IONIAN,},  // Vが音程そのまま。Vとの差分分並行移動し、音程をkeyに合わせる
    ChordTable {name:   "_aeo",     table:  &IONIAN,},  // VIが音程そのまま。VIとの差分分並行移動し、音程をkeyに合わせる

    // No.30-39
    ChordTable {name:   "diatonic",     table:  &IONIAN,},
    ChordTable {name:   "dorian",       table:  &DORIAN,},
    ChordTable {name:   "lydian",       table:  &LYDIAN,},
    ChordTable {name:   "mixolydian",   table:  &MIXOLYDIAN,},
    ChordTable {name:   "aeolian",      table:  &AEOLIAN,},
    ChordTable {name:   "comdim",       table:  &COMDIM,},
    ChordTable {name:   "pentatonic",   table:  &PENTATONIC,},
    ChordTable {name:   "blues",        table:  &BLUES,},
    // scale n(38-49): n半音分上の diatonic scale
    ChordTable {name:   "sc0",      table:  &IONIAN,},
    ChordTable {name:   "sc1",      table:  &SC1,},

    // No.40-49
    ChordTable {name:   "sc2",      table:  &SC2,},
    ChordTable {name:   "sc3",      table:  &SC3,},
    ChordTable {name:   "sc4",      table:  &SC4,},
    ChordTable {name:   "sc5",      table:  &MIXOLYDIAN,},
    ChordTable {name:   "sc6",      table:  &SC6,},
    ChordTable {name:   "sc7",      table:  &LYDIAN,},
    ChordTable {name:   "sc8",      table:  &SC8,},
    ChordTable {name:   "sc9",      table:  &SC9,},
    ChordTable {name:   "sc10",     table:  &SC10,},
    ChordTable {name:   "sc11",     table:  &SC11,},
 
    // No.50-
    ChordTable {name:   "Err",      table:  &ERR,},
    ChordTable {name:   "None",     table:  &NONE,},
];
pub const MAX_CHORD_TABLE: usize = CHORD_TABLE.len();

const THRU:   [i16; 12] = [0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11];
const MAJOR:  [i16; 3]  = [0,  4,  7];
const MINOR:  [i16; 3]  = [0,  3,  7];
const M7TH:   [i16; 4]  = [0, 4, 7, 10];
const MAJ6TH: [i16; 4]  = [0, 4, 7, 9];
const MIN7TH: [i16; 4]  = [0, 3, 7, 10];
const MAJ7TH: [i16; 4]  = [0, 4, 7, 11];
const ADD9TH: [i16; 4]  = [0, 2, 4, 7];
const M9TH:   [i16; 5]  = [0, 2, 4, 7, 10];
const MIN9TH: [i16; 5]  = [0, 2, 3, 7, 10];
const MAJ9TH: [i16; 5]  = [0, 2, 4, 7, 11];
const AUG5TH: [i16; 3]  = [0, 4, 8];
const AUG7TH: [i16; 4]  = [0, 4, 8, 10];
const M7MNS9: [i16; 5]  = [0, 1, 4, 7, 10];
const M7PLS9: [i16; 5]  = [0, 3, 4, 7, 10];
const DIM:    [i16; 4]  = [0, 3, 6, 9];
const MIN7M5: [i16; 4]  = [0, 3, 6, 10];
const SUS4:   [i16; 3]  = [0, 5, 7];
const M7SUS4: [i16; 4]  = [0, 5, 7, 10];
const IONIAN: [i16; 7]  = [0, 2, 4, 5, 7, 9, 11]; // Ionian
const DORIAN: [i16; 7]  = [0, 2, 3, 5, 7, 9, 10]; // Dorian
const LYDIAN: [i16; 7]  = [0, 2, 4, 6, 7, 9, 11]; // Lydian
const MIXOLYDIAN: [i16; 7]  = [0, 2, 4, 5, 7, 9, 10]; // Mixolydian
const AEOLIAN:[i16; 7]  = [0, 2, 3, 5, 7, 8, 10]; // Aeolian
const COMDIM: [i16; 8]  = [0, 2, 3, 5, 6, 8, 9, 11];
const PENTATONIC:[i16; 5] = [0, 2, 4, 7, 9];
const BLUES:  [i16; 6]  = [0, 3, 5, 6, 7, 10];
const ERR:    [i16; 1]  = [0];
const NONE:   [i16; 1]  = [0];
const SC1:    [i16; 7]  = [0, 1, 3, 5, 6, 8, 10];
const SC2:    [i16; 7]  = [1, 2, 4, 6, 7, 9, 11];
const SC3:    [i16; 7]  = [0, 2, 3, 5, 7, 8, 10];
const SC4:    [i16; 7]  = [1, 3, 4, 6, 8, 9, 11];
const SC6:    [i16; 7]  = [1, 3, 5, 6, 8, 10,11];
const SC8:    [i16; 7]  = [0, 1, 3, 5, 7, 8, 10];
const SC9:    [i16; 7]  = [1, 2, 4, 6, 8, 9, 11];
const SC10:   [i16; 7]  = [0, 2, 3, 5, 7, 9, 10];
const SC11:   [i16; 7]  = [1, 3, 4, 6, 8, 10,11];

pub fn get_root_name(idx_num: usize) -> &'static str {
    assert!(idx_num < ROOT_NAME.len());
    ROOT_NAME[idx_num]
}
pub fn get_table(mut idx_num: usize) -> (&'static [i16], bool) {
    let mut upper = false;
    if idx_num > UPPER as usize {
        idx_num -= UPPER as usize;
        upper = true;
    }
    assert!(idx_num < MAX_CHORD_TABLE);
    (CHORD_TABLE[idx_num].table, upper)
}
pub fn get_table_name(mut idx_num: usize) -> &'static str {
    if idx_num > UPPER as usize {idx_num -= UPPER as usize;}
    assert!(idx_num < MAX_CHORD_TABLE);
    CHORD_TABLE[idx_num].name
}
pub fn get_table_num(kind: &str) -> i16 {
    let mut table: i16 = (MAX_CHORD_TABLE-1) as i16;
    for (i, tp) in CHORD_TABLE.iter().enumerate() {
        if tp.name == kind {
            table = i as i16;
            break;
        }
    }
    table
}
pub fn is_movable_scale(mut idx_num: i16, root: i16) -> (bool, i16) {
    if idx_num > UPPER {idx_num -= UPPER;}
    const CHURCH_SCALE_BASE_NOTE: [i16; 6] = [0,0,2,5,7,9];//parasc()使用table分
    let lo_num = get_table_num("_chr");
    let hi_num = get_table_num("_aeo");
    let mut rt: i16 = 0;
    if idx_num >= lo_num && idx_num <= hi_num {
        let idx = (idx_num - lo_num) as usize;
        if idx < CHURCH_SCALE_BASE_NOTE.len() {
            rt = CHURCH_SCALE_BASE_NOTE[idx];
        }
        rt = (root-rt)%12;
        (true,rt)
    }
    else {(false,rt)}
}

//*******************************************************************
//          complement_composition
//*******************************************************************
pub fn complement_composition(input_text: String) -> Option<[Vec<String>;2]> {
    // 1. {} を抜き出し、２つ分の brackets を Vec に入れて戻す
    if let Some((cd, ce)) = divide_brace(input_text){
        //println!("{:?}",cd);

        // 2. 重複補填と ',' で分割
        let cmps_vec = fill_omitted_chord_data(cd);

        // 3. Expression を ',' で分割
        let ex_vec = split_by(',', ce);

        Some([cmps_vec, ex_vec])
    }
    else {None}
}
pub fn divide_brace(input_text: String) -> Option<(String, String)> {
    let mut cmps_info: Vec<String> = Vec::new();

    // {} のセットを抜き出し、中身を cmps_info に入れる
    let mut isx: &str = &input_text;
    loop {
        if let Some(n2) = isx.find('}') {
            cmps_info.push(isx[1..n2].to_string());
            isx = &isx[n2+1..];
            if isx.len() == 0 {break;}
            if let Some(n3) = isx.find('{') {
                if n3 != 0 {break;}
            }
            else {break;}
        }
        else {break;}
    }

    let blk_cnt = cmps_info.len();
    if blk_cnt >= 2 {
        Some((cmps_info[0].clone(), cmps_info[1].clone()))
    }
    else if blk_cnt == 1 {
        Some((cmps_info[0].clone(), "".to_string()))
    }
    else {None}
}
fn fill_omitted_chord_data(cmps: String) -> Vec<String> {
    if cmps.len() == 0 {return vec!["".to_string()];}

    const NO_CHORD: &str = "thru";                  // 省略を thru で補填
    let mut fill: String = "".to_string();          // cmps に補填して fill に入れる
    let mut chord: String = NO_CHORD.to_string();   // 補填用の chord
    let mut end_flag: bool = true;                  // 補填して区切られ済み

    for ltr in cmps.chars() {
        if ltr == ',' {
            fill += &chord;
            fill += ",";
            chord = NO_CHORD.to_string();
            end_flag = true;
        }
        else if ltr == '/' || ltr == '|' {
            fill += &chord;
            fill += "|,";
            chord = NO_CHORD.to_string();
            end_flag = true;
        }
        else if end_flag {
            chord = ltr.to_string(); // 最初の文字を chord に入れる
            end_flag = false;
        }
        else {
            chord.push(ltr);    // 文字を chord に追加
        }
    }
    if chord != "" {
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
pub fn recombine_to_chord_loop(comp: &Vec<String>, tick_for_onemsr: i32, tick_for_onebeat: i32)
    -> (i32, UgContent) {
    if comp.len() == 0 {
        let mut zero = UgContent::new();
        zero.add_dt(vec![0]);
        return (0, zero);
    }
    let max_read_ptr = comp.len();
    let mut read_ptr = 0;

    let mut chord: String;
    let mut dur: i32 = 0;
    let mut tick: i32 = 0;
    let mut msr: i32 = 1;
    let mut rcmb = UgContent::new();
    let mut same_chord: String = "path".to_string();

    while read_ptr < max_read_ptr {
        // generate new tick
        if dur != LAST {
            tick += tick_for_onebeat*dur;
        }
        if dur == LAST || tick >= tick_for_onemsr*msr {
            tick = tick_for_onemsr*msr;
            msr += 1;
        }

        let mut msgs = comp[read_ptr].clone();
        if msgs.chars().any(|x| x==':') {
            // Variation 指定があるか
            let msgs_in_same: Vec<&str> = msgs.split(':').collect();
            let ltr = msgs_in_same[0].chars().nth(0).unwrap_or(' ');
            let num = msgs_in_same[0][1..].parse().unwrap_or(0);
            if msgs_in_same[0].len() == 2 && ltr == '@' && num > 0 {
                // 2文字で、1文字目は'@' 2文字目は 1-9 の数字
                rcmb.add_dt(vec![TYPE_VARI, tick as i16, num, 0]);
            }
            msgs = msgs_in_same[1].to_string();
        }

        (chord, dur) = divide_chord_and_dur(msgs);
        if chord == "" {chord = same_chord.clone();}
        else {same_chord = chord.clone();}

        let (root, table) = convert_chord_to_num(chord);
        rcmb.add_dt(vec![TYPE_CHORD, tick as i16, root, table]);

        read_ptr += 1;
    }        

    tick = msr*tick_for_onemsr;
    (tick, rcmb)
}
fn divide_chord_and_dur(mut chord: String) -> (String, i32) {
    let mut dur: i32 = 1;
    let mut ltr_count = chord.len();
    assert!(ltr_count!=0);

    let last_ltr = chord.chars().last().unwrap_or(' ');
    let mut msr_line: bool = false;
    if last_ltr == '|' {
        chord = chord[0..ltr_count-1].to_string();
        msr_line = true;
    }

    let mut last_ltr = chord.chars().last().unwrap_or(' ');
    while ltr_count >= 1 && last_ltr == '.' {
        dur += 1;
        chord = chord[0..ltr_count-1].to_string();
        last_ltr = chord.chars().last().unwrap_or(' ');
        ltr_count = chord.len();
    }
    if msr_line {dur=LAST;}

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
        chord = chord[0..length-1].to_string();
    }

    // extract root from chord
    loop {
        if length <= ltr_cnt {break;}
        let ltr = chord.chars().nth(ltr_cnt).unwrap_or(' ');
        if ltr == 'I' || ltr == 'V' {
            root_str.push(ltr);
        }
        else if ltr == 'b' {
            root = 1;
            ltr_cnt += 1;
            break;
        }
        else if ltr == '#' {
            root = 3;
            ltr_cnt += 1;
            break;
        }
        else {break;}
        ltr_cnt += 1;
    }

    //  separate with chord, decide root number
    if length > ltr_cnt {
        kind = chord[ltr_cnt..].to_string();
    }
    let mut found = false;
    for (i, rn) in ROOT_NAME.iter().enumerate() {
        if rn == &root_str {
            root += 3*(i as i16);
            kind = "_".to_string() + &kind;
            found = true;
            break;
        }
    }
    if !found {root = NO_ROOT;}

    //  search chord type from Table
    //println!("*<Chord: {}, {}, {}>*", root, root_str, kind);
    let table = get_table_num(&kind) + if take_upper {UPPER} else {0};

    (root, table)
}
