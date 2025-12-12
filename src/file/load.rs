//  Created by Hasebe Masahiko on 2025/12/07.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::lpn_file::*;
use crate::cmd::txt_common::*;
use crate::elapse::tickgen::CrntMsrTick;
use crate::lpnlib::*;
use std::fs;
//*******************************************************************
//      LoadBuffer Struct
//*******************************************************************
pub struct LoadBuffer {
    file_name: String,
    loaded_text: Vec<String>, //ファイルからロードされたテキスト
}
impl LpnFile for LoadBuffer {}
impl LoadBuffer {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            loaded_text: Vec::new(),
        }
    }
    /// ファイル名の取得・設定・クリア
    pub fn get_file_name(&self) -> Option<String> {
        if self.file_name.is_empty() {
            None
        } else {
            Some(self.file_name.clone())
        }
    }
    pub fn set_file_name(&mut self, fname: String) {
        self.file_name = fname;
    }
    pub fn clear_file_name(&mut self) {
        self.file_name = String::new();
    }
    /// .lpn ファイルをロードする
    pub fn load_lpn(&mut self, path: Option<&str>) -> bool {
        let fp_string = self.gen_lpn_file_name(self.file_name.clone(), path);
        let fp = self.path_str(&fp_string);
        self.loaded_text = Vec::new();
        match fs::read_to_string(fp) {
            Ok(content) => {
                for line in content.lines() {
                    if line.len() > 1 {
                        // !rd() 指定、コメント行があれば読み飛ばす
                        let notxt = line[0..2].to_string();
                        if notxt == "//" || notxt == "20" || notxt == "!l" {
                            // コメントでないか、過去の 2023.. が書かれてないか、loadではないか
                            continue;
                        }
                        if line.len() >= 4 && &line[0..4] == "!rd(" {
                            // 読み飛ばす
                            continue;
                        }
                    }
                    // ここまで来たら、読み込む行
                    if !line.is_empty() {
                        // blk指定がなく、ファイル全体をロードする場合
                        self.loaded_text.push(line.to_string());
                    }
                }
            }
            Err(_err) => println!("Can't open a file"),
        };
        !self.loaded_text.is_empty()
    }
    /// .lpn ファイルから !rd(num): data 形式の行を読み込み、data 部分を返す
    pub fn read_line_from_lpn(&self, path: Option<&str>, num: usize) -> Option<String> {
        let mut real_path = LOAD_FOLDER.to_string();
        let fname = self.file_name.clone();
        if let Some(lp) = path {
            real_path = real_path + "/" + lp;
        }
        match fs::read_to_string(real_path + "/" + &fname + ".lpn") {
            Ok(content) => {
                for line in content.lines() {
                    if line.len() >= 4 && &line[0..4] == "!rd(" {
                        let rd_line = split_by(':', line.to_string());
                        if rd_line.len() == 2
                            && extract_number_from_parentheses(&rd_line[0]) == Some(num)
                        {
                            return Some(rd_line[1].clone());
                        }
                    }
                }
                println!("No rd!({}) in {}", num, fname);
            }
            Err(_err) => println!("Can't open a file"),
        };
        None
    }
    /// ファイル内で !blk() を使ったデータにおいて、
    /// 指定された block から、データの再生開始場所を調べ、
    /// そこから block が終わるまでのデータを返す
    pub fn get_loaded_blk(&self, selected_blk: &str) -> Vec<String> {
        let mut txt_this_time: Vec<String> = Vec::new();
        let mut idx: Option<usize> = None;
        let blk_or = |ctxt: &str| ctxt.len() > 5 && ctxt[0..5] == *"!blk(";

        let arg_input = selected_blk.trim().split(',').collect::<Vec<&str>>();
        let selected_blk = arg_input[0];
        println!("Arg Input: {:?}", arg_input);

        // 先頭を探す
        let mut arguments: Vec<&str> = Vec::new();
        for crnt in self.loaded_text.iter().enumerate() {
            let ctxt = crnt.1;
            if blk_or(ctxt) {
                let argument = extract_texts_from_parentheses(ctxt);
                arguments = argument.split(',').collect::<Vec<&str>>();
                if arguments[0] != selected_blk {
                    continue;
                }
                // 見つかった
                idx = Some(crnt.0 + 1);
                break;
            }
        }

        if arguments.is_empty() {
            // ブロックが見つからなかった
            return txt_this_time;
        }
        let generics_num = arguments.len() - 1;
        let generics_args = if generics_num > 0 {
            &arguments[1..]
            .iter()
            .map(|arg| arg.split('=').collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
        } else {
            &Vec::new()
        };

        // デバッグ表示
        println!("Block: {}", selected_blk);
        println!("Generics: {}", generics_num);
        for ga in generics_args.iter() {
            if ga.len() == 2 {
                println!("  {} = {}", ga[0], ga[1]);
            } else {
                println!("  {}", ga[0]);
            }
        }

        // ここから txt_this_time に記録
        if let Some(start_idx) = idx {
            for n in start_idx..self.loaded_text.len() {
                let ctxt = &self.loaded_text[n];
                if blk_or(ctxt) || ctxt.is_empty() {
                    // 次のブロック、あるいは空行
                    break;
                } else {
                    txt_this_time.push(self.loaded_text[n].clone());
                }
            }
        }

        // generics 引数の置換
        for ga in generics_args.iter().enumerate() {
            for line in txt_this_time.iter_mut() {
                if line.contains(ga.1[0]) {
                    // 置換
                    let to_str = if arg_input.len() - 1 > ga.0 {
                        arg_input[ga.0 + 1]
                    } else if ga.1.len() == 2 {
                        ga.1[1]
                    } else {
                        ""
                    };
                    *line = line.replace(ga.1[0], to_str);
                }
            }
        }

        txt_this_time
    }
    /// ファイル内で !msr() を使ったデータにおいて、
    /// 指定された小節数から次の !msr() までのデータを返す
    pub fn get_from_msr_to_next(&self, mt: CrntMsrTick) -> (Vec<String>, Option<CrntMsrTick>) {
        let mut txt_this_time: Vec<String> = Vec::new();
        let mut idx: usize = 0;
        let start_msr: usize = mt.msr as usize;
        let msr_exists = |ctxt: &str| ctxt.len() > 5 && ctxt[0..5] == *"!msr(";

        // 先頭を探す
        if start_msr != 0 {
            for crnt in self.loaded_text.iter().enumerate() {
                let ctxt = crnt.1;
                if msr_exists(ctxt) && extract_number_from_parentheses(ctxt) == Some(start_msr) {
                    idx = crnt.0 + 1;
                    break;
                }
            }
        }

        // ここから txt_this_time に記録
        let blk_starts = |ctxt: &str| ctxt.len() > 5 && ctxt[0..5] == *"!blk(";
        let rd_exists = |ctxt: &str| ctxt.len() > 5 && ctxt[0..5] == *"!rd(";
        let mut blk_keeps = false;
        for n in idx..self.loaded_text.len() {
            let ctxt = &self.loaded_text[n];
            if msr_exists(ctxt) {
                // !msr() の場合
                let msr = extract_number_from_parentheses(ctxt).unwrap_or(0);
                return (
                    txt_this_time,
                    Some(CrntMsrTick {
                        msr: msr.try_into().unwrap_or(0),
                        tick: 0,
                        tick_for_onemsr: 0,
                        ..Default::default()
                    }),
                );
            } else if blk_starts(ctxt) {
                blk_keeps = true;
            } else if blk_keeps {
                if ctxt.is_empty() {
                    // 空行
                    blk_keeps = false;
                }
            } else if rd_exists(ctxt) {
            } else {
                txt_this_time.push(self.loaded_text[n].clone());
            }
        }
        // 最後まで行った場合
        (
            txt_this_time,
            Some(CrntMsrTick {
                msr: LAST,
                tick: 0,
                tick_for_onemsr: 0,
                ..Default::default()
            }),
        )
    }
    /// ファイル内で !msr() を使ったデータにおいて、
    /// 最初から、指定された小節後最初の !msr() までのデータを返す
    pub fn get_from_0_to_mt(&self, mt: CrntMsrTick) -> (Vec<String>, Option<CrntMsrTick>) {
        let mut txt_this_time: Vec<String> = Vec::new();
        let mut next_msr_tick = None;
        let msr_exists = |ctxt: &str| ctxt.len() > 5 && ctxt[0..5] == *"!msr(";
        let crnt_msr = mt.msr as usize;
        // 先頭を探す
        if crnt_msr != 0 {
            for crnt in self.loaded_text.iter().enumerate() {
                let ctxt = crnt.1;
                if msr_exists(ctxt) {
                    // !msr() の場合
                    if let Some(msr) = extract_number_from_parentheses(ctxt)
                        && msr >= crnt_msr
                    {
                        next_msr_tick = Some(CrntMsrTick {
                            msr: (msr as i32),
                            tick: 0,
                            tick_for_onemsr: 0,
                            ..Default::default()
                        });
                        break;
                    }
                } else {
                    txt_this_time.push(ctxt.clone());
                }
            }
        }
        if next_msr_tick.is_none() {
            // 0小節目からの再生
            next_msr_tick = Some(CrntMsrTick {
                msr: LAST,
                tick: 0,
                tick_for_onemsr: 0,
                ..Default::default()
            });
        }
        (txt_this_time, next_msr_tick)
    }
    /// ファイル内で !msr() を使ったデータにおいて、
    /// 指定された小節に !msr() があれば、そのデータを返す
    pub fn get_from_msr(&self, mt: CrntMsrTick) -> (Vec<String>, Option<CrntMsrTick>) {
        let mut txt_this_time: Vec<String> = Vec::new();
        let mut next_msr_tick = None;
        let msr_exists = |ctxt: &str| ctxt.len() > 5 && ctxt[0..5] == *"!msr(";
        let crnt_msr = mt.msr as usize;
        let mut sw = false;
        // 先頭を探す
        if crnt_msr != 0 {
            for crnt in self.loaded_text.iter().enumerate() {
                let ctxt = crnt.1;
                if msr_exists(ctxt) {
                    // !msr() の場合
                    if let Some(msr) = extract_number_from_parentheses(ctxt) {
                        if msr == crnt_msr {
                            sw = true;
                        } else if sw {
                            // すでに !msr() が見つかっているので、ここで終了
                            next_msr_tick = Some(CrntMsrTick {
                                msr: (msr as i32),
                                tick: 0,
                                tick_for_onemsr: 0,
                                ..Default::default()
                            });
                            break;
                        }
                    }
                } else if sw {
                    txt_this_time.push(ctxt.clone());
                }
            }
        }
        (txt_this_time, next_msr_tick)
    }
}
