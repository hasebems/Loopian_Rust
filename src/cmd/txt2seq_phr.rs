//  Created by Hasebe Masahiko on 2023/02/15.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use crate::lpnlib::*;

//*******************************************************************
//          complement_phrase
//*******************************************************************
pub fn complement_phrase(
    input_text: String,
    cluster_word: &str,
) -> (Vec<String>, Vec<String>, Vec<bool>) {
    // 1. space 削除
    let phr = input_text.trim().to_string();

    // 2. [] を検出し、音符情報と、その他の情報を分け、音符情報はさらに : で分割、auftaktの展開
    let (nttmp, ne) = divide_brackets(phr);
    let ntdiv = split_by(':', nttmp);
    let (nt, ntatrb) = div_atrb(ntdiv);

    // 3. 関数を . で分割し、音符変調と音楽表現に分ける
    let mut nev = split_by('.', ne);
    nev.retain(|nt| nt != "");
    let (nmvec, nevec) = divide_notemod_and_musicex(nev);

    // 4. <> の検出と、囲まれた要素へのコマンド追加と cluster の展開
    let nttmp = divide_arrow_bracket(nt);
    let nt2 = nttmp.replace('c', cluster_word);

    // 5. ,| 重複による休符指示の補填、音符のVector化
    let nt3 = fill_omitted_note_data(nt2);
    let mut ntvec = split_by(',', nt3);

    // 6. 同音繰り返しの展開
    loop {
        let (nvr_tmp, no_exist) = note_repeat(ntvec.clone());
        ntvec = nvr_tmp.clone();
        if no_exist {
            break;
        }
    }

    // 7. 音符変調関数の適用
    for ne in nmvec.iter() {
        if &ne[0..3] == "rpt" {
            ntvec = repeat_ntimes(ntvec, ne);
        }
    }

    return (ntvec, nevec, ntatrb);
}
fn divide_brackets(input_text: String) -> (String, String) {
    let mut note_info: Vec<String> = Vec::new();

    // [] のセットを抜き出し、中身と、その後の文字列を note_info に入れる
    let mut isx: &str = &input_text;
    if let Some(n2) = isx.find(']') {
        note_info.push(isx[1..n2].to_string());
        isx = &isx[n2 + 1..];
        if isx.len() != 0 {
            note_info.push(isx.to_string());
        }
    }

    let bracket_num = note_info.len();
    if bracket_num == 1 {
        note_info.push("".to_string());
    } else if bracket_num == 0 {
        return ("".to_string(), "".to_string());
    }
    return (note_info[0].clone(), note_info[1].clone());
}
fn divide_arrow_bracket(nt: String) -> String {
    let mut one_arrow_flg = false;
    let mut two_arrow_flg = false;
    let mut arrow_cnt = 0;
    let mut ret_str: String = "".to_string();
    for ltr in nt.chars() {
        if ltr == '<' {
            if arrow_cnt == 1 {
                two_arrow_flg = true;
                one_arrow_flg = false;
            } else if arrow_cnt == 0 {
                one_arrow_flg = true;
            }
            arrow_cnt += 1;
            ret_str.push('>');
        } else if ltr == '>' {
            if two_arrow_flg && arrow_cnt == 0 {
                arrow_cnt = -1;
            } else if one_arrow_flg && arrow_cnt == 0 {
                one_arrow_flg = false;
            } else if arrow_cnt == -1 {
                two_arrow_flg = false;
            } else {
                ret_str.push(ltr);
            }
        } else {
            arrow_cnt = 0;
            ret_str.push(ltr);
            if ltr == ',' {
                if one_arrow_flg {
                    ret_str.push('>');
                } else if two_arrow_flg {
                    ret_str.push('>');
                    ret_str.push('>');
                }
            }
        }
    }
    ret_str
}
fn div_atrb(mut ntdiv: Vec<String>) -> (String, Vec<bool>) {
    let dnum = ntdiv.len();
    let mut nt = "".to_string();
    let mut ntatrb = vec!["".to_string()];
    let mut atrb = vec![false, false];
    if dnum >= 2 {
        nt = ntdiv.pop().unwrap_or("".to_string());
        ntatrb = ntdiv;
    } else if dnum == 1 {
        nt = ntdiv[0].clone();
    }

    // Attribute の調査
    for a in ntatrb.iter() {
        if a.contains('A') {
            let beat = a.chars().nth(1).unwrap_or('0').to_digit(10).unwrap_or(0);
            println!("Auftakt Start Beat: {}", beat);
            if beat > 0 {
                atrb[0] = true;
                if beat > 1 {
                    let mut rest = String::from("qx");
                    for _ in 0..beat - 2 {
                        rest.push_str(".")
                    }
                    nt = rest + "," + &nt;
                }
            }
        } else if a == "RT" {
            atrb[1] = true;
        }
    }

    (nt, atrb)
}
fn fill_omitted_note_data(mut nf: String) -> String {
    let phr_len = nf.len();
    if phr_len == 0 {
        return "".to_string();
    } else if phr_len >= 2 {
        if nf.ends_with("//") {
            nf.pop();
            nf += "LPEND";
        }
    }

    let mut fill: String = "".to_string();
    let mut doremi = "x".to_string();
    let mut doremi_end_flag = true;
    for ltr in nf.chars() {
        if ltr == ',' {
            fill += &doremi;
            fill += ",";
            doremi = "x".to_string(); // ,| 連続入力による、休符指示の補填
            doremi_end_flag = true;
        } else if ltr == '|' || ltr == '/' {
            fill += &doremi;
            fill += "|,";
            doremi = "x".to_string(); // ,| 連続入力による、休符指示の補填
            doremi_end_flag = true;
        } else {
            if doremi_end_flag {
                doremi = (ltr).to_string();
                doremi_end_flag = false;
            } else {
                doremi.push(ltr);
            }
        }
    }
    if doremi != "" {
        fill += &doremi;
    }
    fill
}
/// Note Modulation Function と Music Expression Function を分離する
fn divide_notemod_and_musicex(nev: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut nm: Vec<String> = Vec::new();
    let mut ne: Vec<String> = Vec::new();

    for nx in nev.iter() {
        if &nx[0..3] == "rpt" {
            nm.push(nx.to_string());
        } else {
            ne.push(nx.to_string());
        }
    }
    if ne.len() == 0 {
        ne.push("raw".to_string());
    }
    (nm, ne)
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
                    let number: i32 = one[j + 1..].parse().unwrap_or(0);
                    if number > 1 {
                        for _ in 0..number - 1 {
                            new_vec.insert(i + 1, one[..j].to_string());
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
/// 同じ Phrase を指定回数回、コピーし追加する
fn repeat_ntimes(nv: Vec<String>, ne: &str) -> Vec<String> {
    let mut nnv: Vec<String> = Vec::new();
    let num = extract_number_from_parentheses(ne);
    nnv.extend(nv.clone()); //  repeat前
    for _ in 0..num {
        nnv.push("$RPT".to_string());
        nnv.extend(nv.clone());
    }
    nnv
}

//*******************************************************************
///          recombine_to_internal_format
//*******************************************************************
pub fn recombine_to_internal_format(
    ntvec: &Vec<String>,
    expvec: &Vec<String>,
    imd: InputMode,
    base_note: i32,
    tick_for_onemsr: i32,
) -> (i32, bool, Vec<PhrEvt>) {
    let (exp_vel, _exp_others) = get_dyn_info(expvec.clone());
    let mut read_ptr = 0;
    let mut last_nt: i32 = 0;
    let mut tick: i32 = 0;
    let mut msr: i32 = 1;
    let mut base_dur: i32 = DEFAULT_TICK_FOR_QUARTER;
    let mut rcmb = Vec::new();
    let mut mes_top: bool = false;
    let (max_read_ptr, do_loop) = judge_no_loop(ntvec);

    while read_ptr < max_read_ptr {
        let nt_origin = ntvec[read_ptr].clone();
        let (note_text, trns) = extract_trans_info(nt_origin);

        let (notes, mes_end, dur_cnt, diff_vel, bdur, lnt) =
            break_up_nt_dur_vel(note_text, base_note, base_dur, last_nt, imd);
        base_dur = bdur;
        last_nt = lnt; // 次回の音程の上下判断のため

        assert!(notes.len() != 0);
        if notes[0] == RPT_HEAD {
            // 繰り返し指定があったことを示すイベント
            let nt_data = PhrEvt::gen_repeat(tick as i16);
            rcmb.push(nt_data);
            last_nt = 0; // closed の判断用の前Noteの値をクリアする -> 繰り返し最初の音のオクターブが最初と同じになる
        } else {
            // NO_NOTE 含む（タイの時に使用）
            let next_msr_tick = tick_for_onemsr * msr;
            if tick < next_msr_tick {
                // duration
                let mut note_dur = get_real_dur(base_dur, dur_cnt, next_msr_tick - tick);
                if next_msr_tick - tick < note_dur {
                    note_dur = next_msr_tick - tick; // 小節線を超えたら、音価をそこでリミット
                }

                // velocity
                let mut last_vel: i32 = exp_vel + diff_vel;
                if last_vel > 127 {
                    last_vel = 127;
                } else if last_vel < 1 {
                    last_vel = 1;
                }

                // add to recombined data
                rcmb = add_note(rcmb, tick, notes, note_dur, last_vel as i16, mes_top, trns);
                tick += note_dur;
            }
            if mes_end {
                // 小節線があった場合
                tick = next_msr_tick;
                msr += 1;
                mes_top = true;
            } else {
                mes_top = false;
            }
        }
        read_ptr += 1; // out from repeat
    }
    (tick, do_loop, rcmb)
}
fn judge_no_loop(ntvec: &Vec<String>) -> (usize, bool) {
    let mut max_read_ptr = ntvec.len();
    // LPENDの検出
    let do_loop = if ntvec.ends_with(&["LPEND".to_string()]) {
        false
    } else {
        true
    };
    if !do_loop {
        max_read_ptr -= 1;
    }
    (max_read_ptr, do_loop)
}
fn get_dyn_info(expvec: Vec<String>) -> (i32, Vec<String>) {
    let mut vel = END_OF_DATA;
    let mut retvec = expvec.clone();
    for (i, txt) in expvec.iter().enumerate() {
        if &txt[0..3] == "dyn" {
            let dyntxt = extract_texts_from_parentheses(txt);
            vel = convert_exp2vel(dyntxt);
            if vel != END_OF_DATA {
                retvec.remove(i);
                break;
            }
        }
    }
    if vel == END_OF_DATA {
        vel = convert_exp2vel("p");
    }
    (vel, retvec)
}
fn extract_trans_info(origin: String) -> (String, i16) {
    if origin.len() > 2 && &origin[0..2] == ">>" {
        (origin[2..].to_string(), TRNS_NONE)
    } else if &origin[0..1] == ">" {
        (origin[1..].to_string(), TRNS_PARA)
    } else {
        (origin, TRNS_COM)
    }
}
fn break_up_nt_dur_vel(
    note_text: String,
    base_note: i32,
    bdur: i32,
    last_nt: i32,
    imd: InputMode,
) -> (Vec<u8>, bool, i32, i32, i32, i32)
//(notes, mes_end, dur_cnt, diff_vel, base_dur, last_nt)
{
    let first_ltr = note_text.chars().nth(0).unwrap_or(' ');
    if first_ltr == '$' {
        // complement時に入れた特殊マーク$
        return (vec![RPT_HEAD], false, 0, 0, bdur, last_nt);
    }

    //  小節線のチェック
    let mut mes_end = false;
    let mut ntext1 = note_text.clone();
    if note_text.chars().last().unwrap_or(' ') == '|' {
        // 小節最後のイベント
        mes_end = true;
        ntext1.pop();
    }

    //  頭にOctave記号(+-)があれば、一度ここで抜いておいて、duration/velocity の解析を終えたら文字列を再結合
    let mut oct = "".to_string();
    loop {
        let c = ntext1.chars().nth(0).unwrap_or(' ');
        if c == '+' {
            oct.push('+');
            ntext1.remove(0);
        } else if c == '-' {
            oct.push('-');
            ntext1.remove(0);
        } else {
            break;
        }
    }

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, base_dur, dur_cnt) = gen_dur_info(ntext1, bdur);
    let (ntext4, diff_vel) = gen_diff_vel(ntext3);
    let ntext5 = format!("{}{}", oct, &ntext4); // +-の再結合

    let mut notes_vec: Vec<String> = split_by_by('=', '_', ntext5);
    if notes_vec.len() == 1 {
        notes_vec = split_note(notes_vec[0].clone());
    }

    let mut notes: Vec<u8> = Vec::new();
    let mut next_last_nt = last_nt;
    let notes_num = notes_vec.len();
    if notes_num == 0 {
        notes.push(NO_NOTE);
    } else if notes_num == 1 {
        let mut doremi: i32 = 0;
        if imd == InputMode::Fixed {
            doremi = convert_doremi_fixed(notes_vec[0].to_string());
        } else if imd == InputMode::Closer {
            doremi = convert_doremi_closer(notes_vec[0].to_string(), last_nt);
        }
        if doremi < NO_MIDI_VALUE as i32 {
            // special meaning
            next_last_nt = doremi;
        }
        let base_pitch = add_base_and_doremi(base_note, doremi);
        notes.push(base_pitch);
    } else {
        for nt in notes_vec.iter() {
            // 同時発音
            let doremi = convert_doremi_fixed(nt.to_string());
            let base_pitch = add_base_and_doremi(base_note, doremi);
            notes.push(base_pitch);
        }
    }
    (notes, mes_end, dur_cnt, diff_vel, base_dur, next_last_nt)
}
fn add_base_and_doremi(base_note: i32, doremi: i32) -> u8 {
    let mut base_pitch: i32;
    if doremi >= NO_MIDI_VALUE as i32 {
        // special meaning ex. NO_NOTE
        base_pitch = doremi;
    } else {
        base_pitch = base_note + doremi;
        if base_pitch >= MAX_NOTE_NUMBER as i32 {
            base_pitch = MAX_NOTE_NUMBER as i32;
        } else if base_pitch < MIN_NOTE_NUMBER as i32 {
            base_pitch = MIN_NOTE_NUMBER as i32;
        }
    }
    return base_pitch as u8;
}
fn gen_dur_info(nt: String, bdur: i32) -> (String, i32, i32) {
    // 階名指定が無く、小節冒頭のタイの場合の音価を判定
    let first_ltr = nt.chars().nth(0).unwrap_or(' ');
    if first_ltr == 'o' {
        return ("".to_string(), bdur, LAST);
    } else if first_ltr == '.' {
        let mut dot_cnt = 0;
        for ltr in nt.chars() {
            if ltr == '.' {
                dot_cnt += 1;
            }
        }
        return ("".to_string(), bdur, dot_cnt);
    }

    //  タイなどの音価を解析し、dur_cnt を確定
    let mut ntext = nt;
    let mut dur_cnt: i32 = 1;
    let txtlen = ntext.len();
    if txtlen > 0 {
        if ntext.chars().nth(txtlen - 1).unwrap_or(' ') == 'o' {
            dur_cnt = LAST;
            ntext.pop();
        } else {
            loop {
                let length = ntext.len();
                if length == 0 {
                    break;
                }
                let ltr = ntext.chars().nth(length - 1).unwrap_or(' ');
                if ltr == '.' || ltr == '~' {
                    dur_cnt += 1;
                    ntext.pop();
                } else {
                    break;
                }
            }
        }
    }

    //  基準音価を解析し、base_dur を確定
    let mut base_dur = bdur;
    let txtlen = ntext.len();
    if txtlen > 0 {
        (ntext, base_dur) = decide_dur(ntext, base_dur);
    }

    (ntext, base_dur, dur_cnt)
}
fn decide_dur(mut ntext: String, mut base_dur: i32) -> (String, i32) {
    let mut triplet: i16 = 0;
    let mut idx = 1;
    let mut fst_ltr = ntext.chars().nth(0).unwrap_or(' ');
    if fst_ltr == '3' || fst_ltr == '5' {
        triplet = fst_ltr.to_digit(10).unwrap_or(1) as i16;
        fst_ltr = ntext.chars().nth(1).unwrap_or(' ');
    }
    if fst_ltr == '\'' || fst_ltr == 'e' {
        if ntext.chars().nth(1).unwrap_or(' ') == '\"' {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 8;
            idx = 2;
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 2;
        }
    } else if fst_ltr == '\"' || fst_ltr == 'v' {
        if ntext.chars().nth(1).unwrap_or(' ') == '\"' {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 16;
            idx = 2;
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 4;
        }
    } else if fst_ltr == 'w' {
        base_dur = DEFAULT_TICK_FOR_QUARTER / 8;
    } else if fst_ltr == 'q' {
        if ntext.chars().nth(1).unwrap_or(' ') == '\'' {
            base_dur = DEFAULT_TICK_FOR_QUARTER * 3 / 2;
            idx = 2;
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER;
        }
    } else if fst_ltr == 'h' {
        if ntext.chars().nth(1).unwrap_or(' ') == '\'' {
            base_dur = DEFAULT_TICK_FOR_QUARTER * 3;
            idx = 2;
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER * 2;
        }
    } else {
        idx = 0;
    }
    if triplet != 0 {
        base_dur = (base_dur * 2) / triplet as i32;
        idx = 2;
    }
    ntext = ntext[idx..].to_string();
    (ntext, base_dur)
}
fn gen_diff_vel(nt: String) -> (String, i32) {
    let mut ntext = nt;
    let mut diff_vel = 0;
    let mut last_ltr = if ntext.len() > 0 {
        ntext.chars().nth(ntext.len() - 1).unwrap_or(' ')
    } else {
        ' '
    };
    while last_ltr == '^' {
        diff_vel += 10;
        ntext.pop();
        last_ltr = if ntext.len() > 0 {
            ntext.chars().nth(ntext.len() - 1).unwrap_or(' ')
        } else {
            ' '
        };
    }
    while last_ltr == '%' {
        diff_vel -= 20;
        ntext.pop();
        last_ltr = if ntext.len() > 0 {
            ntext.chars().nth(ntext.len() - 1).unwrap_or(' ')
        } else {
            ' '
        };
    }
    (ntext, diff_vel)
}
fn get_real_dur(base_dur: i32, dur_cnt: i32, rest_tick: i32) -> i32 {
    if dur_cnt == LAST {
        rest_tick
    } else if base_dur == KEEP {
        base_dur * dur_cnt
    } else {
        base_dur * dur_cnt
    }
}
fn add_note(
    rcmb: Vec<PhrEvt>,
    tick: i32,
    notes: Vec<u8>,
    note_dur: i32,
    last_vel: i16,
    mes_top: bool,
    trns: i16,
) -> Vec<PhrEvt> {
    let mut return_rcmb = rcmb.clone();
    for note in notes.iter() {
        if *note == REST {
            continue;
        } else if *note == NO_NOTE {
            if mes_top {
                // 小節先頭にタイがあった場合、前の音の音価を増やす
                // 前回の入力が '=' による和音入力だった場合も考え、直前の同じタイミングのデータを全て調べる
                let mut search_idx = return_rcmb.len() - 1;
                let last_tick = return_rcmb[search_idx].tick;
                loop {
                    if return_rcmb[search_idx].tick == last_tick {
                        let dur = return_rcmb[search_idx].dur;
                        return_rcmb[search_idx].dur = dur + note_dur as i16;
                    } else {
                        break;
                    }
                    if search_idx == 0 {
                        break;
                    }
                    search_idx -= 1;
                }
            } else {
                continue;
            }
        } else {
            let nt_data = PhrEvt {
                mtype: TYPE_NOTE,
                tick: tick as i16,
                dur: note_dur as i16,
                note: *note as i16,
                vel: last_vel,
                trns,
            };
            //: Vec<i16> =
            //    vec![TYPE_NOTE, tick as i16, note_dur as i16, *note as i16, last_vel];
            return_rcmb.push(nt_data);
        }
    }
    return_rcmb
}

//*******************************************************************
//          convert_doremi
//*******************************************************************
fn convert_doremi_closer(doremi: String, last_nt: i32) -> i32 {
    if doremi.len() == 0 {
        return NO_NOTE as i32;
    }
    let mut last_doremi = last_nt;
    while last_doremi >= 12 {
        last_doremi -= 12;
    }
    while last_doremi < 0 {
        last_doremi += 12;
    }

    let mut oct_pitch = 0;
    let mut pure_doremi = String::from("");
    for (i, ltr) in doremi.chars().enumerate() {
        if ltr == 'x' {
            return REST as i32;
        } else if ltr == '+' {
            oct_pitch += 12;
        } else if ltr == '-' {
            oct_pitch -= 12;
        } else {
            pure_doremi = doremi[i..].to_string();
            break;
        }
    }

    let mut base_note = 0;
    if pure_doremi.len() != 0 {
        base_note = doremi_number(pure_doremi.chars().nth(0).unwrap_or(' '), base_note);
        pure_doremi.remove(0);
    } else {
        return NO_NOTE as i32;
    }

    if pure_doremi.len() != 0 {
        base_note = doremi_semi_number(pure_doremi.chars().nth(0).unwrap_or(' '), base_note);
    }

    let base_pitch: i32;
    if oct_pitch == 0 {
        // +/- が書かれていない場合
        let mut diff = base_note - last_doremi;
        if diff < 0 {
            diff += 12;
        }
        if diff > 6 {
            base_pitch = last_nt + diff - 12;
        } else {
            base_pitch = last_nt + diff;
        }
    } else if oct_pitch > 0 {
        // + 書かれている場合
        while base_note - last_nt >= 12 {
            base_note -= 12;
        }
        while base_note - last_nt <= oct_pitch - 12 {
            base_note += 12;
        }
        base_pitch = base_note;
    } else {
        // - 書かれている場合
        while base_note - last_nt <= -12 {
            base_note += 12;
        }
        while base_note - last_nt >= oct_pitch + 12 {
            base_note -= 12;
        }
        base_pitch = base_note;
    }
    base_pitch
}
fn convert_doremi_fixed(doremi: String) -> i32 {
    if doremi.len() == 0 {
        return NO_NOTE as i32;
    }
    let mut base_pitch: i32 = 0;
    let mut pure_doremi = String::from("");
    for (i, ltr) in doremi.chars().enumerate() {
        if ltr == 'x' {
            return REST as i32;
        } else if ltr == '+' {
            base_pitch += 12;
        } else if ltr == '-' {
            base_pitch -= 12;
        } else {
            pure_doremi = doremi[i..].to_string();
            break;
        }
    }
    if pure_doremi.len() != 0 {
        base_pitch = doremi_number(pure_doremi.chars().nth(0).unwrap_or(' '), base_pitch);
    } else {
        return NO_NOTE as i32;
    }

    if pure_doremi.len() > 1 {
        base_pitch = doremi_semi_number(pure_doremi.chars().nth(1).unwrap_or(' '), base_pitch);
    }
    base_pitch
}
pub fn split_note(txt: String) -> Vec<String> {
    let mut splitted: Vec<String> = Vec::new();
    let mut first_locate: usize = 0;
    let mut pm_flg = false;
    let mut semi_flg = false;
    let mut set_vec = |i: usize| {
        if first_locate < i {
            splitted.push((&txt[first_locate..i]).to_string());
        }
        first_locate = i;
    };
    for (i, ltr) in txt.chars().enumerate() {
        if ltr == '+' || ltr == '-' {
            if !pm_flg {
                set_vec(i);
            }
            pm_flg = true;
            semi_flg = false;
        } else if ltr == 'd'
            || ltr == 'r'
            || ltr == 'm'
            || ltr == 'f'
            || ltr == 's'
            || ltr == 'l'
            || ltr == 't'
        {
            if semi_flg {
                set_vec(i);
            } else if !pm_flg {
                set_vec(i);
            }
            pm_flg = false;
            semi_flg = false;
        } else if ltr == 'i' || ltr == 'a' {
            pm_flg = false;
            semi_flg = true;
        } else if ltr == 'x' {
            return vec!["x".to_string()];
        } else {
            return vec!["".to_string()];
        }
    }
    set_vec(txt.len());
    splitted
}
