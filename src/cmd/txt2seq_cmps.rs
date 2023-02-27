//  Created by Hasebe Masahiko on 2023/02/24.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;

const ROOT_NAME: [&'static str; 7] = ["I","II","III","IV","V","VI","VII"];
struct ChordTable {
    name: &'static str,
    table: &'static [i32],
}
const CHORD_TABLE: [ChordTable; 39] = [
    ChordTable {name:   "thru",     table:  &THRU,},
    ChordTable {name:   "O",        table:  &THRU,},
    ChordTable {name:   "_",        table:  &MAJOR,},
    ChordTable {name:   "_m",       table:  &MINOR,},
    ChordTable {name:   "_7",       table:  &M7TH,},
    ChordTable {name:   "_6",       table:  &MAJ6TH,},
    ChordTable {name:   "_m7",      table:  &MIN7TH,},
    ChordTable {name:   "_M7",      table:  &MAJ7TH,},
    ChordTable {name:   "_maj7",    table:  &MAJ7TH,},
    ChordTable {name:   "_add9",    table:  &ADD9TH,},

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

    ChordTable {name:   "_dim",     table:  &DIM,},
    ChordTable {name:   "_m7-5",    table:  &MIN7M5,},
    ChordTable {name:   "_sus4",    table:  &SUS4,},
    ChordTable {name:   "_7sus4",   table:  &M7SUS4,},
    ChordTable {name:   "_ion",     table:  &IONIAN,},
    ChordTable {name:   "_dor",     table:  &DORIAN,},
    ChordTable {name:   "_lyd",     table:  &LYDIAN,},
    ChordTable {name:   "_mix",     table:  &MIXOLYDIAN,},
    ChordTable {name:   "_aeo",     table:  &AEOLIAN,},
    ChordTable {name:   "diatonic",     table:  &IONIAN,},

    ChordTable {name:   "dorian",       table:  &DORIAN,},
    ChordTable {name:   "lydian",       table:  &LYDIAN,},
    ChordTable {name:   "mixolydian",   table:  &MIXOLYDIAN,},
    ChordTable {name:   "aeolian",      table:  &AEOLIAN,},
    ChordTable {name:   "comdim",       table:  &COMDIM,},
    ChordTable {name:   "pentatonic",   table:  &PENTATONIC,},
    ChordTable {name:   "blues",        table:  &BLUES,},
    ChordTable {name:   "Err",          table:  &ERR,},
    ChordTable {name:   "None",         table:  &NONE,},
];

const THRU:   [i32; 12] = [0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11];
const MAJOR:  [i32; 3]  = [0,  4,  7];
const MINOR:  [i32; 3]  = [0,  3,  7];
const M7TH:   [i32; 4]  = [0, 4, 7, 10];
const MAJ6TH: [i32; 4]  = [0, 4, 7, 9];
const MIN7TH: [i32; 4]  = [0, 3, 7, 10];
const MAJ7TH: [i32; 4]  = [0, 4, 7, 11];
const ADD9TH: [i32; 4]  = [0, 2, 4, 7];
const M9TH:   [i32; 5]  = [0, 2, 4, 7, 10];
const MIN9TH: [i32; 5]  = [0, 2, 3, 7, 10];
const MAJ9TH: [i32; 5]  = [0, 2, 4, 7, 11];
const AUG5TH: [i32; 3]  = [0, 4, 8];
const AUG7TH: [i32; 4]  = [0, 4, 8, 10];
const M7MNS9: [i32; 5]  = [0, 1, 4, 7, 10];
const M7PLS9: [i32; 5]  = [0, 3, 4, 7, 10];
const DIM:    [i32; 4]  = [0, 3, 6, 9];
const MIN7M5: [i32; 4]  = [0, 3, 6, 10];
const SUS4:   [i32; 3]  = [0, 5, 7];
const M7SUS4: [i32; 4]  = [0, 5, 7, 10];
const IONIAN: [i32; 7]  = [0, 2, 4, 5, 7, 9, 11]; // Ionian
const DORIAN: [i32; 7]  = [0, 2, 3, 5, 7, 9, 10]; // Dorian
const LYDIAN: [i32; 7]  = [0, 2, 4, 6, 7, 9, 11]; // Lydian
const MIXOLYDIAN: [i32; 7]  = [0, 2, 4, 5, 7, 9, 10]; // Mixolydian
const AEOLIAN:[i32; 7]  = [0, 2, 3, 5, 7, 8, 10]; // Aeolian
const COMDIM: [i32; 8]  = [0, 2, 3, 5, 6, 8, 9, 11];
const PENTATONIC:[i32; 5] = [0, 2, 4, 7, 9];
const BLUES:  [i32; 6]  = [0, 3, 5, 6, 7, 10];
const ERR:    [i32; 1]  = [0];
const NONE:   [i32; 2]  = [1000, 1001];  // if more than 127, no sound by limit

pub struct TextParseCmps {}
impl TextParseCmps {
    pub fn something_todo(){}
    pub fn complement_composition(input_text: String) -> [Vec<String>;2] {
        // 1. {} を抜き出し、２つ分の brackets を Vec に入れて戻す
        let (cd, ce) = TextParseCmps::divide_brace(input_text);

        // 2. 重複補填と ',' で分割
        let cmps_vec = TextParseCmps::fill_omitted_chord_data(cd);

        // 3. Expression を ',' で分割
        let ex_vec = lpnlib::split_by(',', ce);

        [cmps_vec, ex_vec]
    }
    pub fn divide_brace(input_text: String) -> (String, String) {
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
            (cmps_info[0].clone(), cmps_info[1].clone())
        }
        else if blk_cnt == 1 {
            (cmps_info[0].clone(), "".to_string())
        }
        else {
            ("".to_string(), "".to_string())
        }
    }
    fn fill_omitted_chord_data(cmps: String) -> Vec<String> {
        //  省略を thru で補填
        const NO_CHORD: &str = "thru";
        let mut end_flag: bool = false;
        let mut fill: String = "".to_string();
        let mut chord: String = NO_CHORD.to_string();
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
            else {
                if end_flag {
                    chord = ltr.to_string();
                    end_flag = false;
                }
                else {
                    chord.push(ltr);
                }
            }
        }
        // space を削除
        fill.retain(|c| !c.is_whitespace());

        // ',' で分割
        lpnlib::split_by(',', fill)
    }
    //=========================================================================
    pub fn recombine_to_chord_loop(comp: &Vec<String>, tick_for_onemsr: i32, tick_for_onebeat: i32)
      -> (i32, Vec<Vec<u16>>) {
        if comp.len() == 0 {
            return (0, vec![vec![0]]);
        }
        let btcnt = tick_for_onemsr/tick_for_onebeat;
        let max_read_ptr = comp.len();
        let mut read_ptr = 0;
        let mut tick: i32 = 0;
        let mut msr: i32 = 0;
        let mut rcmb: Vec<Vec<u16>> = Vec::new();
        let mut same_chord: String = "x".to_string();

        while read_ptr < max_read_ptr {
            let (chord, dur) = TextParseCmps::divide_chord_info(comp[read_ptr].clone(), btcnt);
            if tick < tick_for_onemsr*msr {
                if same_chord != chord {
                    same_chord = chord.clone();
                    let (root, table) = TextParseCmps::convert_chord_to_num(chord);
                    rcmb.push(vec![tick as u16, root, table]);
                }
                tick += tick_for_onebeat;
            }
            if dur == btcnt {
                tick = tick_for_onemsr*msr;
                same_chord = "x".to_string();
                msr += 1;
            }
            read_ptr += 1;  // out from repeat
        }        

        tick = msr*tick_for_onemsr;
        (tick, rcmb)
    }
    fn divide_chord_info(mut chord: String, btcnt: i32) -> (String, i32) {
        let mut dur: i32 = 1;
        let mut ltr_count = chord.len();
        if ltr_count == 0 {
            return (chord, dur);
        }

        let mut last_ltr = chord.chars().last().unwrap_or(' ');
        if last_ltr == '|' {
            dur =  btcnt;
            chord = chord[0..ltr_count-1].to_string();
        }
        else {
            while ltr_count >= 1 && last_ltr == '.' {
                dur += 1;
                chord = chord[0..ltr_count-1].to_string();
                last_ltr = chord.chars().last().unwrap_or(' ');
                ltr_count = chord.len();
            }
        }
        (chord, dur)
    }
    fn convert_chord_to_num(chord: String) -> (u16, u16) {
        let root: u16;
        let kind: String;
        let first_three: String = chord[0..3].to_string();

        //  Root
        if first_three == ROOT_NAME[2] {root=2; kind=chord[3..].to_string();}
        else if first_three == ROOT_NAME[6] {root=6; kind=chord[3..].to_string();}
        else {
            let first_two: String = chord[0..2].to_string();
            if first_two == ROOT_NAME[1] {root=1; kind=chord[2..].to_string();}
            else if first_two == ROOT_NAME[3] {root=3; kind=chord[2..].to_string();}
            else if first_two == ROOT_NAME[5] {root=5; kind=chord[2..].to_string();}
            else {
                let first_ltr: char = chord.chars().nth(0).unwrap_or(' ');
                if first_ltr == 'I' {root=0; kind=chord[1..].to_string();}
                else if first_ltr == 'V' {root=4; kind=chord[1..].to_string();}
                else {return (lpnlib::CANCEL, 0);}
            }
        }

        //  Table
        let mut table: u16 = 0;
        for (i, tp) in CHORD_TABLE.iter().enumerate() {
            if tp.name == kind {
                table = i as u16;
                break;
            }
        }
        (root, table)
    }
}