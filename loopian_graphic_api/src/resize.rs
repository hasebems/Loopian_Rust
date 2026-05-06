use nannou::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct Resize {
    pub full_size_x: f32,
    pub full_size_y: f32,
    pub eight_indic_left: f32,
    pub scroll_txt_left: f32,
    pub input_txt_top: f32,
    pub input_txt_left: f32,
}

impl Resize {
    pub fn new(app: &App) -> Resize {
        const INPUT_TXT_LOWER_MARGIN: f32 = 100.0;
        const MIN_LEFT_MARGIN: f32 = 0.0;
        const MIN_RIGHT_MARGIN: f32 = 30.0;

        let win = app.main_window();
        let win_rect = win.rect();
        let win_width = win_rect.w();
        let win_height = win_rect.h();
        let st_left_margin = -win_width / 6.0 + MIN_LEFT_MARGIN;

        Resize {
            full_size_x: win_width,
            full_size_y: win_height,
            eight_indic_left: win_width / 2.0 - MIN_RIGHT_MARGIN,
            scroll_txt_left: st_left_margin,
            input_txt_top: -win_height / 2.0 + INPUT_TXT_LOWER_MARGIN,
            input_txt_left: 0.0,
        }
    }

    pub fn get_full_size_x(&self) -> f32 {
        self.full_size_x
    }

    pub fn get_full_size_y(&self) -> f32 {
        self.full_size_y
    }

    pub fn get_eight_indic_left(&self) -> f32 {
        self.eight_indic_left
    }

    pub fn get_scroll_txt_left(&self) -> f32 {
        self.scroll_txt_left
    }

    pub fn get_input_txt_top(&self) -> f32 {
        self.input_txt_top
    }

    pub fn get_input_txt_left(&self) -> f32 {
        self.input_txt_left
    }
}
