//  Created by Hasebe Masahiko on 2023/02/15.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt_common::*;
use super::txt2seq_dp;
use crate::lpnlib::*;

//*******************************************************************
//          complement_phrase
//*******************************************************************
#[derive(Debug)]
pub struct PhraseComplemented {
    pub note_str: String,                 // []内
    pub exp_str: String,                  // []の後ろ
    pub note_attribute: Vec<Option<i16>>, // Auftakt などの属性
    pub note_info: Vec<String>,
    pub note_mod: Vec<String>,           // Note Modulation Function
    pub music_exp: Vec<String>,          // Music Expression Function
    pub accia_info: Vec<Option<String>>, // 装飾音符の情報
}

impl PhraseComplemented {
    pub fn new() -> Self {
        PhraseComplemented {
            note_str: String::new(),
            exp_str: String::new(),
            note_attribute: vec![None],
            note_info: Vec::new(),
            note_mod: Vec::new(),
            music_exp: Vec::new(),
            accia_info: Vec::new(),
        }
    }
    pub fn divide_brackets(&mut self, input_text: String) {
        // 中身と、その後の文字列を note_str/exp_str に入れる
        let mut isx: &str = &input_text;
        if let Some(n2) = isx.find(']') {
            self.note_str = isx[1..n2].to_string();
            isx = &isx[n2 + 1..];
            if !isx.is_empty() {
                self.exp_str = isx.to_string();
            }
        }
    }
    fn divide_atrb(&mut self, mut ntdiv: Vec<String>) {
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

        self.note_str = nt;
        self.note_attribute = atrb;
    }
    /// Note Modulation Function と Music Expression Function を分離する
    fn divide_notemod_and_musicex(&mut self, nev: Vec<String>) {
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
        //(nm, ne)
        self.note_mod = nm;
        self.music_exp = ne;
    }
    fn divide_arrow_bracket(&mut self) {
        let nt = self.note_str.clone();
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
        self.note_str = ret_str;
    }
    /// ,| 重複による休符指示の補填、()内の ',' を '@' に変換
    fn fill_omitted_note_data(&mut self) {
        let mut dt = self.note_str.clone();
        let phr_len = dt.len();
        if phr_len == 0 {
            self.note_str = "".to_string();
            return;
        } else if phr_len >= 2 && dt.ends_with("//") {
            dt.pop();
            dt.pop();
            dt += ",LPEND";
        }

        let mut fill: String = "".to_string();
        let mut doremi = "x".to_string();
        let mut doremi_end_flag = true;
        let mut in_parentheses = false;
        for ltr in dt.chars() {
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
            } else {
                doremi.push(ltr);
            }
            if ltr == '(' {
                in_parentheses = true;
            } else if ltr == ')' {
                in_parentheses = false;
            }
        }
        if !doremi.is_empty() {
            fill += &doremi;
        }
        self.note_str = fill;
        //println!("+++ Note String:  {}", self.note_str);
    }
    /// *n の繰り返しを展開する(acciaccatura も同様に展開)
    fn note_repeat(&mut self) {
        // 単一パスで '*n' を展開する実装
        let mut out: Vec<String> = Vec::with_capacity(self.note_info.len());
        for note in self.note_info.iter() {
            if let Some(pos) = note.find('*') {
                // '*' の前までを基底ノート文字列とする
                let base = note[..pos].to_string();
                let number: i32 = note[pos + 1..].parse().unwrap_or(0);
                // 最低1回は出力（既存の1つ分）。number>1 の場合は追加分を出す
                out.push(base.clone());
                if number > 1 {
                    // +- 指示がある場合は、2回目以降先頭の1文字を削除しておく（UTF-8 安全）
                    let base = if base.starts_with('-') || base.starts_with('+') {
                        base.chars().skip(1).collect::<String>()
                    } else {
                        base
                    };
                    for _ in 0..number - 1 {
                        out.push(base.clone());
                    }
                }
            } else {
                out.push(note.clone());
            }
        }
        self.note_info = out;
    }
    /// 同じ Phrase を指定回数回、コピーし追加する(acciaccatura も同様に展開)
    fn repeat_ntimes(&mut self, ne: &str) {
        let mut note_vec: Vec<String> = Vec::new();
        let num;
        if let Some(n) = extract_number_from_parentheses(ne) {
            num = n;
        } else {
            num = 1;
        }
        note_vec.extend(self.note_info.clone()); //  repeat前
        for _ in 0..num {
            note_vec.push("$RPT".to_string());
            note_vec.extend(self.note_info.clone()); // 繰り返しの中身
        }
        self.note_info = note_vec;
    }
    /// 装飾音符 (acciaccatura: アッチャッカトゥーラ) を分離する
    fn divide_acciaccatura(&mut self) {
        for nt in self.note_info.iter_mut() {
            let mut at: Option<String> = None; // 装飾音符なし
            if nt.starts_with('(') {
                // 装飾音符あり
                let mut temp = nt.clone();
                if let Some(end) = temp.find(')') {
                    at = Some(temp.drain(0..=end).collect());
                }
            }
            let acc_start = at.as_ref().map(|s| s.len()).unwrap_or(0);
            self.accia_info.push(at);
            *nt = nt[acc_start..].to_string(); // 装飾音符を削除
        }
        //println!("+++ Acciaccatura String:  {:?}", self.accia_info);
    }
    fn get_origin(&self, idx: usize) -> (Option<String>, Option<String>) {
        if idx < self.note_info.len() {
            let nt = self.note_info[idx].clone();
            let at = if idx < self.accia_info.len() {
                self.accia_info[idx].clone()
            } else {
                None
            };
            (Some(nt), at)
        } else {
            (None, None)
        }
    }
}
//*******************************************************************
pub fn complement_phrase(input_text: String, cluster_word: &str) -> Box<PhraseComplemented> {
    let mut pc = Box::new(PhraseComplemented::new());

    // 1. space 削除し、[] を検出して、音符情報と、その他の情報を分ける
    let phr = input_text.trim().to_string();
    pc.divide_brackets(phr);

    // 2. 音符情報はさらに : で分割、auftaktの展開、装飾音符の分離
    let ntdiv = split_by(':', pc.note_str.clone());
    pc.divide_atrb(ntdiv.clone());

    // 3. 関数を . で分割し、音符変調と音楽表現に分ける
    let mut nev = split_by('.', pc.exp_str.clone());
    nev.retain(|nt| !nt.is_empty());
    pc.divide_notemod_and_musicex(nev);

    // 4. <> の検出と、囲まれた要素へのコマンド追加と cluster の展開
    pc.divide_arrow_bracket();
    pc.note_str = pc.note_str.replace('c', cluster_word);

    // 5. ,| 重複による休符指示の補填、()内の ',' を '_' に変換。音符のVector化
    pc.fill_omitted_note_data();
    pc.note_info = split_by(',', pc.note_str.clone());

    // 6. 同音繰り返しの展開、同フレーズの繰り返し展開
    pc.note_repeat();
    let note_mod = pc.note_mod.clone();
    for ne in note_mod.iter() {
        if &ne[0..3] == "rpt" {
            pc.repeat_ntimes(ne);
        }
    }

    // 7. 装飾音符を分離する
    pc.divide_acciaccatura();

    pc
}

