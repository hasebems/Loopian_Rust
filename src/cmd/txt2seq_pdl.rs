//  Created by Hasebe Masahiko on 2025/10/09.
//  Copyright (c) 2025 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//

use crate::lpnlib::*;

pub fn complement_pedal(input_text: String) -> Vec<String> {
    if let Some(n2) = input_text.find(']')
        && let Some(n1) = input_text[..n2].find('[')
        && n1 < n2
    {
        let inner_text = &input_text[n1 + 1..n2];
        let mut dt = inner_text.to_string();
        if inner_text.ends_with("//") {
            dt.pop();
            dt.pop();
            dt += "LPEND";
        }
        // `!` が見つかったら、直後に `/` を挿入（全ての `!` に対して）
        // let modified_text = inner_text.replace("!", "!/");
        let mut parts = dt
            .split("/")
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        // 前後の部分を追加
        let mut result = Vec::new();
        result.append(&mut parts);
        return result;
    }
    println!(">>>complement_pedal: empty!");
    Vec::new()
}
pub fn recombine_to_internal_format_pedal(
    raw: &[String],
    tick_for_onemsr: i32,
    tick_for_beat: i32,
) -> (i32, bool, Vec<PhrEvt>) {
    let max_beat = (tick_for_onemsr / tick_for_beat) as i16;
    let mut pedal_on = false;
    let mut result = Vec::new();
    //do_loop
    let mut do_loop = true;
    let mut raw_vec = raw.to_vec();
    if raw_vec.last().map(|s| s.contains("LPEND")).unwrap_or(false) {
        do_loop = false;
        if let Some(last) = raw_vec.last_mut() {
            *last = last.replace("LPEND", "");
        }
    }
    for (i, s) in raw_vec.iter().enumerate() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (pdl_evt, pdl) = pedal_in_one_msr(trimmed, max_beat, i as i16, pedal_on);
        result.extend(pdl_evt);
        pedal_on = pdl;
    }
    (raw_vec.len() as i32 * tick_for_onemsr, do_loop, result)
}
fn pedal_in_one_msr(
    segment: &str,
    max_beat: i16,
    msr: i16,
    pedal_on: bool, // 前の小節の最後の状態
) -> (Vec<PhrEvt>, bool) {
    #[cfg(feature = "verbose")]
    println!("Analyzing pedal segment: {}", segment);
    let mut result = Vec::new();
    let mut position_before = if pedal_on {
        PedalPos::Full
    } else {
        PedalPos::Off
    };
    let mut beat = 0;

    // segment を解析して PhrEvt に変換し、result に追加する処理を実装
    for c in segment.chars() {
        let position = match c {
            '_' => PedalPos::Full,
            '-' => PedalPos::Half,
            '*' => PedalPos::Off,
            ',' => {
                // 拍間で瞬間的に離す
                push_pedal_event(
                    &mut result,
                    msr,
                    beat,
                    false,
                    PedalPos::Off,
                    position_before,
                );
                position_before = PedalPos::Off;
                continue;
            }
            ';' => {
                // 拍間で瞬間的にハーフにする
                push_pedal_event(
                    &mut result,
                    msr,
                    beat,
                    false,
                    PedalPos::Half,
                    position_before,
                );
                position_before = PedalPos::Half;
                continue;
            }
            _ => continue, // 無視する文字
        };
        if position != position_before {
            push_pedal_event(&mut result, msr, beat, true, position, position_before);
            position_before = position;
        }
        beat += 1;
        if beat >= max_beat {
            break;
        }
    }

    // 小節の最後の処理
    let pedal_on_return = if segment.ends_with('!') {
        position_before != PedalPos::Off
    } else {
        if position_before != PedalPos::Off {
            push_pedal_event(
                &mut result,
                msr,
                max_beat - 1,
                false,
                PedalPos::Off,
                position_before,
            );
        }
        false
    };
    (result, pedal_on_return)
}
fn push_pedal_event(
    result: &mut Vec<PhrEvt>,
    msr: i16,
    beat: i16,
    front: bool,
    position: PedalPos,
    _position_before: PedalPos,
) {
    #[cfg(feature = "verbose")]
    println!(
        "Pedal change at msr {}, beat {}{}: {:?} -> {:?}",
        msr,
        beat,
        if front { "(f)" } else { "(r)" },
        _position_before,
        position
    );
    result.push(PhrEvt::Damper(PedalEvt {
        msr,
        beat,
        front,
        position,
    }));
}
