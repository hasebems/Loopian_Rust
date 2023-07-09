//  Created by Hasebe Masahiko on 2023/01/22.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

//*******************************************************************
//         User Generate Content
//*******************************************************************
#[derive(Default,Clone)]
pub struct UgContent {
    dt: Vec<Vec<i16>>,
}
impl UgContent {
    pub fn new() -> Self {Self {dt: Vec::new(),}}
    pub fn len(&self) -> usize {self.dt.len()}
    pub fn copy_to(&self) -> UgContent {Self{dt: self.dt.to_vec()}}
    pub fn add_dt(&mut self, new_dt: Vec<i16>) {self.dt.push(new_dt);}
    pub fn get_all(&self) -> Vec<Vec<i16>> {self.dt.clone()}
    pub fn get_msg(&self, msg: usize) -> Vec<i16> {self.dt[msg].clone()}
    pub fn get_dt(&self, msg: usize, element: usize) -> i16 {self.dt[msg][element]}
}

//*******************************************************************
//         Index for element
//*******************************************************************
// element for MSG_PHR
pub const TYPE: usize           = 0;
pub const TICK: usize           = 1;
pub const DURATION: usize       = 2;    // for Note
pub const INFOTP: usize         = 2;    // for Info
pub const NOTE: usize           = 3;
pub const VELOCITY: usize       = 4;
pub const TYPE_NOTE_SIZE: usize = 5;

// element for MSG_CMP
//pub const TYPE: usize         = 0;
//pub const TICK: usize         = 1;
pub const CD_ROOT: usize        = 2;
pub const CD_TABLE: usize       = 3;
pub const TYPE_CHORD_SIZE: usize = 4;
pub const POS: usize            = 3;
pub const _TYPE_DAMPER_SIZE: usize = 4;

// MSG_ANA
//pub const TYPE: usize         = 0;
//pub const TICK: usize         = 1;
//pub const DURATION: usize     = 2;
//pub const NOTE: usize         = 3;
pub const ARP_NTCNT: usize      = 4;
pub const ARP_DIFF: usize       = 5;
pub const TYPE_BEAT_SIZE: usize = 6;

//pub const TYPE: usize         = 0;
pub const EXPR: usize           = 1;
pub const _TYPE_EXPR_SIZE: usize = 2;