//  Created by Hasebe Masahiko on 2023/02/15.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use crate::lpnlib;

pub struct TextParse {}
impl TextParse {
    pub fn complement_phrase(input_text: String) -> [Vec<String>;2] {
        // 1. [] を抜き出し、２つ分の brackets を Vec に入れて戻す
        let (ni, ne) = TextParse::divide_brackets(input_text);

        // 2. ,| 重複による休符指示の補填
        let nf = TextParse::fill_omitted_note_data(ni);

        // 3. , で分割
        let mut nvec = TextParse::split_by(',', nf);

        // 4. < >*n を展開
        loop {
            let (nvr_tmp, no_exist) = TextParse::expand_repeat(nvec.clone());
            nvec = nvr_tmp.clone();
            if no_exist {break;}
        }

        // 5. 同音繰り返しの展開
        loop {
            let (nvr_tmp, no_exist) = TextParse::note_repeat(nvec.clone());
            nvec = nvr_tmp.clone();
            if no_exist {break;}
        }

        // 6. Expression を , で分割
        let nevec = TextParse::split_by(',', ne);

        println!("complement_phrase: {:?} exp: {:?}",nvec,nevec);
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
        // ,| 連続入力による、休符指示の補填
        if nf.len() == 0 {return "".to_string();}

        let mut fill: String = "".to_string();
        let mut doremi = "x".to_string();
        let mut doremi_end_flag = true;
        for ltr in nf.chars() {
            if ltr == ',' {
                fill += &doremi;
                fill += ",";
                doremi = "x".to_string();
                doremi_end_flag = true;
            }
            else if ltr == '|' || ltr == '/' {
                fill += &doremi;
                fill += "|,";
                doremi = "x".to_string();
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
                                for h in (repeat_start..(i+1)).rev() {
                                    new_vec.insert(i+1, new_vec[h].clone());
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
    //=========================================================================
    pub fn recombine_to_internal_format(ntvec: &Vec<String>, expvec: &Vec<String>, imd: lpnlib::InputMode,
      oct_setting: i32, tick_for_onemsr: i32) -> (i32, Vec<Vec<u16>>) {
        let max_read_ptr = ntvec.len();
        let (exp_vel, exp_others) = TextParse::get_exp_info(expvec.clone());
        let mut read_ptr = 0;
        let mut last_nt: u8 = 0;
        let mut tick: i32 = 0;
        let mut msr: i32 = 1;
        let mut base_dur: i32 = lpnlib::DEFAULT_TICK_FOR_QUARTER;
        let mut rcmb: Vec<Vec<u16>> = Vec::new();

        while read_ptr < max_read_ptr {
            let note_text = ntvec[read_ptr].clone();

            let (notes, mes_end, bdur, dur_cnt, diff_vel, nt)
              = TextParse::break_up_nt_dur_vel(note_text, oct_setting, last_nt, base_dur, imd);

            base_dur = bdur;
            if nt <= 127 {last_nt = nt;}    // 次回の音程の上下判断のため
            let next_msr_tick = tick_for_onemsr*msr;
            if tick < next_msr_tick {
                let real_dur = TextParse::get_real_dur(base_dur, dur_cnt,
                    next_msr_tick - tick);

                // duration
                let note_dur = TextParse::trans_dur(real_dur, &exp_others);
    
                // velocity
                let mut last_vel: i32 = exp_vel + diff_vel as i32;
                if last_vel > 127 {last_vel = 127;}
                else if last_vel < 1 {last_vel = 1;}

                // add to recombined data
                rcmb = TextParse::add_note(rcmb, tick, notes, note_dur, last_vel as u16);
                tick += real_dur;
            }
            if mes_end {// 小節線があった場合
                tick = next_msr_tick;
                msr += 1;
            }
            read_ptr += 1;  // out from repeat
        }
        println!("recombined_phrase: {:?} whole_tick: {:?}",rcmb,tick);
        (tick, rcmb)
    }
    fn get_exp_info(expvec: Vec<String>) -> (i32, Vec<String>) {
        let mut vel = lpnlib::END_OF_DATA;
        let mut retvec = expvec.clone();
        for (i, txt) in expvec.iter().enumerate() {
            vel = TextParse::convert_exp2vel(txt);
            if vel != lpnlib::END_OF_DATA {
                retvec.remove(i);
                break;
            }
        }
        if vel == lpnlib::END_OF_DATA {vel=lpnlib::DEFAULT_VEL as i32;}
        (vel, retvec)
    }
    fn break_up_nt_dur_vel(note_text: String, oct_setting: i32, last_nt: u8, bdur: i32, imd: lpnlib::InputMode)
      -> (Vec<u8>, bool, i32, u16, u8, u8) { //(notes, mes_end, base_dur, dur_cnt, diff_vel, nt)

        //  小節線のチェック
        let mut mes_end = false;
        let mut ntext1 = note_text.clone();
        if note_text.chars().last().unwrap_or(' ') == '|' { // 小節最後のイベント
            mes_end = true;
            ntext1.pop();
        }

        //  duration 情報、 Velocity 情報の抽出
        let (ntext3, base_dur, dur_cnt) = TextParse::gen_dur_info(ntext1, bdur);
        let (ntext4, diff_vel) = TextParse::gen_diff_vel(ntext3);
        let notes_vec: Vec<String> = TextParse::split_by_by('=', '_', ntext4);

        let mut notes: Vec<u8> = Vec::new();
        let mut doremi: i32 = 0;
        for nt in notes_vec.iter() {    // 同時発音
            if imd == lpnlib::InputMode::Fixed {
                doremi = TextParse::convert_doremi_fixed(nt.to_string());
            }
            else if imd == lpnlib::InputMode::Closer {
                doremi = TextParse::convert_doremi_closer(nt.to_string(), last_nt as i32);
            }
            let mut base_pitch: i32 = oct_setting*12 + lpnlib::DEFAULT_NOTE_NUMBER as i32 + doremi as i32;
            if base_pitch >= lpnlib::MAX_NOTE_NUMBER as i32 {base_pitch = lpnlib::MAX_NOTE_NUMBER as i32;}
            else if base_pitch < lpnlib::MIN_NOTE_NUMBER as i32 {base_pitch = lpnlib::MIN_NOTE_NUMBER as i32;}
            notes.push(base_pitch as u8);
        }
        (notes, mes_end, base_dur, dur_cnt, diff_vel, doremi as u8)
    }
    fn gen_dur_info(nt: String, bdur: i32) -> (String, i32, u16) {
        // +- は、最初にあっても、音価指定の後にあってもいいので、一番前にある +- を削除して、
        // 音価情報を分析、除去した後、あらためて削除した +- を元に戻す
        let mut excnt = 0;
        for (i, ltr) in nt.chars().enumerate() {
            if ltr == '+' || ltr == '-' {continue;}
            else {
                excnt = i;
                break;
            }
        }
        let mut ntext = nt[excnt..].to_string();
        let mut base_dur = bdur;
        let mut dur_cnt: u16 = 1;

        //  タイなどの音価を解析し、dur_cnt を確定
        let txtlen = ntext.len(); 
        if txtlen > 0 {
            if ntext.chars().nth(txtlen-1).unwrap_or(' ') == 'o' {
                base_dur = lpnlib::LAST;
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
        let txtlen = ntext.len();
        if txtlen > 0 {
            let mut triplet: u16 = 0;
            let mut idx = 1;
            let mut fst_ltr = ntext.chars().nth(0).unwrap_or(' ');
            if fst_ltr == '3' || fst_ltr == '5' {
                triplet = fst_ltr as u16;
                fst_ltr = ntext.chars().nth(1).unwrap_or(' ');
            }
            if fst_ltr == '\'' {
                if ntext.chars().nth(2).unwrap_or(' ') == '\"' {
                    base_dur = 60;
                    idx = 2;
                }
                else {base_dur = 240;}
            }
            else if fst_ltr == '\"' {base_dur = 120;}
            else if fst_ltr == 'q' {base_dur = 480;}
            else if fst_ltr == 'h' {base_dur = 960;}
            else {idx = 0;}
            if triplet != 0 {
                base_dur = base_dur*2/triplet as i32;
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
    fn gen_diff_vel(nt: String) -> (String, u8) {
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
    fn get_real_dur(base_dur: i32, dur_cnt: u16, rest_tick: i32) -> i32 {
        if base_dur == lpnlib::LAST {
            rest_tick
        }
        else if base_dur == lpnlib::KEEP {
            base_dur*dur_cnt as i32
        }
        else {
            base_dur*dur_cnt as i32
        }
    }
    fn trans_dur(real_dur: i32, exp_others: &Vec<String>) -> i32 {
        let mut return_dur = real_dur;
        if exp_others.iter().any(|x| x=="stacc") {
            return_dur = real_dur/2;
        }
        if return_dur > 40 {  // 一律 duration 40 を引く
            return_dur - 40
        }
        else {return_dur}
    }
    fn add_note(rcmb: Vec<Vec<u16>>, tick: i32, notes: Vec<u8>, note_dur: i32, last_vel: u16) -> Vec<Vec<u16>> {
        let mut return_rcmb = rcmb.clone();
        if notes.len() != 0 {
            for note in notes.iter() {
                if *note == lpnlib::REST {
                    continue;
                }
                else if *note == lpnlib::NO_NOTE {
                    continue;
                    // python で、前の入力が '=' による和音入力だった場合も考え、直前の同じタイミングのデータを全て調べる
                    // とコメントがあったが、処理内容不明
                /*  same_tick = generated[-1][1]
                    cnt = 0
                    while True:
                        if len(generated) <= cnt: break
                        cnt += 1
                        if generated[-cnt][1] == same_tick:
                            generated[-cnt][2] += real_dur
                        else: break */
                }
                else {
                    let nt_data: Vec<u16> = 
                        vec![lpnlib::TYPE_NOTE, tick as u16, note_dur as u16, *note as u16, last_vel];
                        return_rcmb.push(nt_data);
                }
            }
        }
        else {println!("Error!")}
        return_rcmb
    }
    //=========================================================================
    fn convert_doremi_closer(doremi: String, last_nt: i32) -> i32 {
        if doremi.len() == 0 {return lpnlib::NO_NOTE as i32;}
        let mut last_doremi = last_nt;
        while last_doremi >= 12 {last_doremi -= 12;}
        while last_doremi < 0 {last_doremi += 12;}

        let mut oct_pitch = 0;
        let mut pure_doremi = String::from("");
        for (i, ltr) in doremi.chars().enumerate() {
            if ltr == 'x' {return lpnlib::REST as i32;}
            else if ltr == '+' {oct_pitch += 12;}
            else if ltr == '-' {oct_pitch -= 12;}
            else {
                pure_doremi = doremi[i..].to_string();
                break;
            }
        }

        let mut base_note = 0;
        if pure_doremi.len() != 0 {
            match pure_doremi.chars().nth(0).unwrap_or(' ') {
                'd' => base_note += 0,
                'r' => base_note += 2,
                'm' => base_note += 4,
                'f' => base_note += 5,
                's' => base_note += 7,
                'l' => base_note += 9,
                't' => base_note += 11,
                _   => {return lpnlib::NO_NOTE as i32;},
            }
        }
        else {return lpnlib::NO_NOTE as i32;}
        pure_doremi.remove(0);

        if pure_doremi.len() != 0 {
            match pure_doremi.chars().nth(0).unwrap_or(' ') {
                'i' => base_note += 1,
                'a' => base_note -= 1,
                _   => (),
            }
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
        if doremi.len() == 0 {return lpnlib::NO_NOTE as i32;}
        let mut base_pitch: i32 = 0;
        let mut pure_doremi = String::from("");
        for (i, ltr) in doremi.chars().enumerate() {
            if ltr == 'x' {return lpnlib::REST as i32;}
            else if ltr == '+' {base_pitch += 12;}
            else if ltr == '-' {base_pitch -= 12;}
            else {
                pure_doremi = doremi[i..].to_string();
                break;
            }
        }
        if pure_doremi.len() != 0 {
            match pure_doremi.chars().nth(0).unwrap_or(' ') {
                'd' => base_pitch += 0,
                'r' => base_pitch += 2,
                'm' => base_pitch += 4,
                'f' => base_pitch += 5,
                's' => base_pitch += 7,
                'l' => base_pitch += 9,
                't' => base_pitch += 11,
                _   => {return lpnlib::NO_NOTE as i32;},
            }
        }
        else {return lpnlib::NO_NOTE as i32;}

        if pure_doremi.len() > 1 {
            match pure_doremi.chars().nth(1).unwrap_or(' ') {
                'i' => base_pitch += 1,
                'a' => base_pitch -= 1,
                _   => ()
            }
        }
        base_pitch
    }
    fn convert_exp2vel(vel_text: &str) -> i32 {
        match vel_text {
            "ff" => 127,
            "f"  => 114,
            "mf" => 100,
            "mp" => 84,
            "p"  => 64,
            "pp" => 48,
            "ppp"   => 24,
            "pppp"  => 12,
            "ppppp" => 1,
            _    => lpnlib::END_OF_DATA,
        }
    }
    pub fn split_by(splitter: char, txt: String) -> Vec<String> {
        let mut splited: Vec<String> = Vec::new();
        let mut old_locate: usize = 0;
        for (i, ltr) in txt.chars().enumerate() {
            if ltr == splitter {
                splited.push((&txt[old_locate..i]).to_string());
                old_locate = i+1;
            }
        }
        splited.push((&txt[old_locate..txt.len()]).to_string());
        splited
    }
    pub fn split_by_by(sp1: char, sp2: char, txt: String) -> Vec<String> {
        let mut splited: Vec<String> = Vec::new();
        let mut old_locate: usize = 0;
        for (i, ltr) in txt.chars().enumerate() {
            if ltr == sp1 || ltr == sp2 {
                splited.push((&txt[old_locate..i]).to_string());
                old_locate = i+1;
            }
        }
        splited.push((&txt[old_locate..txt.len()]).to_string());
        splited
    }
}