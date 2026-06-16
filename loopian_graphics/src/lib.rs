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

type GraphicFactory = fn(&GraphicContext<'_>) -> Option<Box<dyn GenerativeView>>;

static GRAPHIC_REGISTRY: OnceLock<Mutex<HashMap<String, GraphicFactory>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, GraphicFactory>> {
    GRAPHIC_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn ensure_builtin_graphics() {
    let mut reg = registry().lock().expect("Graphic registry mutex poisoned");
    if !reg.is_empty() {
        return;
    }
    reg.insert("voice".to_string(), create_voice);
    reg.insert("lissa".to_string(), create_lissajous);
    reg.insert("beatlissa".to_string(), create_beatlissa);
    reg.insert("sinewave".to_string(), create_sinewave);
    reg.insert("rain".to_string(), create_raineffect);
    reg.insert("fish".to_string(), create_fish);
    reg.insert("jumping".to_string(), create_jumping);
    reg.insert("wavestick".to_string(), create_wavestick);
    reg.insert("circlethreads".to_string(), create_circlethreads);
    reg.insert("noteroll".to_string(), create_noteroll);
    reg.insert("spring".to_string(), create_spring);
}

pub fn register_graphic(name: impl Into<String>, factory: GraphicFactory) {
    ensure_builtin_graphics();
    let mut reg = registry().lock().expect("Graphic registry mutex poisoned");
    reg.insert(name.into(), factory);
}

fn find_factory(name: &str) -> Option<GraphicFactory> {
    ensure_builtin_graphics();
    let reg = registry().lock().expect("Graphic registry mutex poisoned");
    reg.get(name).copied()
}

pub fn builtin_graphic_names() -> Vec<String> {
    ensure_builtin_graphics();
    let reg = registry().lock().expect("Graphic registry mutex poisoned");
    let mut names = reg.keys().cloned().collect::<Vec<String>>();
    names.sort();
    names
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
