use eframe::{egui::*};
use eframe::egui;

//#[derive(Default)]
pub struct LoopianApp {
    input_locate: u32,
    input_text: String,
}

impl LoopianApp {
    const SPACE_LEFT: f32 = 30.0;
    const SPACE_RIGHT: f32 = 870.0;
    const LEFT_MERGIN: f32 = 5.0;
    const LETTER_WIDTH: f32 = 10.0;

    const SPACE_UPPER: f32 = 420.0;
    const SPACE_LOWER: f32 = 450.0;
    const UPPER_MERGIN: f32 = 2.0;
    const LOWER_MERGIN: f32 = 3.0;
    const CURSOR_MERGIN: f32 = 6.0;
    const CURSOR_THICKNESS: f32 = 4.0;

    const PROMPT_LETTERS: usize = 3;

    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        fonts.font_data.insert(
            "profont".to_owned(),
            FontData::from_static(include_bytes!("../assets/newyork.ttf")),
        );
        fonts.font_data.insert(
            "monofont".to_owned(),
            FontData::from_static(include_bytes!("../assets/courier.ttc")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(FontFamily::Proportional)    //  search value of this key
            .or_default()                       //  if not found
            .insert(0, "profont".to_owned());

        // Put my font first (highest priority) for monospace text:
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "monofont".to_owned());

        // Tell egui to use these fonts:
        cc.egui_ctx.set_fonts(fonts);

        Self {
            input_locate: 0,
            input_text: String::new(),
        }
    }
    //  for update()
    fn update_eight_indicator(ui: &mut egui::Ui) {
        for i in 0..4 {
            for j in 0..2 {
                ui.painter().rect_filled(
                    Rect { min: Pos2 {x:Self::SPACE_LEFT + 220.0*(i as f32),
                                      y:50.0+50.0*(j as f32)}, 
                           max: Pos2 {x:210.0+220.0*(i as f32),
                                      y:80.0+50.0*(j as f32)},}, //  location
                    8.0,                //  curve
                    Color32::from_rgb(180, 180, 180),     //  color
                );
            }
        }
    }    
    fn update_title(ui: &mut egui::Ui) {
        ui.put(
            Rect { min: Pos2 {x:395.0, y:2.0}, max: Pos2 {x:505.0, y:47.0},}, //  location
            Label::new(RichText::new("Loopian")
                .size(32.0)
                .color(Color32::WHITE)
                .family(FontFamily::Proportional)
            )
        );
    }
    fn update_scroll_text(ui: &mut egui::Ui) {
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(30.0, 150.0), pos2(870.0, 400.0)),
            2.0,                              //  curve
            Color32::from_rgb(48, 48, 48)     //  color
        );            
        let past_text = String::from("Hello, World!");
        for i in 0..10 {
            let cnt = past_text.chars().count();
            ui.put(
                Rect { min: Pos2 {x:Self::SPACE_LEFT,  y:150.0+24.0*(i as f32)}, 
                       max: Pos2 {x:Self::SPACE_LEFT + 10.0*(cnt as f32),
                                  y:175.0+25.0*(i as f32)},},
                Label::new(RichText::new(&past_text)
                    .size(16.0)
                    .color(Color32::WHITE)
                    .family(FontFamily::Monospace)
                )
            );
        }
    }
    fn command_key(&self, key: &Key) {
        println!("Key>>{:?}",key);
    }
    fn input_letter(&mut self, letters: Vec<&String>) {
        println!("Letters:{:?}",letters);
        letters.iter().for_each(|ltr| {self.input_text.push_str(ltr);});
    }
    fn update_input_text(&self, ui: &mut egui::Ui) {
        let ltrcnt = self.input_text.chars().count() + Self::PROMPT_LETTERS;
        // Paint Letter Space
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(Self::SPACE_LEFT,Self::SPACE_UPPER),
                               pos2(Self::SPACE_RIGHT,Self::SPACE_LOWER)),
            2.0,                              //  curve
            Color32::from_rgb(48, 48, 48)     //  color
        );
        // Paint cursor
        ui.painter().rect_filled(
            Rect { min: Pos2 {x:Self::SPACE_LEFT + Self::LEFT_MERGIN
                                + Self::LETTER_WIDTH*(ltrcnt as f32)
                                + 3.25 - 0.25*(ltrcnt as f32), // 謎の調整
                            y:Self::SPACE_LOWER - Self::CURSOR_MERGIN},
                   max: Pos2 {x:Self::SPACE_LEFT + Self::LEFT_MERGIN - 1.0
                                + Self::LETTER_WIDTH*((ltrcnt+1) as f32)
                                + 3.25 - 0.25*(ltrcnt as f32), // 謎の調整
                            y:Self::SPACE_LOWER - Self::CURSOR_MERGIN + Self::CURSOR_THICKNESS},},
            0.0,                              //  curve
            Color32::from_rgb(160, 160, 160)  //  color
        );
        // Draw Letters
        ui.put( // Prompt
            Rect { min: Pos2 {x:Self::SPACE_LEFT + Self::LEFT_MERGIN,
                              y:Self::SPACE_UPPER + Self::UPPER_MERGIN},
                   max: Pos2 {x:Self::SPACE_LEFT + Self::LEFT_MERGIN 
                                + Self::LETTER_WIDTH*(Self::PROMPT_LETTERS as f32),
                              y:Self::SPACE_LOWER - Self::LOWER_MERGIN},},
            Label::new(RichText::new("R1>")
                .size(16.0).color(Color32::from_rgb(0,200,200)).family(FontFamily::Monospace))
        );
        ui.put( // User Input
            Rect { min: Pos2 {x:Self::SPACE_LEFT + Self::LEFT_MERGIN 
                                + Self::LETTER_WIDTH*(Self::PROMPT_LETTERS as f32)
                                + 3.25 - 0.25*(ltrcnt as f32), // 謎の調整
                              y:Self::SPACE_UPPER + Self::UPPER_MERGIN},
                   max: Pos2 {x:Self::SPACE_LEFT + Self::LEFT_MERGIN 
                                + Self::LETTER_WIDTH*(ltrcnt as f32)
                                + 3.25 - 0.25*(ltrcnt as f32), // 謎の調整
                              y:Self::SPACE_LOWER - Self::LOWER_MERGIN},},
            Label::new(RichText::new(&self.input_text)
                .size(16.0).color(Color32::WHITE).family(FontFamily::Monospace))
        );
    }
}

