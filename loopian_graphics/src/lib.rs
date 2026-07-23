use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use loopian_graphic_api::generative_view::{GenerativeView, GraphMode, GraphicMsg};

pub mod view_beatlissa;
pub mod view_circlethds;
pub mod view_fish;
pub mod view_jumping;
pub mod view_lissajous;
pub mod view_noteroll;
pub mod view_raineffect;
pub mod view_sinewave;
pub mod view_spring;
pub mod view_voice4;
pub mod view_wavestick;

use view_beatlissa::BeatLissa;
use view_circlethds::CircleThread;
use view_fish::SchoolOfFish;
use view_jumping::Jumping;
use view_lissajous::Lissajous;
use view_noteroll::NoteRoll;
use view_raineffect::RainEffect;
use view_sinewave::SineWave;
use view_spring::Spring;
use view_voice4::Voice4;
use view_wavestick::WaveStick;

pub struct GraphicContext<'a> {
    pub crnt_time: f32,
    pub gmode: GraphMode,
    pub meter_text: &'a str,
    pub font_nrm: nannou::text::Font,
    pub arg: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub struct BuiltinGraphic {
    pub id: usize,
    pub name: &'static str,
    pub list_note: &'static str,
    factory: GraphicFactory,
}

impl BuiltinGraphic {
    const fn new(
        id: usize,
        name: &'static str,
        list_note: &'static str,
        factory: GraphicFactory,
    ) -> Self {
        Self {
            id,
            name,
            list_note,
            factory,
        }
    }

    fn factory(self) -> GraphicFactory {
        self.factory
    }
}

type GraphicFactory = fn(&GraphicContext<'_>) -> Option<Box<dyn GenerativeView>>;

const BUILTIN_GRAPHICS: &[BuiltinGraphic] = &[
    BuiltinGraphic::new(1, "beatlissa", "(0/1)", create_beatlissa),
    BuiltinGraphic::new(2, "circlethreads", "", create_circlethreads),
    BuiltinGraphic::new(3, "fish", "", create_fish),
    BuiltinGraphic::new(4, "jumping", "", create_jumping),
    BuiltinGraphic::new(5, "lissa", "", create_lissajous),
    BuiltinGraphic::new(6, "noteroll", "(v/h)", create_noteroll),
    BuiltinGraphic::new(7, "rain", "", create_raineffect),
    BuiltinGraphic::new(8, "sinewave", "", create_sinewave),
    BuiltinGraphic::new(9, "spring", "", create_spring),
    BuiltinGraphic::new(10, "voice", "", create_voice),
    BuiltinGraphic::new(11, "wavestick", "", create_wavestick),
];

static GRAPHIC_REGISTRY: OnceLock<Mutex<HashMap<String, GraphicFactory>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, GraphicFactory>> {
    GRAPHIC_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn install_graphic_name_resolver() {
    loopian_graphic_api::generative_view::register_graphic_name_resolver(builtin_graphic_name);
}

pub fn builtin_graphics() -> &'static [BuiltinGraphic] {
    BUILTIN_GRAPHICS
}

pub fn builtin_graphic_list_text() -> String {
    BUILTIN_GRAPHICS
        .iter()
        .map(|graphic| format!("{}: {}{}", graphic.id, graphic.name, graphic.list_note))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn builtin_graphic_names() -> Vec<String> {
    BUILTIN_GRAPHICS
        .iter()
        .map(|graphic| graphic.name.to_string())
        .collect()
}

pub fn builtin_graphic_name(id: usize) -> Option<&'static str> {
    BUILTIN_GRAPHICS
        .iter()
        .find(|graphic| graphic.id == id)
        .map(|graphic| graphic.name)
}

pub fn builtin_graphic_id(name: &str) -> Option<usize> {
    BUILTIN_GRAPHICS
        .iter()
        .find(|graphic| graphic.name == name)
        .map(|graphic| graphic.id)
}

fn builtin_factory_by_id(id: usize) -> Option<GraphicFactory> {
    BUILTIN_GRAPHICS
        .iter()
        .find(|graphic| graphic.id == id)
        .map(|graphic| graphic.factory())
}

fn builtin_factory_by_name(name: &str) -> Option<GraphicFactory> {
    BUILTIN_GRAPHICS
        .iter()
        .find(|graphic| graphic.name == name)
        .map(|graphic| graphic.factory())
}

pub fn register_graphic(name: impl Into<String>, factory: GraphicFactory) {
    let mut reg = registry().lock().expect("Graphic registry mutex poisoned");
    reg.insert(name.into(), factory);
}

fn find_factory(name: &str) -> Option<GraphicFactory> {
    if let Ok(id) = name.parse::<usize>()
        && let Some(factory) = builtin_factory_by_id(id)
    {
        return Some(factory);
    }

    if let Some(factory) = builtin_factory_by_name(name) {
        return Some(factory);
    }

    let reg = registry().lock().expect("Graphic registry mutex poisoned");
    reg.get(name).copied()
}

fn create_voice(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(Voice4::new(ctx.font_nrm.clone())))
}

fn create_lissajous(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(Lissajous::new(ctx.gmode)))
}

fn create_beatlissa(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    let md = ctx.arg.and_then(|x| x.parse::<i32>().ok()).unwrap_or(0);
    let num = ctx
        .meter_text
        .split('/')
        .next()
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or(0);
    Some(Box::new(BeatLissa::new(num, ctx.crnt_time, md, ctx.gmode)))
}

fn create_sinewave(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(SineWave::new(ctx.gmode)))
}

fn create_raineffect(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(RainEffect::new(ctx.gmode)))
}

fn create_fish(_ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(SchoolOfFish::new()))
}

fn create_jumping(_ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(Jumping::new()))
}

fn create_wavestick(_ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(WaveStick::new()))
}

fn create_circlethreads(_ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(CircleThread::new()))
}

fn create_spring(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(Spring::new(ctx.font_nrm.clone())))
}

fn create_noteroll(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    let roll_mode = ctx.arg.unwrap_or("v");
    Some(Box::new(NoteRoll::new(roll_mode, ctx.gmode)))
}

pub fn get_view_instance(
    crnt_time: f32,
    gmsg: &GraphicMsg,
    gmode: GraphMode,
    meter_text: &str,
    font_nrm: nannou::text::Font,
) -> Option<Box<dyn GenerativeView>> {
    if let GraphicMsg::Pattern { name, arg } = gmsg {
        let ctx = GraphicContext {
            crnt_time,
            gmode,
            meter_text,
            font_nrm,
            arg: arg.as_deref(),
        };
        if let Some(factory) = find_factory(name) {
            factory(&ctx)
        } else {
            None
        }
    } else {
        None
    }
}
