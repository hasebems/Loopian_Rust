//  Created by Hasebe Masahiko on 2023/06/16.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use chrono::Local;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::txt_common::*;
use crate::elapse::tickgen::CrntMsrTick;
use crate::lpnlib::*;

pub struct History {
    input_lines: Vec<(String, String)>,
    history_ptr: usize,
    loaded_text: Vec<String>,
}

impl History {
    const LOG_FOLDER: &'static str = "log";
    const LOAD_FOLDER: &'static str = "load";

    pub fn new() -> Self {
        Self {
            input_lines: Vec::new(),
            history_ptr: 0,
            loaded_text: Vec::new(),
        }
    }
    pub fn gen_log(&mut self) {
        // フォルダ作成
        self.make_folder(Self::LOG_FOLDER);

        // 時間をファイル名に使う
        let file = Local::now().format("%Y-%m-%d_%H-%M-%S.lpn").to_string();
        let mut path_str = String::from(Self::LOG_FOLDER);
        path_str += "/";
        path_str += &file;
        let path = Path::new(&path_str);
        let display = path.display();
        // log収集
        let mut whole_txt: String = String::new();
        let mut txt_exist = false;
        for line in self.input_lines.iter() {
            if line.0.len() > 0 && line.1 != "quit" {
                //whole_txt += &line.0.to_string(); // 日付時刻の挿入
                whole_txt += &line.1.to_string();
                whole_txt += "\n";
                txt_exist = true;
            }
        }
        if txt_exist {
            // ファイル作成
            let mut file = match File::create(&path) {
                Err(why) => panic!("couldn't create {}: {}", display, why),
                Ok(file) => file,
            };
            // ファイル書き込み
            match file.write_all(whole_txt.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => println!("successfully wrote to {}", display),
            }
        } else {
            println!("No file!");
        }
    }
    pub fn _get_scroll_text(&self, line: usize) -> (String, String) {
        self.input_lines[line].clone()
    }
    pub fn set_scroll_text(&mut self, time: String, cmd: String) -> usize {
        self.input_lines.push((time.clone(), cmd));
        self.update_history_ptr()
    }
    pub fn load_lpn(&mut self, fname: &str, path: Option<String>) -> bool {
        self.loaded_text = Vec::new();
        self.make_folder(Self::LOAD_FOLDER); // フォルダ作成
        let mut real_path = Self::LOAD_FOLDER.to_string();
        if let Some(lp) = path {
            real_path = real_path + "/" + &lp;
        }
        println!("Path: {}", real_path);
        match fs::read_to_string(real_path + "/" + &fname + ".lpn") {
            Ok(content) => {
                for line in content.lines() {
                    let mut comment = false;
                    if line.len() > 1 {
                        // コメントでないか、過去の 2023.. が書かれてないか
                        let notxt = line[0..2].to_string();
                        if notxt == "//" || notxt == "20" {
                            comment = true;
                        }
                    }
                    if line.len() > 0 && !comment {
                        self.loaded_text.push(line.to_string());
                    }
                }
            }
            Err(_err) => println!("Can't open a file"),
        };
        if self.loaded_text.len() != 0 {
            true
        } else {
            false
        }
    }
    pub fn get_loaded_text(&self, mt: CrntMsrTick) -> (Vec<String>, Option<CrntMsrTick>) {
        let mut txt_this_time: Vec<String> = Vec::new();
        let mut idx: usize = 0;
        // 先頭を探す
        if mt.msr != 0 {
            for crnt in self.loaded_text.iter().enumerate() {
                let ctxt = crnt.1;
                if ctxt.len() > 6 && ctxt[0..6] == *"!wait(" {
                    let msr = extract_number_from_parentheses(ctxt);
                    if msr == mt.msr.try_into().unwrap_or(0) {
                        idx = crnt.0 + 1;
                        break;
                    }
                }
            }
        }
        // ここから記録
        for n in idx..self.loaded_text.len() {
            let ctxt = &self.loaded_text[n];
            if ctxt.len() > 6 && ctxt[0..6] == *"!wait(" {
                let msr = extract_number_from_parentheses(ctxt);
                println!("YYY{:?}",txt_this_time);
                return (
                    txt_this_time,
                    Some(CrntMsrTick {
                        msr: msr.try_into().unwrap_or(0),
                        tick: 0,
                        tick_for_onemsr: 0,
                    }),
                );
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
            }),
        )
    }
    pub fn arrow_up(&mut self) -> Option<(String, usize)> {
        let max_count = self.input_lines.len();
        if self.history_ptr >= 1 {
            self.history_ptr -= 1;
        }
        if max_count > 0 && self.history_ptr < max_count {
            Some((
                self.input_lines[self.history_ptr].1.clone(),
                self.history_ptr,
            ))
        } else {
            None
        }
    }
    pub fn arrow_down(&mut self) -> Option<(String, usize)> {
        let max_count = self.input_lines.len();
        if self.history_ptr < max_count {
            self.history_ptr += 1;
        }
        if max_count > 0 && self.history_ptr < max_count {
            Some((
                self.input_lines[self.history_ptr].1.clone(),
                self.history_ptr,
            ))
        } else if self.history_ptr >= max_count {
            Some(("".to_string(), self.history_ptr))
        } else {
            None
        }
    }
    fn update_history_ptr(&mut self) -> usize {
        self.history_ptr = self.input_lines.len();
        self.history_ptr
    }
    fn make_folder(&self, folder_name: &str) {
        let path = Path::new(folder_name);
        if !path.is_dir() {
            fs::create_dir_all(path).unwrap();
        }
    }
}
