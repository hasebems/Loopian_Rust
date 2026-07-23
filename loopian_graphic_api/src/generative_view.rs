use nannou::prelude::*;
use std::sync::{Mutex, OnceLock};

use crate::Resize;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GraphicPatternSpec {
    pub name: String,
    pub arg: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GraphicMsg {
    What,
    NoMsg,
    LightMode,
    DarkMode,
    TextVisibleCtrl,
    Title(String, String),
    Pattern { name: String, arg: Option<String> },
    AutoPattern {
        bar: usize,
        patterns: Vec<GraphicPatternSpec>,
    },
    AutoOff,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GraphMode {
    Dark,
    Light,
}

type GraphicNameResolver = fn(usize) -> Option<&'static str>;

static GRAPHIC_NAME_RESOLVER: OnceLock<Mutex<Option<GraphicNameResolver>>> = OnceLock::new();

fn graphic_name_resolver() -> &'static Mutex<Option<GraphicNameResolver>> {
    GRAPHIC_NAME_RESOLVER.get_or_init(|| Mutex::new(None))
}

pub fn register_graphic_name_resolver(resolver: GraphicNameResolver) {
    let mut stored = graphic_name_resolver()
        .lock()
        .expect("Graphic name resolver mutex poisoned");
    *stored = Some(resolver);
}

pub fn parse_graphic_msg(input_msgs: Vec<String>) -> (String, GraphicMsg) {
    if input_msgs.len() < 2 {
        return ("what?".to_string(), GraphicMsg::What);
    }
    if input_msgs[1] == "light" {
        ("Changed Graphic!".to_string(), GraphicMsg::LightMode)
    } else if input_msgs[1] == "dark" {
        ("Changed Graphic!".to_string(), GraphicMsg::DarkMode)
    } else if input_msgs[1].starts_with("title") {
        let txt = extract_texts_from_parentheses(&input_msgs[1]);
        let txts = txt.split(',').collect::<Vec<&str>>();
        let title_txt = txts.first().unwrap_or(&"");
        let subtitle_txt = txts.get(1).unwrap_or(&"");
        (
            format!("Set Title: {}", title_txt),
            GraphicMsg::Title(title_txt.to_string(), subtitle_txt.to_string()),
        )
    } else if input_msgs[1].starts_with("auto") {
        auto_message(&input_msgs[1])
    } else if let Some(ptn_spec) = resolve_ptn_spec(&input_msgs[1]) {
        pattern_message(&ptn_spec)
    } else {
        pattern_message(&input_msgs[1])
    }
}

fn auto_message(text: &str) -> (String, GraphicMsg) {
    let txt = extract_texts_from_parentheses(text);
    let params = txt.split(',').map(|x| x.trim()).collect::<Vec<&str>>();
    if params.is_empty() {
        return ("what?".to_string(), GraphicMsg::What);
    }

    let bar = params[0].parse::<usize>().unwrap_or(0);
    if bar == 0 {
        return ("Graphic auto mode off.".to_string(), GraphicMsg::AutoOff);
    }
    if params.len() < 2 {
        return ("what?".to_string(), GraphicMsg::What);
    }

    let patterns = parse_auto_pattern_specs(params[1]);
    if patterns.is_empty() {
        ("what?".to_string(), GraphicMsg::What)
    } else {
        (
            format!("Graphic auto mode on: every {} bar(s).", bar),
            GraphicMsg::AutoPattern { bar, patterns },
        )
    }
}

fn parse_auto_pattern_specs(ptn_seq: &str) -> Vec<GraphicPatternSpec> {
    ptn_seq
        .split('-')
        .filter_map(resolve_graphic_spec)
        .collect()
}

fn resolve_graphic_spec(text: &str) -> Option<GraphicPatternSpec> {
    let spec_text = if text.starts_with("ptn") {
        resolve_ptn_spec(text)?
    } else if text
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        resolve_numeric_ptn_spec(text)?
    } else {
        text.trim().to_string()
    };

    pattern_spec_from_text(&spec_text)
}

fn resolve_numeric_ptn_spec(text: &str) -> Option<String> {
    let (id_text, arg) = split_name_and_arg(text);
    let id = id_text.trim().parse::<usize>().ok()?;

    let resolver = graphic_name_resolver()
        .lock()
        .expect("Graphic name resolver mutex poisoned");
    let graphic_name = resolver.as_ref().and_then(|resolve| resolve(id))?;

    Some(match arg {
        Some(arg) if !arg.trim().is_empty() => format!("{}({})", graphic_name, arg),
        _ => graphic_name.to_string(),
    })
}

fn resolve_ptn_spec(text: &str) -> Option<String> {
    if !text.starts_with("ptn") {
        return None;
    }

    let txt = extract_texts_from_parentheses(text);
    let (id_text, arg) = split_name_and_arg(&txt);
    let id = id_text.trim().parse::<usize>().ok()?;

    let resolver = graphic_name_resolver()
        .lock()
        .expect("Graphic name resolver mutex poisoned");
    let graphic_name = resolver.as_ref().and_then(|resolve| resolve(id))?;

    Some(match arg {
        Some(arg) if !arg.trim().is_empty() => format!("{}({})", graphic_name, arg),
        _ => graphic_name.to_string(),
    })
}

fn pattern_message(text: &str) -> (String, GraphicMsg) {
    if let Some(spec) = pattern_spec_from_text(text) {
        (
            "Changed Graphic!".to_string(),
            GraphicMsg::Pattern {
                name: spec.name,
                arg: spec.arg,
            },
        )
    } else {
        ("what?".to_string(), GraphicMsg::What)
    }
}

fn pattern_spec_from_text(text: &str) -> Option<GraphicPatternSpec> {
    let (name, arg) = split_name_and_arg(text);
    if name.is_empty() {
        None
    } else {
        Some(GraphicPatternSpec {
            name: name.to_string(),
            arg,
        })
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
