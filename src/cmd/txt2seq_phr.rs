//  Created by Hasebe Masahiko on 2023/02/15.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib::*;

//*******************************************************************
//          complement_phrase
//*******************************************************************
pub fn complement_phrase(input_text: String) -> [Vec<String>;2] {
    // 1. [] を抜き出し、２つ分の brackets を Vec に入れて戻す
    let (ni, ne) = divide_brackets(input_text);

    // 2. ,| 重複による休符指示の補填
    let mut nf = fill_omitted_note_data(ni);
    nf.retain(|c| !c.is_whitespace());  // space を削除

    // 3. , で分割
    let mut nvec = split_by(',', nf);

    // 4. < >*n を展開
    loop {
        let (nvr_tmp, no_exist) = expand_repeat(nvec.clone());
        nvec = nvr_tmp.clone();
        if no_exist {break;}
    }

    // 5. 同音繰り返しの展開
    loop {
        let (nvr_tmp, no_exist) = note_repeat(nvec.clone());
        nvec = nvr_tmp.clone();
        if no_exist {break;}
    }

    // 6. Expression を , で分割
    ne.clone().retain(|c| !c.is_whitespace());  // space を削除
    let nevec = split_by(',', ne);

    return [nvec,nevec];
}
fn divide_brackets(input_text: String) -> (String, String) {
    let mut note_info: Vec<String> = Vec::new();

    // [] のセットを抜き出し、中身を note_info に入れる
    let mut isx: &str = &input_text;
    loop {
        if let Some(n2) = isx.find(']') {
            note_info.push(isx[1..n2].to_string());
            isx = &isx[n2+1..];
            if isx.len() == 0 {break;}
            if let Some(n3) = isx.find('[') {
                if n3 != 0 {break;}
            }
            else {break;}
        }
        else {break;}
    }

    let bracket_num = note_info.len();
    if bracket_num == 1 {
        note_info.push("raw".to_string());
    }
    else if bracket_num == 0 || bracket_num > 2 {
        return ("".to_string(), "".to_string());
    }
    return (note_info[0].clone(), note_info[1].clone());
}
fn fill_omitted_note_data(nf: String) -> String {
    if nf.len() == 0 {return "".to_string();}

    let mut fill: String = "".to_string();
    let mut doremi = "x".to_string();
    let mut doremi_end_flag = true;
    for ltr in nf.chars() {
        if ltr == ',' {
            fill += &doremi;
            fill += ",";
            doremi = "x".to_string();// ,| 連続入力による、休符指示の補填
            doremi_end_flag = true;
        }
        else if ltr == '|' || ltr == '/' {
            fill += &doremi;
            fill += "|,";
            doremi = "x".to_string();// ,| 連続入力による、休符指示の補填
            doremi_end_flag = true;
        }
        else {
            if doremi_end_flag {
                doremi = (ltr).to_string();
                doremi_end_flag = false;
            }
            else {
                doremi.push(ltr);
            }
        }
    }
    if doremi != "" {
        fill += &doremi;
    }
    fill
}
fn expand_repeat(nv: Vec<String>) -> (Vec<String>, bool) {
    let mut new_vec = nv.clone();
    let mut repeat_start: usize = nv.len();
    let mut first_bracket: bool = false;
    let mut no_exist: bool = true;
    for (i, one) in nv.iter().enumerate() {
        if one.contains("<") {
            if let Some(ltr) = &one.chars().nth(0) {
                if *ltr == '<' {
                    new_vec[i] = one[1..].to_string();
                    repeat_start = i;
                    first_bracket = true;
                }
            }
        }
        else if first_bracket && one.contains(">") {
            no_exist = false;
            let mut remain_num: usize = one.len();
            for (j, ltr) in one.chars().enumerate() {
                if ltr == '>' {
                    new_vec[i] = one[0..j].to_string();
                    remain_num = j;
                }
                else if j == remain_num+1 && ltr == '*' {
                    let number: i32 = one[j+1..].parse().unwrap_or(0);
                    if number > 1 {
                        for _ in 0..number-1 {
                            // 繰り返しマーク(&RPT)の挿入
                            new_vec.insert(i+1, "$RPT".to_string());
                            for h in (repeat_start..(i+1)).rev() {
                                // 繰り返しの後ろから、同じindexに挿入していく
                                new_vec.insert(i+2, new_vec[h].clone());
                            }
                        }
                    }
                    break;
                }
            }
            break;
        }
    }
    (new_vec, no_exist)
}
fn note_repeat(nv: Vec<String>) -> (Vec<String>, bool) {
    let mut new_vec = nv.clone();
    let mut no_exist: bool = true;
    for (i, one) in nv.iter().enumerate() {
        if one.contains("*") {
            no_exist = false;
            for (j, ltr) in one.chars().enumerate() {
                if ltr == '*' {
                    new_vec[i] = one[..j].to_string();
                    let number: i32 = one[j+1..].parse().unwrap_or(0);
                    if number > 1 {
                        for _ in 0..number-1 {
                            new_vec.insert(i+1, one[..j].to_string());
                        }
                    }
                    break;
                }
            }
            break;
        }
    }
    (new_vec, no_exist)
}

