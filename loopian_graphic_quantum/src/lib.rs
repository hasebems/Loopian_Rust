use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use loopian_graphic_api::generative_view::{GenerativeView, GraphMode, GraphicMsg};

pub mod view_spring;

use view_spring::Spring;

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

fn create_spring(ctx: &GraphicContext<'_>) -> Option<Box<dyn GenerativeView>> {
    Some(Box::new(Spring::new(ctx.font_nrm.clone())))
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
        find_factory(name).and_then(|factory| factory(&ctx))
    } else {
        None
    }
}
