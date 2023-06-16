//  Created by Hasebe Masahiko on 2023/06/16.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use chrono::Local;

pub struct History {
    input_lines: Vec<(String, String)>,
    history_ptr: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            input_lines: Vec::new(),
            history_ptr: 0,
        }
    }
    pub fn gen_log(&mut self) {
        // フォルダ作成
        let path = Path::new("log");
        if !path.is_dir() {
            fs::create_dir_all(path).unwrap();
        }
        // 時間をファイル名に使う
        let file = Local::now().format("%Y-%m-%d_%H-%M-%S.txt").to_string();
        let path_str = "log/".to_string() + &file;
        let path = Path::new(&path_str);
        let display = path.display();
        // log収集
        let mut whole_txt: String = String::new();
        for line in self.input_lines.iter() {
            if line.0.len() > 0 {
                whole_txt += &line.0.to_string();
                whole_txt += &line.1.to_string();
                whole_txt += "\n";
            }
        }
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
    }
    pub fn _get_scroll_text(&self, line: usize) -> (String, String) {
        self.input_lines[line].clone()
    }
    pub fn set_scroll_text(&mut self, time: String, cmd: String) {
        self.input_lines.push((time, cmd));
        self.history_ptr = self.input_lines.len();
    }
    pub fn arrow_up(&mut self) -> Option<String> {
        let max_count = self.input_lines.len();
        if self.history_ptr >= 1 {self.history_ptr -= 1;}
        if max_count > 0 && self.history_ptr < max_count {
            Some(self.input_lines[self.history_ptr].1.clone())
        }
        else {None}
    }
    pub fn arrow_down(&mut self) -> Option<String> {
        let max_count = self.input_lines.len();
        if self.history_ptr < max_count {self.history_ptr += 1;}
        if max_count > 0 && self.history_ptr < max_count {
            Some(self.input_lines[self.history_ptr].1.clone())
        }
        else if self.history_ptr >= max_count {
            Some("".to_string())
        }
        else {None}
    }
}