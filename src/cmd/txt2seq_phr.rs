//  Created by Hasebe Masahiko on 2023/02/15.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use super::txt2seq_dp::*;
use crate::lpnlib::*;

//*******************************************************************
//          complement_phrase
//*******************************************************************
pub fn complement_phrase(
    input_text: String,
    cluster_word: &str,
) -> (Vec<String>, Vec<String>, Vec<Option<i16>>) {
    // 1. space 削除
    let phr = input_text.trim().to_string();

    // 2. [] を検出し、音符情報と、その他の情報を分け、音符情報はさらに : で分割、auftaktの展開
    let (nttmp, ne) = divide_brackets(phr);
    let ntdiv = split_by(':', nttmp);
    let (nt, ntatrb) = div_atrb(ntdiv);

    // 3. 関数を . で分割し、音符変調と音楽表現に分ける
    let mut nev = split_by('.', ne);
    nev.retain(|nt| !nt.is_empty());
    let (nmvec, nevec) = divide_notemod_and_musicex(nev);

    // 4. <> の検出と、囲まれた要素へのコマンド追加と cluster の展開
    let nttmp = divide_arrow_bracket(nt);
    let nt2 = nttmp.replace('c', cluster_word);

    // 5. ,| 重複による休符指示の補填、()内の ',' を '_' に変換。音符のVector化
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

    (ntvec, nevec, ntatrb)
}
fn divide_brackets(input_text: String) -> (String, String) {
    let mut ninfo = "".to_string();
    let mut minfo = "".to_string();

    // 中身と、その後の文字列を ninfo/minfo に入れる
    let mut isx: &str = &input_text;
    if let Some(n2) = isx.find(']') {
        ninfo = isx[1..n2].to_string();
        isx = &isx[n2 + 1..];
        if !isx.is_empty() {
            minfo = isx.to_string();
        }
    }
    (ninfo, minfo)
}
fn divide_arrow_bracket(nt: String) -> String {
    let mut ret_str: String = "".to_string();
    let mut i = 0;
    while let Some(ltr) = nt.chars().nth(i) {
        if ltr == '<' {
            // 閉じる矢印を探し、その後ろの文字を取得
            if let Some(loc) = nt[i + 1..].chars().position(|c| c == '>') {
                let end_arrow = i + 1 + loc;
                let mut omit = false;
                let mut mark = nt.chars().nth(end_arrow + 1).unwrap_or('~');
                let mut comma = ',';
                if mark == ',' || mark == '/' || mark == '|' {
                    comma = mark;
                    mark = '~';
                    omit = true;
                }
                if mark == 'p' || mark == '!' || mark == '~' || mark == 'n' {
                    for j in i + 1..end_arrow {
                        let nx = nt.chars().nth(j).unwrap_or(' ');
                        if nx == ',' || nx == '|' || nx == '/' {
                            ret_str.push(mark);
                        }
                        if let Some(c) = nt.chars().nth(j) {
                            ret_str.push(c);
                        }
                    }
                    ret_str.push(mark);
                    ret_str.push(comma);
                    if omit {
                        // markがない場合、XX で ',' の分を飛ばす
                        i = end_arrow + 1;
                    } else {
                        // mark + ',' の２文字進める(最後の XX を考慮)
                        i = end_arrow + 2;
                    }
                }
            }
        } else {
            ret_str.push(ltr);
        }
        i += 1; // XX
    }
    //println!("$$$Divided letter in <>: {}", ret_str);
    ret_str
}
fn div_atrb(mut ntdiv: Vec<String>) -> (String, Vec<Option<i16>>) {
    let dnum = ntdiv.len();
    let mut nt = "".to_string();
    let mut ntatrb = vec!["".to_string()];
    let mut atrb: Vec<Option<i16>> = vec![None];
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
            #[cfg(feature = "verbose")]
            println!("Auftakt Start Beat: {beat}");
            if beat > 0 {
                atrb[0] = Some(beat as i16);
                if beat > 1 {
                    let mut rest = String::from("qx");
                    for _ in 0..beat - 2 {
                        rest.push('.')
                    }
                    nt = rest + "," + &nt;
                }
            }
        }
    }

    (nt, atrb)
}
fn fill_omitted_note_data(mut nf: String) -> String {
    let phr_len = nf.len();
    if phr_len == 0 {
        return "".to_string();
    } else if phr_len >= 2 && nf.ends_with("//") {
        nf.pop();
        nf.pop();
        nf += ",LPEND";
    }

    let mut fill: String = "".to_string();
    let mut doremi = "x".to_string();
    let mut doremi_end_flag = true;
    let mut in_parentheses = false;
    for ltr in nf.chars() {
        if ltr == ',' {
            if in_parentheses {
                doremi.push('@'); // ()内の','を、@ に変換
            } else {
                fill += &doremi;
                fill += ",";
                doremi = "x".to_string(); // ,| 連続入力による、休符指示の補填
                doremi_end_flag = true;
            }
        } else if ltr == '|' || ltr == '/' {
            fill += &doremi;
            fill += ",|,";
            doremi = "x".to_string(); // ,| 連続入力による、休符指示の補填
            doremi_end_flag = true;
        } else if doremi_end_flag {
            doremi = (ltr).to_string();
            doremi_end_flag = false;
        } else if ltr == '(' {
            in_parentheses = true;
            doremi.push(ltr);
        } else if ltr == ')' {
            in_parentheses = false;
            doremi.push(ltr);
        } else {
            doremi.push(ltr);
        }
    }
    if !doremi.is_empty() {
        fill += &doremi;
    }
    fill
}
/// Note Modulation Function と Music Expression Function を分離する
fn divide_notemod_and_musicex(nev: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut nm: Vec<String> = Vec::new();
    let mut ne: Vec<String> = Vec::new();

    for nx in nev.iter() {
        if nx.len() >= 3 && &nx[0..3] == "rpt" {
            nm.push(nx.to_string());
        } else {
            ne.push(nx.to_string());
        }
    }
    if ne.is_empty() {
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
    let num;
    if let Some(n) = extract_number_from_parentheses(ne) {
        num = n;
    } else {
        num = 1;
    }
    nnv.extend(nv.clone()); //  repeat前
    for _ in 0..num {
        nnv.push("$RPT".to_string());
        nnv.extend(nv.clone()); // 繰り返しの中身
    }
    nnv
}

//*******************************************************************
///          recombine_to_internal_format
//*******************************************************************
#[derive(Clone, Debug)]
struct AddNoteParam {
    mes_top: bool,
    dur: i32,
    vel: i16,
    trns: TrnsType,
    others: (i16, bool), // (artic, arppeggio)
}
impl Default for AddNoteParam {
    fn default() -> Self {
        AddNoteParam {
            mes_top: false,
            dur: 0,
            vel: 0,
            trns: TrnsType::Com,
            others: (DEFAULT_ARTIC, false),
        }
    }
}
//*******************************************************************
pub fn recombine_to_internal_format(
    ntvec: &[String],
    expvec: &[String],
    imd: InputMode,
    base_note: i32,
    tick_for_onemsr: i32,
) -> (i32, bool, Vec<PhrEvt>) {
    let (exp_vel, _exp_others) = get_dyn_info(expvec.to_vec());
    let mut read_ptr = 0;
    let mut last_nt: i32 = 0;
    let mut crnt_tick: i32 = 0;
    let mut msr: i32 = 1;
    let mut base_dur: i32 = DEFAULT_TICK_FOR_QUARTER;
    let mut rcmb = Vec::new();
    let mut mes_top: bool = false;
    let (max_read_ptr, do_loop) = judge_no_loop(ntvec);
    let mut whole_msr_tick = tick_for_onemsr;
    let mut rest_tick = 0;

    while read_ptr < max_read_ptr {
        let nt_origin = ntvec[read_ptr].clone();
        if nt_origin == "|" {
            // 小節線
            crnt_tick = whole_msr_tick; // 小節頭
            msr += 1;
            whole_msr_tick = tick_for_onemsr * msr; // 次の小節頭
            mes_top = true;
            read_ptr += 1; // out from repeat
            continue;
        }

        // イベント抽出
        rest_tick = whole_msr_tick - crnt_tick;
        let (note_text, trns) = extract_trans_info(nt_origin);
        if note_text == "$RPT" {
            // complement時に入れた、繰り返しを表す特殊マーク$
            let nt_data = PhrEvt::Info(InfoEvt::gen_repeat(crnt_tick as i16));
            rcmb.push(nt_data);
            last_nt = 0; // closed の判断用の前Noteの値をクリアする -> 繰り返し最初の音のオクターブが最初と同じになる
        } else if available_for_dp(&note_text) {
            // Dynamic Pattern
            let (ca_ev, bdur) = treat_dp(
                note_text.clone(),
                base_note,
                base_dur,
                crnt_tick,
                rest_tick,
                exp_vel,
            );
            base_dur = bdur;
            if crnt_tick < whole_msr_tick {
                crnt_tick += ca_ev.dur() as i32;
                rcmb.push(ca_ev);
            }
        } else {
            // Note 処理
            let (notes, note_dur, diff_vel, bdur, lnt, others) =
                break_up_nt_dur_vel(note_text, base_note, base_dur, last_nt, rest_tick, imd);
            last_nt = lnt; // 次回の音程の上下判断のため
            base_dur = bdur;
            if crnt_tick < whole_msr_tick {
                // add to recombined data (NO_NOTE 含む(タイの時に使用))
                let prm = AddNoteParam {
                    mes_top,
                    dur: get_note_dur(note_dur, whole_msr_tick, crnt_tick),
                    vel: velo_limits(exp_vel + diff_vel, 1),
                    trns,
                    others,
                };
                rcmb = add_note(rcmb, crnt_tick, notes, prm);
                crnt_tick += note_dur;
            }
        }
        mes_top = false;
        read_ptr += 1; // out from repeat
    }
    if rest_tick > 0 {
        crnt_tick = whole_msr_tick; // 小節線を超えていれば、次の小節の頭までをwhole_tickとする
    }
    (crnt_tick, do_loop, rcmb)
}
fn judge_no_loop(ntvec: &[String]) -> (usize, bool) {
    let mut max_read_ptr = ntvec.len();
    // LPENDの検出
    let do_loop = !ntvec.ends_with(&["LPEND".to_string()]);
    if !do_loop {
        max_read_ptr -= 1;
    }
    (max_read_ptr, do_loop)
}
fn get_dyn_info(expvec: Vec<String>) -> (i32, Vec<String>) {
    let mut vel = END_OF_DATA;
    let mut retvec = expvec.clone();
    for (i, txt) in expvec.iter().enumerate() {
        if txt.len() >= 3 && &txt[0..3] == "dyn" {
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
fn extract_trans_info(origin: String) -> (String, TrnsType) {
    if origin.len() > 2 && &origin[0..2] == ">>" {
        (origin[2..].to_string(), TrnsType::NoTrns)
    } else if &origin[0..1] == ">" {
        (origin[1..].to_string(), TrnsType::Para)
    } else {
        (origin, TrnsType::Com)
    }
}
/// カンマで区切られた単位の文字列を解析し、ノート番号、tick、velocity を確定する
fn break_up_nt_dur_vel(
    note_text: String, // 分析対象のテキスト
    base_note: i32,    // そのパートの基準音
    bdur: i32,         // 現在の基準音価
    last_nt: i32,      // 前回の音程
    rest_tick: i32,    // 小節の残りtick
    imd: InputMode,    // input mode
) -> (Vec<u8>, i32, i32, i32, i32, (i16, bool))
/*( notes,      // 発音ノート
    dur_tick,    // 音符のtick数
    diff_vel,   // 音量情報
    base_dur,   // 基準音価 -> bdur
    last_nt,    // 次回判定用の今回の音程 -> last_nt
    artic       // アーティキュレーション情報
  )*/
{
    //  頭にOctave記号(+-)があれば、一度ここで抜いておいて、解析を終えたら文字列を再結合
    let mut ntext1 = note_text;
    let oct = extract_top_pm(&mut ntext1);

    //  duration 情報、 Velocity 情報の抽出
    let (ntext3, base_dur, dur_tick, artic) = gen_dur_info(ntext1, bdur, rest_tick);
    let (mut ntext4, diff_vel) = gen_diff_vel(ntext3);

    // 複数音がアルペジオか判断、各音を分離してベクトル化
    let arp = if ntext4.starts_with("$") {
        ntext4.remove(0);
        true
    } else {
        false
    };
    let ntext5 = format!("{}{}", oct, &ntext4); // +-の再結合
    let notes_vec = split_notes(ntext5.clone());

    // 階名への変換
    let mut notes: Vec<u8> = Vec::new();
    let mut next_last_nt = last_nt;
    let mut first_note: Option<i32> = None; // 和音の最初の音程を記録するため
    for (i, nt) in notes_vec.iter().enumerate() {
        let doremi: i32;
        match imd {
            InputMode::Fixed => {
                doremi = convert_doremi_fixed(nt.to_string());
                if i > 0 {
                    break;
                }
            }
            InputMode::Closer => {
                if i == 0 {
                    doremi = convert_doremi_closer(nt.to_string(), next_last_nt);
                    if doremi < NO_MIDI_VALUE as i32 {
                        first_note = Some(doremi);
                    }
                } else {
                    doremi = convert_doremi_upper_closer(nt.to_string(), next_last_nt);
                }
            }
            InputMode::Upcloser => {
                doremi = convert_doremi_upper_closer(nt.to_string(), next_last_nt);
                if i == 0 && doremi < NO_MIDI_VALUE as i32 {
                    first_note = Some(doremi);
                }
            }
        }
        if doremi < NO_MIDI_VALUE as i32 {
            next_last_nt = doremi;
        }
        notes.push(add_base_and_doremi(base_note, doremi));
    }

    // 何も音名が入らなかった時
    if notes.is_empty() {
        notes.push(NO_NOTE);
    } else if notes.len() > 1 && first_note.is_some() {
        // Upcloser + 和音の時、最低音を next_last_nt とする
        next_last_nt = first_note.unwrap();
    }

    (
        notes,
        dur_tick,
        diff_vel,
        base_dur,
        next_last_nt,
        (artic, arp),
    )
}
/// 文字列の冒頭にあるプラスマイナスを抽出
fn extract_top_pm(ntext: &mut String) -> String {
    let mut oct = "".to_string();
    loop {
        let c = ntext.chars().next().unwrap_or(' ');
        if c == '+' {
            oct.push('+');
            ntext.remove(0);
        } else if c == '-' {
            oct.push('-');
            ntext.remove(0);
        } else {
            break;
        }
    }
    oct
}
fn add_base_and_doremi(base_note: i32, doremi: i32) -> u8 {
    let mut base_pitch = doremi;
    if doremi < NO_MIDI_VALUE as i32 {
        // special meaning ex. NO_NOTE
        base_pitch = base_note + doremi;
    }
    base_pitch as u8
}
/// 音価情報を生成
fn gen_dur_info(mut ntext1: String, bdur: i32, rest_tick: i32) -> (String, i32, i32, i16) {
    //  Articulation 情報の抽出
    let mut artic: i16 = DEFAULT_ARTIC;
    if let Some(e) = ntext1.chars().last() {
        if e == '~' {
            artic = 120;
            ntext1.pop();
        } else if e == '!' {
            artic = 50;
            ntext1.pop();
        }
    }

    // 階名指定が無く、小節冒頭のタイの場合の音価を判定
    let (no_nt, ret) = detect_measure_top_tie(ntext1.clone(), bdur, rest_tick);
    if no_nt {
        return (ret.0, ret.1, ret.2, artic);
    }

    // 音価伸ばしを解析し、dur_cnt を確定
    let (ntext1, dur_cnt) = extract_o_dot(ntext1.clone());
    if dur_cnt == LAST {
        return (ntext1, bdur, rest_tick, artic);
    }

    // タイを探して追加する tick を算出
    let (tie_dur, bdur_tie, ntext2) = decide_tie_dur(ntext1);

    //  基準音価を解析し、base_dur を確定
    let mut nt: String = ntext2.clone();
    let mut base_dur: i32 = bdur;
    if !ntext2.is_empty() {
        (nt, base_dur) = decide_dur(ntext2, bdur);
    }
    let tick = base_dur * dur_cnt + tie_dur;

    if bdur_tie != 0 {
        base_dur = bdur_tie
    }
    (nt, base_dur, tick, artic)
}
fn detect_measure_top_tie(nt: String, bdur: i32, rest_tick: i32) -> (bool, (String, i32, i32)) {
    // 階名指定が無く、小節冒頭のタイの場合の音価を判定
    let first_ltr = nt.chars().next().unwrap_or(' ');
    if first_ltr == 'o' {
        return (true, ("".to_string(), bdur, rest_tick));
    } else if first_ltr == '.' {
        let mut dot_cnt = 0;
        for ltr in nt.chars() {
            if ltr == '.' {
                dot_cnt += 1;
            }
        }
        return (true, ("".to_string(), bdur, bdur * dot_cnt));
    } else if first_ltr == '_' {
        let mut tie_dur: i32 = 0;
        let tie = nt[1..].to_string();
        let mut _ntt: String = "".to_string();
        if !tie.is_empty() {
            (_ntt, tie_dur) = decide_dur(tie, 0);
        }
        return (true, ("".to_string(), tie_dur, tie_dur));
    }
    (false, (nt, bdur, rest_tick))
}
fn extract_o_dot(nt: String) -> (String, i32) {
    let mut ntext = nt;
    let mut dur_cnt: i32 = 1;
    if !ntext.is_empty() {
        if let Some('o') = ntext.chars().last() {
            dur_cnt = LAST;
            ntext.pop();
        } else {
            loop {
                if ntext.is_empty() {
                    break;
                }
                if let Some('.') = ntext.chars().last() {
                    dur_cnt += 1;
                    ntext.pop();
                } else {
                    break;
                }
            }
        }
    }
    (ntext, dur_cnt)
}
pub fn decide_tie_dur(ntext1: String) -> (i32, i32, String) {
    let mut tie_dur: i32 = 0;
    let mut rest_str = ntext1;
    let mut bdur_tie: i32 = 0;
    while let Some(num) = rest_str.rfind('_') {
        let tie = rest_str[num + 1..].to_string();
        if !tie.is_empty() {
            let (_ntt, tdur) = decide_dur(tie, 0);
            tie_dur += tdur;
            if bdur_tie == 0 {
                bdur_tie = tdur; // 最後のタイの音価を記録
            }
            rest_str = rest_str[0..num].to_string();
        } else {
            break;
        }
    }
    (tie_dur, bdur_tie, rest_str)
}
pub fn decide_dur(ntext: String, mut base_dur: i32) -> (String, i32) {
    let mut triplet: i16 = 0;
    let mut idx = 1;
    let mut fst_ltr = ntext.chars().next().unwrap_or(' ');
    if fst_ltr == '3' || fst_ltr == '5' {
        triplet = fst_ltr.to_digit(10).unwrap_or(1) as i16;
        fst_ltr = ntext.chars().nth(1).unwrap_or(' ');
    }
    if fst_ltr == '\'' || fst_ltr == 'e' {
        if ntext.chars().nth(1).unwrap_or(' ') == '\'' {
            base_dur = DEFAULT_TICK_FOR_QUARTER * 3 / 4;
            idx = 2;
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 2;
        }
    } else if fst_ltr == '\"' || fst_ltr == 'v' {
        if ntext.chars().nth(1).unwrap_or(' ') == '\'' {
            base_dur = DEFAULT_TICK_FOR_QUARTER * 3 / 8;
            idx = 2;
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 4;
        }
    } else if fst_ltr == 'w' {
        if ntext.chars().nth(1).unwrap_or(' ') == '(' {
            if let Some(p) = ntext.find(')') {
                let dur_str = &ntext[2..p];
                if let Ok(dur) = dur_str.parse::<i32>() {
                    base_dur = dur;
                } else {
                    base_dur = DEFAULT_TICK_FOR_QUARTER / 8;
                }
                idx = p + 1;
            } else {
                base_dur = DEFAULT_TICK_FOR_QUARTER / 8;
            }
        } else {
            base_dur = DEFAULT_TICK_FOR_QUARTER / 8;
        }
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
    let nt = ntext[idx..].to_string();
    (nt, base_dur)
}
pub fn gen_diff_vel(nt: String) -> (String, i32) {
    let mut ntext = nt;
    let mut diff_vel = 0;
    let mut last_ltr = if !ntext.is_empty() {
        ntext.chars().nth(ntext.len() - 1).unwrap_or(' ')
    } else {
        ' '
    };
    while last_ltr == '^' {
        diff_vel += VEL_UP;
        ntext.pop();
        last_ltr = if ntext.is_empty() {
            ' '
        } else {
            ntext.chars().last().unwrap_or(' ')
        };
    }
    while last_ltr == '%' {
        diff_vel += VEL_DOWN;
        ntext.pop();
        last_ltr = if ntext.is_empty() {
            ' '
        } else {
            ntext.chars().last().unwrap_or(' ')
        };
    }
    (ntext, diff_vel)
}
fn add_note(rcmb: Vec<PhrEvt>, tick: i32, notes: Vec<u8>, prm: AddNoteParam) -> Vec<PhrEvt> {
    let mut return_rcmb = rcmb.clone();
    match notes.len() {
        0 => (),
        1 => {
            match notes[0] {
                REST => (),
                NO_NOTE => {
                    // タイの時は、音符の音価を増やすので、ここでは何もしない
                    //return_rcmb.push(PhrEvt::NoNote(NoNoteEvt { tick: tick as i16, dur: prm.dur as i16 }));
                    let l = return_rcmb.len();
                    if prm.mes_top && l > 0 {
                        // 小節先頭にタイがあった場合、前の音の音価を増やす
                        // 前回の入力が和音入力だった場合も考え、直前の同じタイミングのデータを全て調べる
                        let mut search_idx = l - 1;
                        let last_tick = return_rcmb[search_idx].tick();
                        loop {
                            if return_rcmb[search_idx].tick() == last_tick {
                                let dur = return_rcmb[search_idx].dur();
                                return_rcmb[search_idx].set_dur(dur + prm.dur as i16);
                                //return_rcmb[search_idx].vel = prm.vel; // タイの場合、前の音符の音量を使う
                                return_rcmb[search_idx].set_artic(prm.others.0);
                            } else {
                                break;
                            }
                            if search_idx == 0 {
                                break;
                            }
                            search_idx -= 1;
                        }
                    }
                }
                _ => {
                    // 単音の入力
                    let note_data = PhrEvt::Note(NoteEvt {
                        tick: tick as i16,
                        dur: prm.dur as i16,
                        note: notes[0],
                        floating: prm.others.1,
                        vel: prm.vel,
                        trns: prm.trns,
                        artic: prm.others.0,
                    });
                    return_rcmb.push(note_data);
                }
            }
        }
        _ => {
            // 和音の入力
            let note_data = PhrEvt::NoteList(NoteListEvt {
                tick: tick as i16,
                dur: prm.dur as i16,
                notes: notes.clone(),
                floating: prm.others.1,
                vel: prm.vel,
                trns: prm.trns,
                artic: prm.others.0,
            });
            return_rcmb.push(note_data);
        }
    }
    return_rcmb
}
fn get_note_dur(ndur: i32, whole_msr_tick: i32, crnt_tick: i32) -> i32 {
    let mut note_dur = ndur;
    if whole_msr_tick - crnt_tick < note_dur {
        note_dur = whole_msr_tick - crnt_tick; // 小節線を超えたら、音価をそこでリミット
    }
    note_dur
}

//*******************************************************************
//          convert_doremi
//*******************************************************************
/// 最も近い上側の音を選択
fn convert_doremi_upper_closer(doremi: String, last_nt: i32) -> i32 {
    if doremi.is_empty() {
        return NO_NOTE as i32;
    }
    let last_doremi = get_pure_doremi(last_nt);

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

    let mut base_note = doremi_to_notenum(pure_doremi, 0);
    if last_doremi > base_note {
        base_note += 12;
    }
    last_nt - last_doremi + base_note + oct_pitch //return
}
/// 最も近い音を選択
fn convert_doremi_closer(doremi: String, last_nt: i32) -> i32 {
    if doremi.is_empty() {
        return NO_NOTE as i32;
    }
    let last_doremi = get_pure_doremi(last_nt);

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

    let base_note = doremi_to_notenum(pure_doremi, 0);
    let mut diff = base_note - last_doremi;
    if diff <= -6 {
        diff += 12;
    } else if diff > 6 {
        diff -= 12;
    }
    last_nt + diff + oct_pitch // return
}
/// 絶対音高による指定
fn convert_doremi_fixed(doremi: String) -> i32 {
    if doremi.is_empty() {
        return NO_NOTE as i32;
    }
    let mut base_note: i32 = 0;
    let mut pure_doremi = String::from("");
    for (i, ltr) in doremi.chars().enumerate() {
        if ltr == 'x' {
            return REST as i32;
        } else if ltr == '+' {
            base_note += 12;
        } else if ltr == '-' {
            base_note -= 12;
        } else {
            pure_doremi = doremi[i..].to_string();
            break;
        }
    }
    doremi_to_notenum(pure_doremi, base_note)
}
pub fn split_notes(txt: String) -> Vec<String> {
    let mut splitted: Vec<String> = Vec::new();
    let mut first_locate: usize = 0;
    let mut plus_flg = false;
    let mut semi_flg = false;
    let mut set_vec = |i: usize| {
        if first_locate < i {
            splitted.push(txt[first_locate..i].to_string());
        }
        first_locate = i;
    };
    for (i, ltr) in txt.chars().enumerate() {
        if ltr == '+' || ltr == '-' {
            if !plus_flg {
                set_vec(i);
            }
            plus_flg = true;
            semi_flg = false;
        } else if ltr == 'd'
            || ltr == 'r'
            || ltr == 'm'
            || ltr == 'f'
            || ltr == 's'
            || ltr == 'l'
            || ltr == 't'
        {
            if semi_flg || !plus_flg {
                set_vec(i);
            }
            plus_flg = false;
            semi_flg = false;
        } else if ltr == 'i' || ltr == 'a' {
            plus_flg = false;
            semi_flg = true;
        } else if ltr == 'x' {
            return vec!["x".to_string()];
        } else {
            set_vec(i);
            return splitted;
        }
    }
    set_vec(txt.len());
    splitted
}
