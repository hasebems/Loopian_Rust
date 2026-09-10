//  Created by Hasebe Masahiko on 2023/02/14.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::txt2seq_ana::*;
use super::txt2seq_cmps::*;
use super::txt2seq_pdl::*;
use super::txt2seq_phr::*;
use crate::common::lpnlib::*;
use crate::common::txt_common::*;

//*******************************************************************
//          Seq Data Stock Struct
//*******************************************************************
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseCmdError {
    InvalidPart,
    InvalidPhrase,
    /// 保留中の []+ バッファと異なる、明示的な宛先(Part./@n=/@msr()=)を
    /// 伴う「閉じ」入力が来た場合のエラー。保留中バッファは破棄しない。
    PendingMismatch,
}

/// []+ の入力がどこへ格納されるべきかを表す、格納先非依存の宛先。
#[derive(Debug, Clone)]
pub enum PhraseDest {
    /// 通常の Part 別フレーズ(素の `[...]` / `Part.[...]` / `@msr(M)=[...]`)。
    /// 複数パート一括指定(`L`, `ALL` 等)のために Vec で保持する。
    Part(Vec<usize>, PhraseAs),
    /// `@n=[...]` で設定される、part 非依存の共有 Variation フレーズ。
    CommonVariation(usize),
}

/// 保留中(未確定)のフレーズ入力。
#[derive(Debug, Clone)]
struct PendingPhrase {
    dest: PhraseDest,
    raw: String,
}

