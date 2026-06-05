//  Created by Hasebe Masahiko on 2023/11/12.
//  Copyright (c) 2023 Hasebe Masahiko.
//  Released under the MIT license
//  https://opensource.org/licenses/mit-license.php
//
use super::guiev::{GuiEv, INDC_METER};
use super::view_waterripple::WaterRipple;

pub use loopian_graphic_api::generative_view::*;

pub fn generate_graphic_msg(input_msgs: Vec<String>) -> (String, GraphicMsg) {
    if input_msgs.len() >= 2 && input_msgs[1] == "list" {
        let mut patterns = vec!["ripple".to_string()];
        patterns.extend(loopian_graphics::builtin_graphic_names());
        patterns.sort();
        patterns.dedup();
        return (patterns.join("\n"), GraphicMsg::NoMsg);
    }

    loopian_graphic_api::generative_view::generate_graphic_msg(input_msgs)
}

pub fn get_view_instance(
    guiev: &mut GuiEv,
    crnt_time: f32,
    gmsg: &GraphicMsg,
    gmode: GraphMode,
    font_nrm: nannou::text::Font,
) -> Option<Box<dyn GenerativeView>> {
    match gmsg {
        GraphicMsg::Pattern { name, .. } if name == "ripple" => {
            Some(Box::new(WaterRipple::new(gmode)))
        }
        _ => {
            let meter_text = guiev.get_indicator(INDC_METER);
            loopian_graphics::get_view_instance(crnt_time, gmsg, gmode, meter_text, font_nrm)
        }
    }
}
