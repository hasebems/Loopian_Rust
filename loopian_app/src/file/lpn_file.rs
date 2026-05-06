//  Created by Hasebe Masahiko on 2025/02/15.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use chrono::Local;
use std::fs;
use std::path::Path;

pub const LOG_FOLDER: &str = "log";
pub const LOAD_FOLDER: &str = "load";

pub trait LpnFile {
    /// ファイル名のデフォルト値を返す
    fn default_file_name(&self) -> String {
        Local::now().format("%Y-%m-%d_%H-%M-%S.lpn").to_string()
    }
    /// パス付きファイル名からファイルパスのポインタを返す
    fn path_str<'a>(&self, path_str: &'a str) -> &'a Path {
        Path::new(path_str)
    }
    /// フォルダを作成する
    fn make_folder(&self, folder_name: &str) {
        let path = Path::new(folder_name);
        if !path.is_dir() {
            fs::create_dir_all(path).unwrap();
        }
    }
    /// logフォルダを作成する
    fn make_log_folder(&self) {
        self.make_folder(LOG_FOLDER)
    }
    /// ロードファイル名を生成する
    fn gen_lpn_file_name(&self, fname: String, path: Option<&str>) -> String {
        self.make_folder(LOAD_FOLDER); // フォルダ作成
        let mut real_path = LOAD_FOLDER.to_string();
        if let Some(lp) = path {
            real_path = real_path + "/" + lp;
        }
        println!("Path: {}", real_path);
        println!("File: {}", fname.clone() + ".lpn");
        real_path + "/" + &fname + ".lpn"
    }
}