/// `resolve_or_buffer_phrase` の結果。
#[derive(Debug)]
pub enum PendingResult {
    /// 新規/継続バッファ。まだ格納先には送られていない。
    Buffered,
    /// 保留中だった未完成のバッファを破棄し、新しいバッファに差し替えた。
    Replaced,
    /// 保留中のバッファと異なる明示的な宛先を伴う「閉じ」入力が来たためエラー。
    /// 保留中バッファ自体は無傷のまま残る。
    Mismatch,
    /// バッファが解決し、格納先へディスパッチ可能な最終テキストが確定した
    /// (保留を経ていない、素の即時入力の場合も含む)。
    Resolved(PhraseDest, Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionCmdError {
    InvalidPart,
    InvalidComposition,
}

//*******************************************************************
//          []+ / rpt()+ 追加入力機能: 宛先非依存のバッファリング処理
//*******************************************************************
/// この入力が「開き」(まだ閉じておらず、次の入力を待つべき)パターンかどうか。
fn is_opening_pattern(input_vec: &[String]) -> bool {
    if input_vec.is_empty() {
        return false;
    }
    let first = &input_vec[0];
    let first_len = first.len();
    (input_vec.len() >= 2
        && first.ends_with(']')
        && input_vec[1].starts_with("rpt(")
        && input_vec[1].ends_with(")+"))
        || (first_len >= 2 && first.ends_with("]+"))
}
/// 「開き」パターンの入力を `raw` に連結する(1回目/2回目以降のいずれも扱う)。
fn try_extend_additional(raw: &mut String, input_vec: &[String]) {
    if input_vec.is_empty() {
        return;
    }
    let first = &input_vec[0];
    let first_len = first.len();
    if input_vec.len() >= 2
        && first.ends_with(']')
        && input_vec[1].starts_with("rpt(")
        && input_vec[1].ends_with(")+")
    {
        let rpt_cnt = extract_number_from_parentheses(&input_vec[1]).unwrap_or(1);
        for i in 0..(rpt_cnt + 1) {
            if i == 0 && raw.is_empty() {
                *raw = first[0..(first_len - 1)].to_string();
            } else {
                *raw += &first[1..(first_len - 1)];
            }
        }
    } else if first_len >= 2 && first.ends_with("]+") {
        if raw.is_empty() {
            *raw = first[0..(first_len - 2)].to_string();
        } else {
            *raw += &first[1..(first_len - 2)];
        }
    }
}
/// 保留中の `raw` と、「閉じ」入力(`]` で終わり `+` を伴わない)を結合する。
fn combine_additional(raw: &str, input_vec: Vec<String>) -> Vec<String> {
    let mut new_vec = input_vec;
    match new_vec.get_mut(0) {
        Some(head) if !head.is_empty() => {
            *head = raw.to_string() + &head[1..];
        }
        Some(head) => {
            *head = raw.to_string();
        }
        None => {
            new_vec.push(raw.to_string());
        }
    }
    #[cfg(feature = "verbose")]
    println!("Additional Phrase: {new_vec:?}");
    new_vec
}

// SeqDataStock の責務
//  入力された Phrase/Composition Data の変換と保持
#[derive(Debug)]
pub struct SeqDataStock {
    pdt: Vec<Vec<PhraseDataStock>>,
    cdt: [CompositionDataStock; MAX_INST_PART],
    pdldt: [PedalDataStock; MAX_PEDAL_PART],
    input_mode: InputMode,
    cluster_memory: String,
    pending: Option<PendingPhrase>,
    common_vari: Vec<Vec<String>>,
    /// 各 Variation 番号(index)を、現在 Composition で参照している
    /// パート番号の一覧。`{X@n}` を設定したパートを記録しておき、
    /// `@n=[...]` が変更された時にそのパートへ再送するために使う。
    common_vari_subscribers: Vec<Vec<usize>>,
    tick_for_onemsr: i32,
    tick_for_beat: i32,
    bpm: i16,
}
impl SeqDataStock {
    pub fn new() -> Self {
        let mut pd = Vec::new();
        for i in 0..(MAX_KBD_PART + MAX_VIOLIN_PART) {
            let mut vari = Vec::new();
            let base_note = Self::default_base_note(i);
            for _ in 0..(MAX_VARIATION + 1) {
                vari.push(PhraseDataStock::new(base_note));
            }
            pd.push(vari);
        }
        Self {
            pdt: pd,
            cdt: Default::default(),
            pdldt: Default::default(),
            input_mode: InputMode::Closer,
            cluster_memory: "".to_string(),
            pending: None,
            common_vari: vec![Vec::new(); MAX_VARIATION],
            common_vari_subscribers: vec![Vec::new(); MAX_VARIATION],
            tick_for_onemsr: DEFAULT_TICK_FOR_ONE_MEASURE,
            tick_for_beat: DEFAULT_TICK_FOR_QUARTER,
            bpm: DEFAULT_BPM,
        }
    }
    pub fn get_pdstk(&self, part: usize, vari: PhraseAs) -> Option<&PhraseDataStock> {
        let num = match vari {
            PhraseAs::Normal => 0,
            PhraseAs::Variation(v) => v,
            PhraseAs::Measure(_m) => MAX_VARIATION,
        };
        let ptnum = ptnum(part);
        if self.pdt[ptnum][num].send_enable {
            Some(&self.pdt[ptnum][num])
        } else {
            None
        }
    }
    pub fn get_cdstk(&self, part: usize) -> &CompositionDataStock {
        let ptnum = ptnum(part);
        &self.cdt[ptnum]
    }
    pub fn get_pdlstk(&self, part: usize) -> &PedalDataStock {
        &self.pdldt[part - MAX_ALL_KBD_PART]
    }
    pub fn set_cluster_memory(&mut self, word: String) {
        self.cluster_memory = word;
    }
    /// []+ の保留/継続/差し替え/エラーを判定し、確定した入力のみを返す。
    /// バッファリングの詳細(文字列連結・rpt展開)は宛先(`PhraseDest`)に
    /// 一切依存しない。宛先ごとの格納処理(part 別 pdt への反映、共有
    /// Variation ストアへの格納)は呼び出し元が `PendingResult::Resolved`
    /// を受け取ってから行う。
    ///
    /// `explicit` は、今回の入力が `Part.[...]` / `@n=[...]` / `@msr(M)=[...]`
    /// のように明示的な宛先指定を伴うかどうかを表す。素の `[...]` は
    /// `explicit = false` とする。
    pub fn resolve_or_buffer_phrase(
        &mut self,
        explicit: bool,
        dest: PhraseDest,
        tokens: Vec<String>,
    ) -> PendingResult {
        let opening = is_opening_pattern(&tokens);

        if let Some(pending) = self.pending.take() {
            if !explicit {
                // 素の [...] は、宛先指定の有無に関わらず既存の pending を継続する。
                // これにより [aaa]+ → [bbb]+ → [ccc] のような多段連結も、
                // 最初に確定した宛先のまま正しく動作する。
                let mut raw = pending.raw;
                if opening {
                    try_extend_additional(&mut raw, &tokens);
                    self.pending = Some(PendingPhrase {
                        dest: pending.dest,
                        raw,
                    });
                    PendingResult::Buffered
                } else {
                    let combined = combine_additional(&raw, tokens);
                    PendingResult::Resolved(pending.dest, combined)
                }
            } else if opening {
                // 明示的な宛先を伴う新規オープン: 前の未完成バッファは破棄して差し替える。
                let mut raw = String::new();
                try_extend_additional(&mut raw, &tokens);
                self.pending = Some(PendingPhrase { dest, raw });
                PendingResult::Replaced
            } else {
                // 明示的な宛先を伴う「閉じ」入力が、別の保留中バッファと衝突。
                // 保留中バッファは変更せずエラーを返す。
                self.pending = Some(pending);
                PendingResult::Mismatch
            }
        } else if opening {
            let mut raw = String::new();
            try_extend_additional(&mut raw, &tokens);
            self.pending = Some(PendingPhrase { dest, raw });
            PendingResult::Buffered
        } else {
            // 保留なし、かつ []+ を伴わない通常入力: そのまま確定。
            PendingResult::Resolved(dest, tokens)
        }
    }
    /// 解決済みの生テキストを、part 別ストレージ(pdt/pdldt)へ格納する。
    /// []+ の保留判定は済んでいる前提で、ここでは一切行わない。
    pub fn apply_phrase_data(
        &mut self,
        part: usize,
        vari: PhraseAs,
        normalized_vec: Vec<String>,
    ) -> Result<(), PhraseCmdError> {
        if part < MAX_KBD_PART || (VIOLIN1..=VIOLIN2).contains(&part) {
            let num = match vari {
                PhraseAs::Normal => 0,
                PhraseAs::Variation(v) => v,
                PhraseAs::Measure(_m) => MAX_VARIATION,
            };
            let ptnum = ptnum(part);
            if self.pdt[ptnum][num].set_raw_vec(normalized_vec, Some(&self.cluster_memory)) {
                self.pdt[ptnum][num].set_recombined(
                    Some(self.input_mode),
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                    Some(false),
                );
                return Ok(());
            }
            return Err(PhraseCmdError::InvalidPhrase);
        } else if (DAMPER_PART..=SHIFT_PART).contains(&part) {
            // Damper/Sostenuto/Shift part の場合
            if self.pdldt[part - MAX_ALL_KBD_PART].set_raw(normalized_vec.join("."), None) {
                self.pdldt[part - MAX_ALL_KBD_PART].set_recombined(
                    None,
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                    None,
                );
                return Ok(());
            }
            return Err(PhraseCmdError::InvalidPhrase);
        }
        Err(PhraseCmdError::InvalidPart)
    }
    /// `@n=[...]` で確定した生テキストを、part 非依存の共有ストアへ格納する。
    /// この時点では、どのパートの pdt にも complement/recombine は行わず、
    /// elapse への送信も行わない([]+ が閉じただけの状態)。
    pub fn set_common_variation(&mut self, vari: usize, input_text: Vec<String>) -> bool {
        if vari == 0 || vari >= self.common_vari.len() {
            return false;
        }
        self.common_vari[vari] = input_text;
        true
    }
    /// 共有 Variation ストアの内容を、指定パートの `pdt[ptnum][vari]` に
    /// (そのパート固有の base_note/tick で)複合・再構成する。
    /// Composition 側で `{X/@n}` が指定された時点で呼ばれる。
    /// 反映できた(=呼び出し元が elapse へ送るべき)場合に true を返す。
    pub fn sync_common_variation_to_part(&mut self, part: usize, vari: usize) -> bool {
        if vari == 0 || vari >= self.common_vari.len() || self.common_vari[vari].is_empty() {
            return false;
        }
        let ptnum = ptnum(part);
        if ptnum >= self.pdt.len() {
            return false;
        }
        let raw = self.common_vari[vari].clone();
        if self.pdt[ptnum][vari].set_raw_vec(raw, Some(&self.cluster_memory)) {
            self.pdt[ptnum][vari].set_recombined(
                Some(self.input_mode),
                self.tick_for_onemsr,
                self.tick_for_beat,
                Some(false),
            );
            true
        } else {
            false
        }
    }
    /// 指定パートが Composition で参照している Variation 番号集合を、
    /// 今回の Composition 設定結果(`vari_refs`)に合わせて更新する。
    /// 参照しなくなった番号の購読リストからはこのパートを外し、新たに
    /// 参照した番号の購読リストにはこのパートを加える。これにより、
    /// `common_vari_subscribers[n]` は常に「現在 `@n` を参照している
    /// パート一覧」を保つ。
    pub fn update_variation_subscription(&mut self, part: usize, vari_refs: &[usize]) {
        for (n, subs) in self.common_vari_subscribers.iter_mut().enumerate() {
            if vari_refs.contains(&n) {
                if !subs.contains(&part) {
                    subs.push(part);
                }
            } else {
                subs.retain(|&p| p != part);
            }
        }
    }
    /// 指定 Variation 番号を、現在 Composition で参照しているパート一覧。
    /// `@n=[...]` が変更された時、ここで得たパートへ再送する。
    pub fn common_variation_subscribers(&self, vari: usize) -> Vec<usize> {
        if vari == 0 || vari >= self.common_vari_subscribers.len() {
            Vec::new()
        } else {
            self.common_vari_subscribers[vari].clone()
        }
    }
    /// `!clear` 等、全データ消去時に共有 Variation ストアと購読情報も消去する。
    pub fn clear_common_variation(&mut self) {
        for v in self.common_vari.iter_mut() {
            v.clear();
        }
        for s in self.common_vari_subscribers.iter_mut() {
            s.clear();
        }
    }
    pub fn del_raw_phrase(&mut self, part: usize) {
        if part < MAX_KBD_PART {
            for i in 0..(MAX_VARIATION + 1) {
                if self.pdt[part][i].set_raw("[]".to_string(), Some(&self.cluster_memory)) {
                    self.pdt[part][i].set_recombined(
                        Some(self.input_mode),
                        self.tick_for_onemsr,
                        self.tick_for_beat,
                        Some(false),
                    );
                }
            }
        } else if (DAMPER_PART..=SHIFT_PART).contains(&part) {
            // Damper/Sostenuto/Shift part の場合
            if self.pdldt[part - MAX_ALL_KBD_PART].set_raw("[]".to_string(), None) {
                self.pdldt[part - MAX_ALL_KBD_PART].set_recombined(
                    None,
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                    None,
                );
            }
        } else if (VIOLIN1..=VIOLIN2).contains(&part) {
            let ptnum = ptnum(part);
            for i in 0..(MAX_VARIATION + 1) {
                if self.pdt[ptnum][i].set_raw("[]".to_string(), Some(&self.cluster_memory)) {
                    self.pdt[ptnum][i].set_recombined(
                        Some(self.input_mode),
                        self.tick_for_onemsr,
                        self.tick_for_beat,
                        Some(false),
                    );
                }
            }
        }
    }
    /// Composition をセットし、成功時は入力中に参照された Variation 番号
    /// (`@n`)の一覧を返す。呼び出し元はこれを使って、共有 Variation ストア
    /// の内容をこのパートへ同期・送信するかどうかを判断する。
    pub fn set_raw_composition(
        &mut self,
        part: usize,
        input_text: Vec<String>,
    ) -> Result<Vec<usize>, CompositionCmdError> {
        let ptnum = ptnum(part);
        if ptnum < MAX_INST_PART
            && let Some(refs) = self.cdt[ptnum].set_raw_vec(input_text)
        {
            self.cdt[ptnum].set_recombined(None, self.tick_for_onemsr, self.tick_for_beat, None);
            return Ok(refs);
        }
        if part < MAX_INST_PART {
            Err(CompositionCmdError::InvalidComposition)
        } else {
            Err(CompositionCmdError::InvalidPart)
        }
    }
    pub fn change_beat(&mut self, numerator: i16, denomirator: i16) {
        #[cfg(feature = "verbose")]
        println!("beat: {numerator}/{denomirator}");
        self.tick_for_onemsr =
            DEFAULT_TICK_FOR_ONE_MEASURE * (numerator as i32) / (denomirator as i32);
        self.tick_for_beat = DEFAULT_TICK_FOR_QUARTER * 4 / (denomirator as i32);
        self.recombine_all();
    }
    pub fn change_bpm(&mut self, bpm: i16) {
        self.bpm = bpm;
        self.recombine_phr_all();
    }
    pub fn change_oct(&mut self, oct: i32, relative: bool, part: usize) -> bool {
        let mut update = false;
        let new_bd: i32;
        let ptnum = ptnum(part);
        if oct == 0 {
            // Reset Octave
            new_bd = Self::default_base_note(ptnum);
            let dbn_now = self.pdt[ptnum][0].base_note;
            if new_bd != dbn_now {
                update = true;
            }
        } else {
            let old = self.pdt[ptnum][0].base_note / 12 - 1;
            let mut new = old;
            if relative {
                new += oct;
            } else {
                new = oct;
            }
            if new >= 8 {
                new = 7;
            } else if new < 1 {
                new = 1;
            }
            update = old != new;
            new_bd = (new + 1) * 12;
        }
        if update {
            for epd in self.pdt[ptnum].iter_mut() {
                epd.base_note = new_bd;
                epd.set_recombined(
                    Some(self.input_mode),
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                    Some(true),
                );
            }
        }
        update
    }
    pub fn change_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }
    fn recombine_phr_all(&mut self) {
        for pd in self.pdt.iter_mut() {
            for epd in pd.iter_mut() {
                epd.set_recombined(
                    Some(self.input_mode),
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                    Some(true),
                );
            }
        }
        for epdl in self.pdldt.iter_mut() {
            epdl.set_recombined(None, self.tick_for_onemsr, self.tick_for_beat, None);
        }
    }
    fn recombine_all(&mut self) {
        for (i, pd) in self.pdt.iter_mut().enumerate() {
            for epd in pd.iter_mut() {
                epd.set_recombined(
                    Some(self.input_mode),
                    self.tick_for_onemsr,
                    self.tick_for_beat,
                    Some(true),
                );
            }
            self.cdt[i].set_recombined(None, self.tick_for_onemsr, self.tick_for_beat, None);
        }
        for epdl in self.pdldt.iter_mut() {
            epdl.set_recombined(None, self.tick_for_onemsr, self.tick_for_beat, None);
        }
    }
    fn default_base_note(ptnum: usize) -> i32 {
        // ptnum: MAX_KBD_PART のあと、VIOLIN1, VIOLIN2 と続く
        if ptnum < MAX_KBD_PART {
            (DEFAULT_NOTE_NUMBER as i32) + 12 * ((ptnum as i32) - 2)
        //} else if ptnum < MAX_KBD_PART + MAX_VIOLIN_PART {
        //    72
        } else {
            60
        }
    }
}

//*******************************************************************
//          Data Stock Trait
//*******************************************************************
pub trait DataStock {
    /// Set raw input text and complement data
    fn set_raw(&mut self, input_text: String, additional_word: Option<&str>) -> bool;
    /// Recombine to internal format data
    fn set_recombined(
        &mut self,
        input_mode: Option<InputMode>,
        tick_for_onemsr: i32,
        tick_for_beat: i32,
        resend: Option<bool>,
    );
    /// Get final ElpsMsg
    fn get_final(&self, part: i16, vari: Option<PhraseAs>) -> ElpsMsg;
}

//*******************************************************************
//          Phrase Data Stock Struct
//*******************************************************************
#[derive(Debug)]
pub struct PhraseDataStock {
    base_note: i32,
    raw: Vec<String>,
    cmpl: Option<Box<PhraseComplemented>>,
    phr: Vec<PhrEvt>,
    ana: Vec<AnaEvt>,
    do_loop: bool,
    whole_tick: i32,
    send_enable: bool,
}
impl PhraseDataStock {
    fn new(base_note: i32) -> Self {
        Self {
            base_note,
            raw: Vec::new(),
            cmpl: None,
            phr: Vec::new(),
            ana: Vec::new(),
            do_loop: true,
            whole_tick: 0,
            send_enable: true,
        }
    }
    fn set_raw_vec(&mut self, input_text: Vec<String>, cluster_word: Option<&str>) -> bool {
        self.send_enable = true;
        self.raw = input_text.clone();

        self.cmpl = Some(complement_phrase(input_text, cluster_word.unwrap_or("")));
        if cfg!(feature = "verbose") {
            self.debug_print();
        }
        true
    }
    pub fn _get_cmpl_nt(&self) -> Vec<String> {
        // for test
        if let Some(cmpl) = &self.cmpl {
            cmpl.note_info.clone()
        } else {
            vec!["".to_string()]
        }
    }
    pub fn get_phr(&self) -> &Vec<PhrEvt> {
        &self.phr
    }
    fn debug_print(&self) {
        println!(
            "complement_phrase: {:?} exp: {:?} atrb: {:?} accia: {:?}",
            if let Some(cmpl) = &self.cmpl {
                cmpl.note_info.clone()
            } else {
                ["-".to_string()].to_vec()
            },
            if let Some(cmpl) = &self.cmpl {
                cmpl.music_exp.clone()
            } else {
                ["-".to_string()].to_vec()
            },
            if let Some(cmpl) = &self.cmpl {
                cmpl.note_attribute.clone()
            } else {
                [None].to_vec()
            },
            if let Some(cmpl) = &self.cmpl {
                cmpl.accia_info.clone()
            } else {
                [None].to_vec()
            }
        );
    }
    fn check_empty_phrase(&mut self) -> bool {
        if let Some(cmpl) = &self.cmpl {
            if cmpl.note_info == [""] {
                //  clear
                self.phr = Vec::new();
                self.ana = Vec::new();
                self.whole_tick = 0;
                true
            } else {
                false
            }
        } else {
            //  clear
            self.phr = Vec::new();
            self.ana = Vec::new();
            self.whole_tick = 0;
            true
        }
    }
}
impl DataStock for PhraseDataStock {
    fn set_raw(&mut self, input_text: String, cluster_word: Option<&str>) -> bool {
        self.set_raw_vec(vec![input_text], cluster_word)
    }
    fn set_recombined(
        &mut self,
        input_mode: Option<InputMode>,
        tick_for_onemsr: i32,
        _tick_for_beat: i32,
        resend: Option<bool>,
    ) {
        // 2.5. check empty phrase
        if self.check_empty_phrase() {
            return;
        }

        // 3.recombined data
        let (whole_tick, do_loop, rcmb) = recombine_to_internal_format(
            self.cmpl.as_ref().unwrap(),
            input_mode.unwrap_or(InputMode::Closer),
            self.base_note,
            tick_for_onemsr,
        );
        if resend.unwrap_or(true) && !do_loop {
            // do_loop が false の場合は、再生しない
            #[cfg(feature = "verbose")]
            println!("do_loop is false, not updating phrase data.");
            self.send_enable = false;
            return;
        }
        self.phr = rcmb;
        self.do_loop = do_loop;
        self.whole_tick = whole_tick;

        // 4.analysed data
        self.ana = analyse_data(&self.phr, &self.cmpl.as_ref().unwrap().music_exp);
        #[cfg(feature = "verbose")]
        {
            println!("final_phrase: {:?}", self.phr);
            println!(
                "whole_tick: {:?} do_loop: {:?}",
                self.whole_tick, self.do_loop
            );
            println!("analyse: {:?}", self.ana);
        }
    }
    fn get_final(&self, part: i16, vari: Option<PhraseAs>) -> ElpsMsg {
        let do_loop = vari == Some(PhraseAs::Normal) && self.do_loop;
        ElpsMsg::Phr(
            part,
            PhrData {
                whole_tick: self.whole_tick as i16,
                do_loop,
                evts: self.phr.clone(),
                ana: self.ana.clone(),
                vari: match vari {
                    Some(v) => v,
                    None => PhraseAs::Normal,
                },
                auftakt: self
                    .cmpl
                    .as_ref()
                    .and_then(|c| c.note_attribute[0])
                    .unwrap_or(0),
            },
        )
    }
}

//*******************************************************************
//          Composition Data Stock Struct
//*******************************************************************
#[derive(Debug)]
pub struct CompositionDataStock {
    raw: Vec<String>,
    cmpl_cd: Vec<String>,
    chord: Vec<CmpEvt>,
    do_loop: bool,
    whole_tick: i32,
}
impl Default for CompositionDataStock {
    fn default() -> Self {
        Self {
            raw: Vec::new(),
            cmpl_cd: vec!["".to_string()],
            chord: Vec::new(),
            do_loop: true,
            whole_tick: 0,
        }
    }
}
impl CompositionDataStock {
    /// 成功時、入力中に含まれていた `@n` 参照(Variation番号)の一覧を返す。
    fn set_raw_vec(&mut self, input_text: Vec<String>) -> Option<Vec<usize>> {
        self.raw = input_text.clone();

        if let Some(cmpl) = complement_composition(input_text) {
            let refs = scan_variation_refs(&cmpl);
            self.cmpl_cd = cmpl.clone();
            #[cfg(feature = "verbose")]
            println!("complement_composition: {cmpl:?}");
            Some(refs)
        } else {
            println!("Composition input failed!");
            None
        }
    }
}
impl DataStock for CompositionDataStock {
    fn get_final(&self, part: i16, _vari: Option<PhraseAs>) -> ElpsMsg {
        ElpsMsg::Cmp(
            part,
            CmpData {
                whole_tick: self.whole_tick as i16,
                do_loop: self.do_loop,
                evts: self.chord.clone(),
                measure: NOTHING,
            },
        )
    }
    fn set_raw(&mut self, input_text: String, _additional_word: Option<&str>) -> bool {
        self.set_raw_vec(vec![input_text]).is_some()
    }
    fn set_recombined(
        &mut self,
        _input_mode: Option<InputMode>,
        tick_for_onemsr: i32,
        tick_for_beat: i32,
        _resend: Option<bool>,
    ) {
        if self.cmpl_cd == [""] {
            // clear
            self.chord = Vec::new();
            self.whole_tick = 0;
            self.do_loop = true;
            //println!("no_composition...");
        } else {
            // 3.recombined data
            let (whole_tick, do_loop, rcmb) =
                recombine_to_chord_loop(&self.cmpl_cd, tick_for_onemsr, tick_for_beat);
            self.chord = rcmb;
            self.do_loop = do_loop;
            self.whole_tick = whole_tick;
            #[cfg(feature = "verbose")]
            println!(
                "final_composition: {:?} whole_tick: {:?}",
                self.chord, self.whole_tick
            );
        }
    }
}

//*******************************************************************
//          Pedal Data Stock Struct
//*******************************************************************
#[derive(Debug, Default)]
pub struct PedalDataStock {
    pub raw: Vec<String>,
    pdl: Vec<PhrEvt>,
    do_loop: bool,
    whole_tick: i32,
}
impl DataStock for PedalDataStock {
    fn set_raw(&mut self, input_text: String, _additional_word: Option<&str>) -> bool {
        self.raw = complement_pedal(input_text);
        #[cfg(feature = "verbose")]
        println!("PedalDataStock: {:?}", self.raw);
        true
    }
    fn set_recombined(
        &mut self,
        _input_mode: Option<InputMode>,
        tick_for_onemsr: i32,
        tick_for_beat: i32,
        _resend: Option<bool>,
    ) {
        let (whole_tick, do_loop, rcmb) =
            recombine_to_internal_format_pedal(&self.raw, tick_for_onemsr, tick_for_beat);
        self.pdl = rcmb;
        self.do_loop = do_loop;
        self.whole_tick = whole_tick;
        #[cfg(feature = "verbose")]
        println!("PedalDataStock recombined: {:?}", self.pdl);
    }
    fn get_final(&self, part: i16, _vari: Option<PhraseAs>) -> ElpsMsg {
        ElpsMsg::Phr(
            part,
            PhrData {
                whole_tick: self.whole_tick as i16,
                do_loop: self.do_loop,
                evts: self.pdl.clone(),
                ana: Vec::new(),
                vari: PhraseAs::default(),
                auftakt: 0,
            },
        )
    }
}
