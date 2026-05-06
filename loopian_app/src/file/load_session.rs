//  Created by Hasebe Masahiko on 2026/03/21.
//  Copyright (c) 2026 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::load::*;
use crate::common::lpnlib::*;
use crate::common::txt_common::*;
use crate::elapse::tickgen::CrntMsrTick;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCmdType {
    Phrase,
    Realtime,
    Any,
}

#[derive(Debug, Clone)]
pub struct SessionDispatch {
    pub cmd_type: SessionCmdType,
    pub loaded: Vec<String>,
    pub next_msr: Option<i32>,
}

pub enum FileAction {
    Dispatch {
        dispatch: SessionDispatch,
        playable: bool,
    },
    RunRawCommands(Vec<String>),
    SetInputText(String),
    SetMeasure(i16),
    ClearEngineData,
}

pub struct FileCommandPlan {
    pub actions: Vec<FileAction>,
    pub notice: Option<FileNotice>,
}

pub enum FileCommandResult {
    NotHandled,
    Handled(FileCommandPlan),
}

pub enum FileNotice {
    LoadedFromFile(String),
    NoFile,
    LoadFailed,
    AllDataCleared,
    NoSuchBlock,
    NoData,
    NoSuchMeasure,
    NoFileLoaded,
}

pub enum LoadFileResult {
    Loaded {
        file_name: String,
        dispatch: Option<SessionDispatch>,
    },
    NoFile,
    LoadFailed,
}

pub enum LoadByMsrResult {
    NoData,
    Loaded {
        realtime: SessionDispatch,
        any: Option<SessionDispatch>,
        set_measure: i16,
    },
}

#[derive(PartialEq, Eq)]
enum AutoLoadState {
    BeforeLoading,
    Reached,
    PhraseLoaded,
}

pub struct LoadSession {
    next_msr_tick: Option<CrntMsrTick>,
    auto_load_buffer: (Vec<String>, Option<CrntMsrTick>),
    auto_load_state: AutoLoadState,
    load_buffer: LoadBuffer,
}