//*******************************************************************
//          recombine_to_internal_format
//*******************************************************************
pub fn recombine_to_internal_format(ntvec: &Vec<String>, expvec: &Vec<String>, imd: InputMode,
    base_note: i32, tick_for_onemsr: i32) -> (i32, Vec<Vec<i16>>) {
    let max_read_ptr = ntvec.len();
    let (exp_vel, _exp_others) = get_exp_info(expvec.clone());
    let mut read_ptr = 0;
    let mut last_nt: i32 = 0;
    let mut tick: i32 = 0;
    let mut msr: i32 = 1;
    let mut base_dur: i32 = DEFAULT_TICK_FOR_QUARTER;
    let mut rcmb: Vec<Vec<i16>> = Vec::new();
    let mut mes_top: bool = false;

    while read_ptr < max_read_ptr {
        let note_text = ntvec[read_ptr].clone();

        let (notes, mes_end, dur_cnt, diff_vel, bdur, lnt)
            = break_up_nt_dur_vel(note_text, base_note, base_dur, last_nt, imd);
        base_dur = bdur;
        last_nt = lnt;    // 次回の音程の上下判断のため

        if notes[0] >= MAX_NOTE_NUMBER || notes[0] < MIN_NOTE_NUMBER {
            match notes[0] {
                RPT_HEAD => {
                    // 繰り返し指定があったことを示すイベント
                    let nt_data: Vec<i16> = vec![TYPE_INFO, tick as i16, RPT_HEAD as i16, 0,0];
                    rcmb.push(nt_data);                    
                }
                _ => (),
            }
        }
        else {
            let next_msr_tick = tick_for_onemsr*msr;
            if tick < next_msr_tick {
                // duration
                let note_dur = get_real_dur(base_dur, dur_cnt, next_msr_tick - tick);
    
                // velocity
                let mut last_vel: i32 = exp_vel + diff_vel;
                if last_vel > 127 {last_vel = 127;}
                else if last_vel < 1 {last_vel = 1;}
    
                // add to recombined data
                rcmb = add_note(rcmb, tick, notes, note_dur, last_vel as i16, mes_top);
                tick += note_dur;
            }
            if mes_end {// 小節線があった場合
                tick = next_msr_tick;
                msr += 1;
                mes_top = true;
            }
            else {mes_top = false;}
        }
        read_ptr += 1;  // out from repeat
    }
    (tick, rcmb)
}
fn get_exp_info(expvec: Vec<String>) -> (i32, Vec<String>) {
    let mut vel = END_OF_DATA;
    let mut retvec = expvec.clone();
    for (i, txt) in expvec.iter().enumerate() {
        vel = convert_exp2vel(txt);
        if vel != END_OF_DATA {
            retvec.remove(i);
            break;
        }
    }
    if vel == END_OF_DATA {vel=DEFAULT_VEL as i32;}
    (vel, retvec)
}
fn break_up_nt_dur_vel(note_text: String, base_note: i32, bdur: i32, last_nt: i32, imd: InputMode)
    -> (Vec<u8>, bool, i32, i32, i32, i32)
    //(notes, mes_end, dur_cnt, diff_vel, base_dur, last_nt)
    {
    if note_text.chars().nth(0).unwrap_or(' ') == '$' {
        return (vec![RPT_HEAD], false, 0, 0, bdur, last_nt)
    }

    //  小節線のチェック
    let mut mes_end = false;
    let mut ntext1 = note_text.clone();
    if note_text.chars().last().unwrap_or(' ') == '|' { // 小節最後のイベント
        mes_end = true;
        ntext1.pop();
    }

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, base_dur, dur_cnt) = gen_dur_info(ntext1, bdur);
    let (ntext4, diff_vel) = gen_diff_vel(ntext3);
    let notes_vec: Vec<String> = split_by_by('=', '_', ntext4);

    let mut notes: Vec<u8> = Vec::new();
    let mut doremi: i32 = 0;
    for nt in notes_vec.iter() {    // 同時発音
        if imd == InputMode::Fixed {
            doremi = convert_doremi_fixed(nt.to_string());
        }
        else if imd == InputMode::Closer {
            doremi = convert_doremi_closer(nt.to_string(), last_nt);
        }
        let mut base_pitch: i32;
        if doremi >= NO_MIDI_VALUE as i32 { // special meaning
            base_pitch = doremi;
            doremi = last_nt;
        }
        else {
            base_pitch = base_note + doremi;
            if base_pitch >= MAX_NOTE_NUMBER as i32 {base_pitch = MAX_NOTE_NUMBER as i32;}
            else if base_pitch < MIN_NOTE_NUMBER as i32 {base_pitch = MIN_NOTE_NUMBER as i32;}
        }
        notes.push(base_pitch as u8);
    }
    (notes, mes_end, dur_cnt, diff_vel, base_dur, doremi)
}
fn gen_dur_info(nt: String, bdur: i32) -> (String, i32, i32) {
    // 階名指定が無く、小節冒頭のタイの場合の音価を判定
    let first_ltr = nt.chars().nth(0).unwrap_or(' ');
    if first_ltr == 'o' {return ("".to_string(), bdur, LAST);}
    if first_ltr == '.' {
        let mut dot_cnt = 0;
        for ltr in nt.chars() {if ltr == '.' {dot_cnt += 1;}}
        return ("".to_string(), bdur, dot_cnt);
    }
    let mut excnt = 0;
    if first_ltr == '+' || first_ltr == '-' {
        // +- は、最初にあっても、音価指定の後にあってもいいので、一番前にある +- を削除して、
        // 音価情報を分析、除去した後、あらためて削除した +- を元に戻す
        for (i, ltr) in nt.chars().enumerate() {
            if ltr == '+' || ltr == '-' {continue;}
            else {
                excnt = i;
                break;
            }
        }
    }
    let mut ntext = nt[excnt..].to_string();

    //  タイなどの音価を解析し、dur_cnt を確定
    let mut dur_cnt: i32 = 1;
    let txtlen = ntext.len(); 
    if txtlen > 0 {
        if ntext.chars().nth(txtlen-1).unwrap_or(' ') == 'o' {
            dur_cnt = LAST;
            ntext.pop();
        }
        else {
            loop {
                let length = ntext.len();
                if length == 0 {break;}
                let ltr = ntext.chars().nth(length-1).unwrap_or(' ');
                if ltr == '.' || ltr == '~' {
                    dur_cnt += 1;
                    ntext.pop();
                }
                else {break;}
            }
        }
    }

    //  基準音価を解析し、base_dur を確定
    let mut base_dur = bdur;
    let txtlen = ntext.len();
    if txtlen > 0 {
        let mut triplet: i16 = 0;
        let mut idx = 1;
        let mut fst_ltr = ntext.chars().nth(0).unwrap_or(' ');
        if fst_ltr == '3' || fst_ltr == '5' {
            triplet = fst_ltr.to_digit(10).unwrap_or(1) as i16;
            fst_ltr = ntext.chars().nth(1).unwrap_or(' ');
        }
        if fst_ltr == '\'' {
            if ntext.chars().nth(2).unwrap_or(' ') == '\"' {
                base_dur = DEFAULT_TICK_FOR_QUARTER/8;
                idx = 2;
            }
            else {base_dur = DEFAULT_TICK_FOR_QUARTER/2;}
        }
        else if fst_ltr == '\"' {base_dur = DEFAULT_TICK_FOR_QUARTER/4;}
        else if fst_ltr == 'q' {base_dur = DEFAULT_TICK_FOR_QUARTER;}
        else if fst_ltr == 'h' {base_dur = DEFAULT_TICK_FOR_QUARTER*2;}
        else {idx = 0;}
        if triplet != 0 {
            base_dur = (base_dur*2)/triplet as i32;
            idx = 2;
        }
        ntext = ntext[idx..].to_string();
    }

    //  +- を戻す
    if excnt != 0 {
        ntext = nt[0..excnt].to_string() + &ntext;
    }
    (ntext, base_dur, dur_cnt)
}
fn gen_diff_vel(nt: String) -> (String, i32) {
    let mut ntext = nt;
    let mut diff_vel = 0;
    let mut last_ltr = ntext.chars().nth(ntext.len()-1).unwrap_or(' ');
    while last_ltr == '^' {
        diff_vel += 10;
        ntext.pop();
        last_ltr = ntext.chars().nth(ntext.len()-1).unwrap_or(' ');
    }
    while last_ltr == '%' {
        diff_vel -= 20;
        ntext.pop();
        last_ltr = ntext.chars().nth(ntext.len()-1).unwrap_or(' ');
    }
    (ntext, diff_vel)
}
fn get_real_dur(base_dur: i32, dur_cnt: i32, rest_tick: i32) -> i32 {
    if dur_cnt == LAST {rest_tick}
    else if base_dur == KEEP {base_dur*dur_cnt}
    else {base_dur*dur_cnt}
}
fn add_note(rcmb: Vec<Vec<i16>>, tick: i32, notes: Vec<u8>, note_dur: i32, last_vel: i16, mes_top: bool)
    -> Vec<Vec<i16>> {
    assert!(notes.len() != 0);
    let mut return_rcmb = rcmb.clone();
    for note in notes.iter() {
        if *note == REST {
            continue;
        }
        else if *note == NO_NOTE {
            if mes_top {
                // 小節先頭にタイがあった場合、前の音の音価を増やす
                // 前回の入力が '=' による和音入力だった場合も考え、直前の同じタイミングのデータを全て調べる
                let mut search_idx = return_rcmb.len()-1;
                let last_tick = return_rcmb[search_idx][1];
                loop {
                    if return_rcmb[search_idx][1] == last_tick {
                        return_rcmb[search_idx][2] += note_dur as i16;
                    }
                    else {break;}
                    if search_idx == 0 {break;}
                    search_idx -= 1;
                }
            }
            else {continue;}
        }
        else {
            let nt_data: Vec<i16> = 
                vec![TYPE_NOTE, tick as i16, note_dur as i16, *note as i16, last_vel];
            return_rcmb.push(nt_data);
        }
    }
    return_rcmb
}
fn convert_doremi_closer(doremi: String, last_nt: i32) -> i32 {
    if doremi.len() == 0 {return NO_NOTE as i32;}
    let mut last_doremi = last_nt;
    while last_doremi >= 12 {last_doremi -= 12;}
    while last_doremi < 0 {last_doremi += 12;}

    let mut oct_pitch = 0;
    let mut pure_doremi = String::from("");
    for (i, ltr) in doremi.chars().enumerate() {
        if ltr == 'x' {return REST as i32;}
        else if ltr == '+' {oct_pitch += 12;}
        else if ltr == '-' {oct_pitch -= 12;}
        else {
            pure_doremi = doremi[i..].to_string();
            break;
        }
    }

    let mut base_note = 0;
    if pure_doremi.len() != 0 {
        base_note = doremi_number(pure_doremi.chars().nth(0).unwrap_or(' '), base_note);
        pure_doremi.remove(0);
    }
    else {return NO_NOTE as i32;}

    if pure_doremi.len() != 0 {
        base_note = doremi_semi_number(pure_doremi.chars().nth(0).unwrap_or(' '), base_note);
    }

    let base_pitch: i32;
    if oct_pitch == 0 { // +/- が書かれていない場合
        let mut diff = base_note - last_doremi;
        if diff < 0 {diff += 12;}
        if diff > 6 {base_pitch = last_nt+diff-12;}
        else {base_pitch = last_nt+diff;}
    }
    else if oct_pitch > 0 { // + 書かれている場合
        while base_note - last_nt >= 12 {base_note -= 12;}
        while base_note - last_nt <= oct_pitch - 12 {base_note += 12;}
        base_pitch = base_note;
    }
    else {  // - 書かれている場合
        while base_note - last_nt <= -12 {base_note += 12;}
        while base_note - last_nt >= oct_pitch + 12 {base_note -= 12;}
        base_pitch = base_note;
    }
    base_pitch
}
fn convert_doremi_fixed(doremi: String) -> i32 {
    if doremi.len() == 0 {return NO_NOTE as i32;}
    let mut base_pitch: i32 = 0;
    let mut pure_doremi = String::from("");
    for (i, ltr) in doremi.chars().enumerate() {
        if ltr == 'x' {return REST as i32;}
        else if ltr == '+' {base_pitch += 12;}
        else if ltr == '-' {base_pitch -= 12;}
        else {
            pure_doremi = doremi[i..].to_string();
            break;
        }
    }
    if pure_doremi.len() != 0 {
        base_pitch = doremi_number(pure_doremi.chars().nth(0).unwrap_or(' '), base_pitch);
    }
    else {return NO_NOTE as i32;}

    if pure_doremi.len() > 1 {
        base_pitch = doremi_semi_number(pure_doremi.chars().nth(1).unwrap_or(' '), base_pitch);
    }
    base_pitch
}

