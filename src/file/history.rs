//  Created by Hasebe Masahiko on 2023/06/16.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::lpn_file::*;
use std::fs;
use std::io::Write;
//*******************************************************************
//      History Struct
//*******************************************************************
pub struct History {
    input_lines: Vec<(String, String)>, // (time, command) : 過去の入力履歴
    history_ptr: usize,
}
impl LpnFile for History {}
impl History {
    pub fn new() -> Self {
        Self {
            input_lines: Vec::new(),
            history_ptr: 0,
        }
    }
    /// ログファイルを生成する (num: 何行目からのログを出力するか)
    pub fn gen_log(&mut self, num: usize, fname: String) {
        // 無ければフォルダ作成
        self.make_log_folder();

        // 時間をファイル名に使う
        let fname = if fname.is_empty() {
            //Local::now().format("%Y-%m-%d_%H-%M-%S.lpn").to_string()
            self.default_file_name()
        } else {
            fname + ".lpn"
        };
        let fn_with_path = &(String::from(LOG_FOLDER) + "/" + &fname);
        let file_handler = self.path_str(fn_with_path);
        let display = file_handler.display();
        // log収集
        let mut whole_txt: String = String::new();
        let mut txt_exist = false;
        for (i, line) in self.input_lines.iter().enumerate() {
            if i < num {
                continue;
            }
            whole_txt += &line.1.to_string();
            whole_txt += "\n";
            txt_exist = true;
        }
        if txt_exist {
            // ファイル作成
            let mut file = match fs::File::create(file_handler) {
                Err(why) => panic!("couldn't create {}: {}", display, why),
                Ok(file) => file,
            };
            // ファイル書き込み
            match file.write_all(whole_txt.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => println!("successfully wrote to {}", display),
            }
        } else {
            println!("No text!");
        }
    }
    pub fn _get_scroll_text(&self, line: usize) -> (String, String) {
        self.input_lines[line].clone()
    }
    /// history にコマンドを追加する
    pub fn set_scroll_text(&mut self, prefix: String, cmd: String) -> usize {
        self.input_lines.push((prefix, cmd));
        self.update_history_ptr()
    }
    /// history の中で、上矢印キーが押されたときの処理
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
    /// history の中で、下矢印キーが押されたときの処理
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
    /// history ポインタを最新に更新する
    fn update_history_ptr(&mut self) -> usize {
        self.history_ptr = self.input_lines.len();
        self.history_ptr
    }
}