impl LoadSession {
    pub fn new() -> Self {
        Self {
            next_msr_tick: None,
            auto_load_buffer: (vec![], None),
            auto_load_state: AutoLoadState::BeforeLoading,
            load_buffer: LoadBuffer::new(),
        }
    }
    pub fn clear(&mut self) {
        self.reset_runtime_state();
        self.load_buffer.clear_file_name();
    }
    pub fn has_file_name(&self) -> bool {
        self.load_buffer.get_file_name().is_some()
    }
    pub fn set_file_name(&mut self, file_name: String) {
        self.load_buffer.set_file_name(file_name);
    }
    pub fn read_line_from_lpn(&self, path: Option<&str>, num: usize) -> Option<String> {
        self.load_buffer.read_line_from_lpn(path, num)
    }
    pub fn get_loaded_blk(&self, blk_name: &str) -> Vec<String> {
        self.load_buffer.get_loaded_blk(blk_name)
    }
    pub fn load_file(&mut self, path: Option<&str>, playable: bool) -> LoadFileResult {
        if let Some(file_name) = self.load_buffer.get_file_name() {
            if self.load_buffer.load_lpn(path) {
                if playable {
                    LoadFileResult::Loaded {
                        file_name,
                        dispatch: None,
                    }
                } else {
                    self.clear();
                    let loaded = self
                        .load_buffer
                        .get_from_msr_to_next(CrntMsrTick::default());
                    self.next_msr_tick = loaded.1;
                    LoadFileResult::Loaded {
                        file_name,
                        dispatch: Some(SessionDispatch {
                            cmd_type: SessionCmdType::Any,
                            loaded: loaded.0,
                            next_msr: None,
                        }),
                    }
                }
            } else {
                LoadFileResult::LoadFailed
            }
        } else {
            LoadFileResult::NoFile
        }
    }
    pub fn prepare_play_from_top(&mut self) -> Option<SessionDispatch> {
        if self.has_file_name() {
            self.reset_runtime_state();
            let loaded = self
                .load_buffer
                .get_from_msr_to_next(CrntMsrTick::default());
            self.next_msr_tick = loaded.1;
            Some(SessionDispatch {
                cmd_type: SessionCmdType::Any,
                loaded: loaded.0,
                next_msr: Some(1),
            })
        } else {
            None
        }
    }
    pub fn load_by_msr(&mut self, msr: usize) -> LoadByMsrResult {
        let mt = CrntMsrTick {
            msr: msr as i32,
            ..Default::default()
        };

        let loaded = self.load_buffer.get_from_0_to_mt(mt);
        if loaded.0.is_empty() {
            return LoadByMsrResult::NoData;
        }

        self.next_msr_tick = loaded.1;
        let realtime = SessionDispatch {
            cmd_type: SessionCmdType::Realtime,
            loaded: loaded.0,
            next_msr: None,
        };

        let loaded = self.load_buffer.get_from_msr(mt);
        let any = if loaded.0.is_empty() {
            None
        } else {
            Some(SessionDispatch {
                cmd_type: SessionCmdType::Any,
                loaded: loaded.0,
                next_msr: Some(msr as i32),
            })
        };

        self.auto_load_buffer = (vec![], None);
        self.auto_load_state = AutoLoadState::BeforeLoading;

        let set_measure = if msr > 0 { (msr as i16) - 1 } else { 0 };
        LoadByMsrResult::Loaded {
            realtime,
            any,
            set_measure,
        }
    }
    pub fn poll_auto_load(&mut self, crnt: CrntMsrTick, rest_tick: i32) -> Option<SessionDispatch> {
        if let Some(next_mt) = self.next_msr_tick
            && next_mt.msr != LAST
            && next_mt.msr > 0
            && next_mt.msr - 1 == crnt.msr
        {
            if self.auto_load_state == AutoLoadState::BeforeLoading {
                self.auto_load_buffer = self.load_buffer.get_from_msr_to_next(next_mt);
                self.auto_load_state = AutoLoadState::Reached;
            } else if self.auto_load_state == AutoLoadState::Reached && crnt.tick > rest_tick {
                self.auto_load_state = AutoLoadState::PhraseLoaded;
                return Some(SessionDispatch {
                    cmd_type: SessionCmdType::Phrase,
                    loaded: self.auto_load_buffer.0.clone(),
                    next_msr: Some(next_mt.msr),
                });
            } else if self.auto_load_state == AutoLoadState::PhraseLoaded
                && crnt.tick_for_onemsr - crnt.tick < rest_tick
            {
                self.next_msr_tick = self.auto_load_buffer.1;
                self.auto_load_state = AutoLoadState::BeforeLoading;
                return Some(SessionDispatch {
                    cmd_type: SessionCmdType::Realtime,
                    loaded: self.auto_load_buffer.0.clone(),
                    next_msr: None,
                });
            }
        }
        None
    }

