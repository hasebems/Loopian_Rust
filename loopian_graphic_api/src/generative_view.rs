use nannou::prelude::*;

use crate::Resize;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GraphicMsg {
    What,
    NoMsg,
    LightMode,
    DarkMode,
    TextVisibleCtrl,
    Title(String, String),
    Pattern { name: String, arg: Option<String> },
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GraphMode {
    Dark,
    Light,
}

pub fn generate_graphic_msg(input_msgs: Vec<String>) -> (String, GraphicMsg) {
    if input_msgs.len() < 2 {
        return ("what?".to_string(), GraphicMsg::What);
    }
    if input_msgs[1] == "light" {
        ("Changed Graphic!".to_string(), GraphicMsg::LightMode)
    } else if input_msgs[1] == "dark" {
        ("Changed Graphic!".to_string(), GraphicMsg::DarkMode)
    } else if input_msgs[1] == "title" {
        let txt = extract_texts_from_parentheses(&input_msgs[1]);
        let txts = txt.split(',').collect::<Vec<&str>>();
        let title_txt = txts.first().unwrap_or(&"");
        let subtitle_txt = txts.get(1).unwrap_or(&"");
        (
            format!("Set Title: {}", title_txt),
            GraphicMsg::Title(title_txt.to_string(), subtitle_txt.to_string()),
        )
    } else {
        let (name, arg) = split_name_and_arg(&input_msgs[1]);
        if name.is_empty() {
            ("what?".to_string(), GraphicMsg::What)
        } else {
            (
                "Changed Graphic!".to_string(),
                GraphicMsg::Pattern {
                    name: name.to_string(),
                    arg,
                },
            )
        }
    }
}

pub trait GenerativeView {
    fn update_model(&mut self, crnt_time: f32, rs: Resize);
    fn note_on(&mut self, _nt: i32, _vel: i32, _pt: i32, _tm: f32) {}
    fn on_beat(&mut self, _bt: i32, _ct: f32, _dt: f32) {}
    fn set_mode(&mut self, _mode: GraphMode) {}
    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize);
}

pub trait NoteObj {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool;
    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize);
}

pub trait BeatObj {
    fn update_model(&mut self, crnt_time: f32, rs: Resize) -> bool;
    fn disp(&self, draw: Draw, crnt_time: f32, rs: Resize);
}

fn extract_texts_from_parentheses(text: &str) -> String {
    if let (Some(start), Some(end)) = (text.find('('), text.rfind(')'))
        && start + 1 < end
    {
        return text[start + 1..end].to_string();
    }
    "".to_string()
}

fn split_name_and_arg(text: &str) -> (&str, Option<String>) {
    if let Some(idx) = text.find('(') {
        let name = text[..idx].trim();
        let arg = extract_texts_from_parentheses(text);
        if arg.is_empty() {
            (name, None)
        } else {
            (name, Some(arg))
        }
    } else {
        (text.trim(), None)
    }
}
