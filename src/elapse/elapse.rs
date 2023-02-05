//  Created by Hasebe Masahiko on 2023/01/31
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

pub trait Elapse {
    fn id(&self) -> u32;            // id を得る
    fn prio(&self) -> u32;          // priority を得る
    fn next(&self) -> (i32, u32);   // 次に呼ばれる小節番号、Tick数を返す
    fn start(&mut self);            // User による start/play 時にコールされる
    fn stop(&mut self);             // User による stop 時にコールされる
    fn fine(&mut self);             // User による fine があった次の小節先頭でコールされる
    fn process(&mut self, msr: i32, tick: u32);    // 再生 msr/tick に達したらコールされる
    fn destroy_me(&self) -> bool;   // 自クラスが役割を終えた時に True を返す
}