    pub fn handle_file_command(&mut self, itxt: &str, path: Option<&str>) -> FileCommandResult {
        if itxt.starts_with("!l") || itxt.starts_with("!load") {
            let actions: Vec<FileAction> = Vec::new();
            let fnx = split_by('.', itxt.to_string());
            if fnx.len() >= 2 {
                self.set_file_name(fnx[1].clone());
            }
            let notice = match self.load_file(path, true) {
                LoadFileResult::Loaded { file_name, .. } => {
                    Some(FileNotice::LoadedFromFile(file_name))
                }
                LoadFileResult::NoFile => Some(FileNotice::NoFile),
                LoadFileResult::LoadFailed => Some(FileNotice::LoadFailed),
            };
            return FileCommandResult::Handled(FileCommandPlan { actions, notice });
        }

        if itxt.starts_with("!h") || itxt.starts_with("!history") {
            let fnx = split_by('.', itxt.to_string());
            if fnx.len() >= 2 {
                self.set_file_name(fnx[1].clone());
            }
            let mut actions: Vec<FileAction> = Vec::new();
            let notice = match self.load_file(path, false) {
                LoadFileResult::Loaded {
                    file_name,
                    dispatch,
                } => {
                    if let Some(d) = dispatch {
                        actions.push(FileAction::Dispatch {
                            dispatch: d,
                            playable: false,
                        });
                    }
                    Some(FileNotice::LoadedFromFile(file_name))
                }
                LoadFileResult::NoFile => Some(FileNotice::NoFile),
                LoadFileResult::LoadFailed => Some(FileNotice::LoadFailed),
            };
            return FileCommandResult::Handled(FileCommandPlan { actions, notice });
        }

        if itxt == "!clear" || itxt == "!clr" || itxt == "!c" {
            self.clear();
            return FileCommandResult::Handled(FileCommandPlan {
                actions: vec![FileAction::ClearEngineData],
                notice: Some(FileNotice::AllDataCleared),
            });
        }

        if itxt.starts_with("!r") || itxt.starts_with("!rd") || itxt.starts_with("!read") {
            let num = if itxt.contains('(') {
                extract_number_from_parentheses(itxt).unwrap_or(0)
            } else if itxt.len() >= 3 {
                itxt[2..].parse::<usize>().unwrap_or(0)
            } else {
                0
            };
            let mut actions: Vec<FileAction> = Vec::new();
            if let Some(cmd) = self.read_line_from_lpn(path, num) {
                actions.push(FileAction::SetInputText(cmd));
            }
            return FileCommandResult::Handled(FileCommandPlan {
                actions,
                notice: None,
            });
        }

        if itxt.starts_with("!blk(") {
            let blk_name = extract_texts_from_parentheses(itxt);
            let loaded_blk = self.get_loaded_blk(blk_name);
            if loaded_blk.is_empty() {
                return FileCommandResult::Handled(FileCommandPlan {
                    actions: Vec::new(),
                    notice: Some(FileNotice::NoSuchBlock),
                });
            }
            return FileCommandResult::Handled(FileCommandPlan {
                actions: vec![FileAction::RunRawCommands(loaded_blk)],
                notice: None,
            });
        }

        if itxt.starts_with("!msr(") {
            if let Some(msr_num) = extract_number_from_parentheses(itxt) {
                match self.load_by_msr(msr_num) {
                    LoadByMsrResult::NoData => {
                        return FileCommandResult::Handled(FileCommandPlan {
                            actions: Vec::new(),
                            notice: Some(FileNotice::NoData),
                        });
                    }
                    LoadByMsrResult::Loaded {
                        realtime,
                        any,
                        set_measure,
                    } => {
                        let mut actions: Vec<FileAction> = vec![FileAction::Dispatch {
                            dispatch: realtime,
                            playable: true,
                        }];
                        if let Some(d) = any {
                            actions.push(FileAction::Dispatch {
                                dispatch: d,
                                playable: true,
                            });
                        }
                        actions.push(FileAction::SetMeasure(set_measure));
                        return FileCommandResult::Handled(FileCommandPlan {
                            actions,
                            notice: None,
                        });
                    }
                }
            }
            return FileCommandResult::Handled(FileCommandPlan {
                actions: Vec::new(),
                notice: Some(FileNotice::NoSuchMeasure),
            });
        }

        if itxt == "!play" || itxt == "!p" {
            if let Some(dispatch) = self.prepare_play_from_top() {
                return FileCommandResult::Handled(FileCommandPlan {
                    actions: vec![FileAction::Dispatch {
                        dispatch,
                        playable: true,
                    }],
                    notice: None,
                });
            }
            return FileCommandResult::Handled(FileCommandPlan {
                actions: Vec::new(),
                notice: Some(FileNotice::NoFileLoaded),
            });
        }

        FileCommandResult::NotHandled
    }

    fn reset_runtime_state(&mut self) {
        self.next_msr_tick = None;
        self.auto_load_buffer = (vec![], None);
        self.auto_load_state = AutoLoadState::BeforeLoading;
    }
}
