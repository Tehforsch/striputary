use eframe::egui::{Color32, Key};

pub static PLOT_ASPECT: f32 = 100.0;
pub static CUT_LINE_COLOR: Color32 = Color32::GREEN;
pub static UNCUT_LINE_COLOR: Color32 = Color32::RED;

pub static CUT_LABEL_COLOR: Color32 = Color32::GREEN;
pub static UNCUT_LABEL_COLOR: Color32 = Color32::WHITE;

pub static SELECTED_FILL_COLOR: Color32 = Color32::GRAY;
pub static SELECTED_TEXT_COLOR: Color32 = Color32::BLACK;

pub static CUT_KEY: Key = Key::Enter;
pub static PLAYBACK_KEY: Key = Key::Space;
pub static SCROLL_DOWN_KEY: Key = Key::ArrowDown;
pub static SCROLL_UP_KEY: Key = Key::ArrowUp;

pub static NUM_PLOTS_TO_SHOW: usize = 16;