//*******************************************************************
///          recombine_to_internal_format
//*******************************************************************
#[derive(Clone, Debug)]
struct AddNoteParam {
    mes_top: bool,
    dur: i32,
    vel: i16,
    amp: Amp,
    trns: TrnsType,
    others: (i16, bool), // (artic, arppeggio)
}
impl Default for AddNoteParam {
    fn default() -> Self {
        AddNoteParam {
            mes_top: false,
            dur: 0,
            vel: 0,
            amp: Amp::default(),
            trns: TrnsType::Com,
            others: (DEFAULT_ARTIC, false),
        }
    }
}
//*******************************************************************
#[derive(Debug)]
pub struct PhraseRecombined {
    pub rcmb: Vec<PhrEvt>,
    pub last_nt: i32,
    pub base_dur: i32,
    pub read_ptr: usize,
    pub msr: i32,
    pub whole_msr_tick: i32,
    tick_for_onemsr: i32,
    base_note: i32,
}
impl PhraseRecombined {
    const ACCIACCATURA_LENGTH: i16 = 60;
    fn new(tick_for_onemsr: i32, base_note: i32) -> Self {
        PhraseRecombined {
            rcmb: Vec::new(),
            last_nt: 0,
            base_dur: DEFAULT_TICK_FOR_QUARTER,
            read_ptr: 0,
            msr: 1,
            whole_msr_tick: tick_for_onemsr,
            tick_for_onemsr,
            base_note,
        }
    }
    fn get_and_inc_read_ptr(&mut self) -> usize {
        let r = self.read_ptr;
        self.read_ptr += 1;
        r
    }
    fn update_crnt_tick(&mut self) -> i32 {
        let crnt_tick = self.whole_msr_tick; // 小節頭
        self.whole_msr_tick = self.tick_for_onemsr * self.new_msr(); // 次の小節頭
        crnt_tick
    }
    fn whole_msr_tick(&self) -> i32 {
        self.whole_msr_tick
    }
    pub fn rest_tick(&self, crnt_tick: i32) -> i32 {
        self.whole_msr_tick - crnt_tick
    }
    fn is_less_than_whole_tick(&self, crnt_tick: i32) -> bool {
        crnt_tick < self.whole_msr_tick
    }
    fn adjust_crnt_tick(&self, crnt_tick: i32) -> i32 {
        if crnt_tick < self.whole_msr_tick {
            // 小節線を超えていれば、次の小節の頭までをwhole_tickとする
            self.whole_msr_tick
        } else {
            crnt_tick
        }
    }
    fn new_msr(&mut self) -> i32 {
        self.msr += 1;
        self.msr
    }
    /// カンマで区切られた単位の文字列を解析し、ノート番号、tick、velocity を確定する
    fn break_up_nt_dur_vel(
        &mut self,
        note_text: String, // 分析対象のテキスト
        crnt_tick: i32,    // 現在の tick
        imd: InputMode,    // input mode
    ) -> (Vec<u8>, i32, i32, i16, (i16, bool)) /*
    (   notes,      // 発音ノート
        dur_tick,   // 音符のtick数
        diff_vel,   // 音量情報
        diff_amp,   // 音量情報
        artic       // アーティキュレーション情報
    )*/ {
        let rest_tick = self.rest_tick(crnt_tick);
        //  頭にOctave記号(+-)があれば、一度ここで抜いておいて、解析を終えたら文字列を再結合
        let mut ntext1 = note_text;
        let oct = extract_top_pm(&mut ntext1);

        //  duration 情報、 Velocity 情報の抽出
        let (ntext3, base_dur, dur_tick, artic) = gen_dur_info(ntext1, self.base_dur, rest_tick);
        let (mut ntext4, diff_vel, diff_amp) = extrude_diff_vel(ntext3);

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
        let mut next_last_nt = self.last_nt;
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
            notes.push(add_base_and_doremi(self.base_note, doremi));
        }

