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
    const LOG_FOLDER: &str = "log";
    const LOAD_FOLDER: &str = "load";

    pub fn new() -> Self {
        Self {
            input_lines: Vec::new(),
            history_ptr: 0,
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
                whole_txt += &line.0.to_string();
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
        }
        else {
            println!("No file!");
        }
    }
    pub fn _get_scroll_text(&self, line: usize) -> (String, String) {
        self.input_lines[line].clone()
    }
    pub fn set_scroll_text(&mut self, time: String, cmd: String) -> usize {
        self.input_lines.push((time, cmd));
        self.update_history_ptr()
    }
    pub fn load_and_set_history(&mut self, time: String, fname: &str) -> usize {
        // フォルダ作成
        self.make_folder(Self::LOAD_FOLDER);

        match fs::read_to_string(Self::LOAD_FOLDER.to_string() + "/" + &fname + ".lpn") {
            Ok(content) => {
                for line in content.lines() {
                    if line.len() > 20 {
                        let load_cmd = &line[20..];
                        if load_cmd != "play" && load_cmd != "stop" &&
                            load_cmd != "right1" && load_cmd != "right2" && load_cmd != "left1" && load_cmd != "left2" {
                            self.input_lines.push((time.clone(), line[20..].to_string()));
                        }
                    }
                }
            }
            Err(_err) => println!("Can't open a file"),
        };
        self.update_history_ptr()
    }
    pub fn arrow_up(&mut self) -> Option<(String, usize)> {
        let max_count = self.input_lines.len();
        if self.history_ptr >= 1 {self.history_ptr -= 1;}
        if max_count > 0 && self.history_ptr < max_count {
            Some((self.input_lines[self.history_ptr].1.clone(), self.history_ptr))
        }
        else {None}
    }
    pub fn arrow_down(&mut self) -> Option<(String, usize)> {
        let max_count = self.input_lines.len();
        if self.history_ptr < max_count {self.history_ptr += 1;}
        if max_count > 0 && self.history_ptr < max_count {
            Some((self.input_lines[self.history_ptr].1.clone(), self.history_ptr))
        }
        else if self.history_ptr >= max_count {
            Some(("".to_string(), self.history_ptr))
        }
        else {None}
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