impl eframe::App for LoopianApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        //  Get Keyboard Event from Egui::Context
        let evts = ctx.input().events.clone();  
        let mut letters: Vec<&String> = vec![];
        for ev in evts.iter() {
            match ev {
                Event::Text(ltr) => letters.push(ltr),
                Event::Key {key,pressed, modifiers:_} => {
                    if pressed == &true {
                        if key == &Key::Enter {self.command_key(key);}
                        else if key == &Key::Backspace {self.command_key(key);}
                        else if key == &Key::ArrowDown {self.command_key(key);}
                        else if key == &Key::ArrowLeft {self.command_key(key);}
                        else if key == &Key::ArrowRight {self.command_key(key);}
                        else if key == &Key::ArrowUp {self.command_key(key);}
                    }
                },
                _ => {},
            }
        }
        if letters.len() >= 1 {self.input_letter(letters);}

        // Configuration for CentralPanel
        let my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow { extrusion: 0.0, color: Color32::BLACK },
            fill: Color32::BLACK,
            stroke: egui::Stroke::new(0.0, Color32::BLACK),
        };

        // Draw CentralPanel
        CentralPanel::default().frame(my_frame).show(ctx, |ui| {
            Self::update_eight_indicator(ui);
            Self::update_title(ui);

            ui.painter().text(
                Pos2 {x:60.0, y:65.0},
                Align2::CENTER_CENTER,
                "key:",
                FontId::new(16.0, FontFamily::Monospace),
                Color32::from_rgb(48, 48, 48)
            );

            //  scroll text
            Self::update_scroll_text(ui);

            //  input text
            self.update_input_text(ui);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some((900.0, 480.0).into()),
        resizable: false,
//        follow_system_theme: false,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native("Loopian", options, Box::new(|cc| Box::new(LoopianApp::new(cc))));
}