        // 何も音名が入らなかった時
        if notes.is_empty() {
            notes.push(NO_NOTE);
        } else if notes.len() > 1 && first_note.is_some() {
            // Upcloser + 和音の時、最低音を next_last_nt とする
            next_last_nt = first_note.unwrap();
        }
        self.last_nt = next_last_nt; // 次回の音程の上下判断のため
        self.base_dur = base_dur; // 次回の音符のために保存しておく
        (notes, dur_tick, diff_vel, diff_amp, (artic, arp))
    }
    /// 音符を指定して、Recombine に追加する
    fn add_note(&mut self, tick: i32, notes: Vec<u8>, prm: AddNoteParam, accia: Option<String>) {
        //let mut return_rcmb = rcmb.clone();
        match notes.len() {
            0 => (),
            1 => {
                match notes[0] {
                    REST => (),
                    NO_NOTE => {
                        // 小節先頭にタイがあった場合、前の音の音価を増やす
                        self.modify_last_note(&prm);
                    }
                    _ => {
                        // 単音の入力
                        let note_data = PhrEvt::Note(NoteEvt {
                            tick: tick as i16,
                            dur: prm.dur as i16,
                            note: notes[0],
                            floating: prm.others.1,
                            vel: prm.vel,
                            amp: prm.amp,
                            trns: prm.trns,
                            artic: prm.others.0,
                        });
                        if let Some(accia_str) = &accia {
                            self.add_accia_note(accia_str, &note_data);
                        }
                        self.rcmb.push(note_data);
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
                    amp: prm.amp,
                    trns: prm.trns,
                    artic: prm.others.0,
                });
                if let Some(accia_str) = &accia {
                    self.add_accia_note(accia_str, &note_data);
                }
                self.rcmb.push(note_data);
            }
        }
        //return_rcmb
    }
    fn modify_last_note(&mut self, prm: &AddNoteParam) {
        let l = self.rcmb.len();
        if prm.mes_top && l > 0 {
            // 前回の入力が和音入力だった場合も考え、直前の同じタイミングのデータを全て調べる
            let mut search_idx = l - 1;
            let last_tick = self.rcmb[search_idx].tick();
            loop {
                if self.rcmb[search_idx].tick() == last_tick {
                    let dur = self.rcmb[search_idx].dur();
                    self.rcmb[search_idx].set_dur(dur + prm.dur as i16);
                    //self.rcmb[search_idx].vel = prm.vel; // タイの場合、前の音符の音量を使う
                    self.rcmb[search_idx].set_artic(prm.others.0);
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
    fn add_accia_note(&mut self, accia_str: &str, note_data: &PhrEvt) {
        // 装飾音符の入力
        let accia_str = extract_texts_from_parentheses(accia_str);
        let accia_notes = accia_str.split('@').collect::<Vec<&str>>();
        let acnum = accia_notes.len() as i16;
        for (i, &nt) in accia_notes.iter().enumerate() {
            if nt.is_empty() {
                continue;
            }
            let accia_value = nt.parse().unwrap_or(0);
            let mut accia_note = note_data.clone();
            accia_note.set_dur(Self::ACCIACCATURA_LENGTH);
            accia_note.set_note((note_data.note() as i8 + accia_value) as u8);
            accia_note.set_tick(accia_note.tick() - Self::ACCIACCATURA_LENGTH * (acnum - i as i16));
            //println!("Added accia note: {:?}", nt);
            self.rcmb.push(accia_note);
        }
    }
}
//*******************************************************************
pub fn recombine_to_internal_format(
    pc: &PhraseComplemented,
    imd: InputMode,
    base_note: i32,
    tick_for_onemsr: i32,
) -> (i32, bool, Vec<PhrEvt>) {
    let mut pr = PhraseRecombined::new(tick_for_onemsr, base_note);
    let (exp_vel, exp_amp, _exp_others) = get_dyn_info(pc.music_exp.clone());
    let mut crnt_tick: i32 = 0;
    let mut mes_top: bool = false;
    let mut do_loop = true;

    loop {
        let (nt_val, accia_val) = pc.get_origin(pr.get_and_inc_read_ptr());
        if nt_val.is_none() {
            break;
        }
        let nt = nt_val.unwrap();
        if nt == "|" {
            // 小節線
            crnt_tick = pr.update_crnt_tick();
            mes_top = true;
            continue;
        }
        if nt == "LPEND" {
            // 繰り返しなしを示す終端
            do_loop = false;
            break;
        }
        let nt_origin = nt;

        // イベント抽出
        let (note_text, trns) = extract_trans_info(nt_origin);
        if note_text == "$RPT" {
            // complement時に入れた、繰り返しを表す特殊マーク$
            let nt_data = PhrEvt::Info(InfoEvt::gen_repeat(crnt_tick as i16));
            pr.rcmb.push(nt_data);
            pr.last_nt = 0; // closed の判断用の前Noteの値をクリアする -> 繰り返し最初の音のオクターブが最初と同じになる
        } else if txt2seq_dp::available_for_dp(&note_text) {
            // Dynamic Pattern
            let ca_ev =
                txt2seq_dp::treat_dp(&mut pr, note_text.clone(), base_note, crnt_tick, exp_vel, exp_amp);
            if pr.is_less_than_whole_tick(crnt_tick) {
                crnt_tick += ca_ev.dur() as i32;
                pr.rcmb.push(ca_ev);
            }
        } else {
            // Note 処理
            let (notes, note_dur, diff_vel, diff_amp, others) =
                pr.break_up_nt_dur_vel(note_text, crnt_tick, imd);
            let amp = Amp { note_amp: diff_amp, phrase_amp: exp_amp };
            if pr.is_less_than_whole_tick(crnt_tick) {
                // add to recombined data (NO_NOTE 含む(タイの時に使用))
                let prm = AddNoteParam {
                    mes_top,
                    dur: get_note_dur(note_dur, pr.whole_msr_tick(), crnt_tick),
                    vel: velo_limits(exp_vel + diff_vel, 1),
                    amp,
                    trns,
                    others,
                };
                pr.add_note(crnt_tick, notes, prm, accia_val);
                crnt_tick += note_dur;
            }
        }
        mes_top = false;
    }

    (pr.adjust_crnt_tick(crnt_tick), do_loop, pr.rcmb)
}
fn get_dyn_info(expvec: Vec<String>) -> (i32, i16, Vec<String>) {
    let mut vel = END_OF_DATA;
    let mut amp = 0;
    let mut retvec = expvec.clone();
    for (i, txt) in expvec.iter().enumerate() {
        if txt.len() >= 3 && &txt[0..3] == "dyn" {
            let dyntxt = extract_texts_from_parentheses(txt);
            vel = convert_exp2vel(dyntxt);
            amp = convert_expstr2amp(dyntxt);
            if vel != END_OF_DATA {
                retvec.remove(i);
                break;
            }
        }
    }
    if vel == END_OF_DATA {
        vel = convert_exp2vel("p");
    }
    (vel, amp, retvec)
}
fn extract_trans_info(origin: String) -> (String, TrnsType) {
    if origin.len() > 2 && &origin[0..2] == ">>" {
        (origin[2..].to_string(), TrnsType::NoTrns)
    } else if !origin.is_empty() && &origin[0..1] == ">" {
        (origin[1..].to_string(), TrnsType::Para)
    } else {
        (origin, TrnsType::Com)
    }
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
pub fn extrude_diff_vel(nt: String) -> (String, i32, i16) {
    let mut ntext = nt;
    let mut diff_vel = 0;
    let mut diff_amp = 0;
    let mut last_ltr = if !ntext.is_empty() {
        ntext.chars().nth(ntext.len() - 1).unwrap_or(' ')
    } else {
        ' '
    };
    while last_ltr == '^' {
        diff_vel += VEL_UP;
        diff_amp += 4;
        ntext.pop();
        last_ltr = if ntext.is_empty() {
            ' '
        } else {
            ntext.chars().last().unwrap_or(' ')
        };
    }
    while last_ltr == '%' {
        diff_vel += VEL_DOWN;
        diff_amp -= 4;
        ntext.pop();
        last_ltr = if ntext.is_empty() {
            ' '
        } else {
            ntext.chars().last().unwrap_or(' ')
        };
    }
    (ntext, diff_vel, diff_amp)
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
    for (i, ltr) in doremi.char_indices() {
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
    for (i, ltr) in doremi.char_indices() {
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
    for (i, ltr) in doremi.char_indices() {
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
    for (i, ltr) in txt.char_indices() {
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
