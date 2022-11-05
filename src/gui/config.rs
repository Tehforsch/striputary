use eframe::egui::Color32;
use eframe::egui::Key;

pub static PLOT_HEIGHT: f32 = 50.0;
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

pub static CUT_BUTTON_SIZE_X: f32 = 200.0;
pub static CUT_BUTTON_SIZE_Y: f32 = 50.0;

pub static MIN_SIDE_BAR_WIDTH: f32 = 200.0;

pub static MIN_NUM_PLOTS_SHOWN: i32 = 5;

pub static CUT_MARKER_WIDTH: f32 = 2.0;
pub static CUT_MARKER_COLOR: Color32 = Color32::YELLOW;
