//  Created by Hasebe Masahiko on 2025/02/14.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use super::lpn_file::*;
use crate::cmd::txt_common::*;
use crate::lpnlib::*;
use std::fs;

/// Convert a file to a timeline file
/// @msr(n) -> !msr(n)
struct CnvFile {
    raw_lines: Vec<String>,
    part_lines: [Vec<String>; MAX_KBD_PART],
}

impl CnvFile {
    pub fn new() -> Self {
        let part_lines: [Vec<String>; MAX_KBD_PART] = Default::default();
        Self {
            raw_lines: Vec::new(),
            part_lines,
        }
    }
    pub fn input_file(&mut self, fp_string: String) {
        let fp = self.path_str(&fp_string);
        match fs::read_to_string(fp) {
            Ok(content) => {
                let mut inside_blk = false;
                let mut ptnum: Option<usize> = None;
                for line in content.lines() {
                    if line.len() > 1 {
                        let top_char = line[0..2].to_string();
                        if top_char == "//" || top_char == "20" || top_char == "!l" {
                            // コメントでないか、過去の 2023.. が書かれてないか、loadではないか
                            continue;
                        } else if line.len() >= 4 && &line[0..4] == "!rd(" {
                            // 読み飛ばす
                            continue;
                        } else if line.len() >= 5 && &line[0..5] == "!blk(" {
                            // 読み飛ばす
                            inside_blk = true;
                            continue;
                        } else if !line.is_empty() && !inside_blk {
                            if line == "L1" {
                                ptnum = Some(0);
                            } else if line == "L2" {
                                ptnum = Some(1);
                            } else if line == "R1" {
                                ptnum = Some(2);
                            } else if line == "R2" {
                                ptnum = Some(3);
                            } else if let Some(p) = ptnum {
                                self.part_lines[p].push(line.to_string());
                            } else {
                                self.raw_lines.push(line.to_string());
                            }
                        }
                    } else if line.len() <= 1 && inside_blk {
                        inside_blk = false;
                    }
                }
            }
            Err(_err) => println!("Can't open a file"),
        };
    }
    pub fn output_file(&mut self, fp_string: String) {
        let fp = self.path_str(&fp_string);
        let mut output = String::from("");
        for line in &self.raw_lines {
            output.push_str(line);
            output.push('\n');
        }
        let mut msr: usize = 0;
        let mut ptidx: [Option<usize>; MAX_KBD_PART] = [Some(0); MAX_KBD_PART];
        let mut bp;
        loop {
            bp = 0;
            let mut msr_out: String = "".to_string();
            for (i, idx) in ptidx.iter_mut().enumerate().take(MAX_KBD_PART) {
                if let Some(index) = idx {
                    *idx = self.put_part_line(i, *index, msr, &mut msr_out);
                } else {
                    bp += 1;
                }
            }
            if bp == MAX_KBD_PART {
                println!("Final Measure: {}", msr);
                break;
            } else if !msr_out.is_empty() {
                let msr_str = "!msr(".to_string() + msr.to_string().as_str() + ")\n";
                output.push('\n');
                output.push_str(&msr_str);
                output.push_str(&msr_out);
                println!("Recorded Measure: {}", msr);
            }
            msr += 1;
        }
        match fs::write(fp, output) {
            Ok(_) => println!("Success"),
            Err(_err) => println!("Can't write a file"),
        };
    }
    fn put_part_line(
        &mut self,
        part: usize,
        idx: usize,
        msr: usize,
        output: &mut String,
    ) -> Option<usize> {
        const PTSTR_TBL: [&str; MAX_KBD_PART] = ["L1.", "L2.", "R1.", "R2."];
        let ptstr = PTSTR_TBL[part];
        if let Some(line) = self.part_lines[part].get(idx) {
            let separated_line = split_by('=', line.to_string());
            let mut ptidx = idx;
            if let Some(msr_num) = extract_number_from_parentheses(&separated_line[0]) {
                if msr_num == msr {
                    let phr = ptstr.to_string() + &separated_line[1];
                    output.push_str(&phr);
                    output.push('\n');
                    ptidx += 1;
                }
            }
            Some(ptidx)
        } else {
            None
        }
    }
}

impl LpnFile for CnvFile {}

pub fn convert_to_timeline(fname: String, path: Option<&str>) {
    let mut cnv = CnvFile::new();
    let file_path = cnv.gen_lpn_file_name(fname, path);
    cnv.input_file(file_path.clone());
    let idx = file_path.len() - 4;
    cnv.output_file(file_path[..idx].to_string() + "_tl.lpn");
}
