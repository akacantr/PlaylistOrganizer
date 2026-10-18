use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};

pub const COLOR_BG: Color32 = Color32::from_rgb(18, 18, 18);
pub const COLOR_PANEL: Color32 = Color32::from_rgb(24, 24, 24);
pub const COLOR_CARD: Color32 = Color32::from_rgb(32, 32, 32);
pub const COLOR_CARD_HOVER: Color32 = Color32::from_rgb(44, 44, 44);
pub const COLOR_SPOTIFY_GREEN: Color32 = Color32::from_rgb(29, 185, 84);
#[allow(dead_code)]
pub const COLOR_SPOTIFY_GREEN_HOVER: Color32 = Color32::from_rgb(30, 215, 96);
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(167, 167, 167);
pub const COLOR_BORDER: Color32 = Color32::from_rgb(40, 40, 40);
pub const COLOR_DANGER: Color32 = Color32::from_rgb(225, 29, 72);

pub const GENRE_PALETTE: &[Color32] = &[
    Color32::from_rgb(29, 185, 84),   // Spotify Green
    Color32::from_rgb(59, 130, 246),  // Blue
    Color32::from_rgb(236, 72, 153),  // Pink
    Color32::from_rgb(139, 92, 246),  // Purple
    Color32::from_rgb(245, 158, 11),  // Amber
    Color32::from_rgb(16, 185, 129),  // Emerald
    Color32::from_rgb(6, 182, 212),   // Cyan
    Color32::from_rgb(249, 115, 22),  // Orange
    Color32::from_rgb(99, 102, 241),  // Indigo
    Color32::from_rgb(225, 29, 72),   // Rose
    Color32::from_rgb(20, 184, 166),  // Teal
    Color32::from_rgb(132, 204, 22),  // Lime
    Color32::from_rgb(168, 85, 247),  // Violet
    Color32::from_rgb(100, 116, 139), // Slate
];

pub fn get_genre_color(name: &str) -> Color32 {
    let mut hash: usize = 0;
    for b in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as usize);
    }
    GENRE_PALETTE[hash % GENRE_PALETTE.len()]
}

pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(COLOR_TEXT_PRIMARY);
    visuals.panel_fill = COLOR_PANEL;
    visuals.window_fill = COLOR_CARD;
    visuals.selection.bg_fill = COLOR_SPOTIFY_GREEN;

    // Smooth roundings
    visuals.window_rounding = Rounding::same(10.0);
    visuals.menu_rounding = Rounding::same(8.0);
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    visuals.widgets.inactive.rounding = Rounding::same(6.0);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);
    visuals.widgets.active.rounding = Rounding::same(6.0);
    visuals.widgets.open.rounding = Rounding::same(6.0);

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = COLOR_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);

    visuals.widgets.inactive.bg_fill = COLOR_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);

    visuals.widgets.hovered.bg_fill = COLOR_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_SPOTIFY_GREEN);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, COLOR_TEXT_PRIMARY);

    visuals.widgets.active.bg_fill = COLOR_SPOTIFY_GREEN;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, COLOR_SPOTIFY_GREEN);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::BLACK);

    ctx.set_visuals(visuals);
}

pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_string())
            .color(Color32::BLACK)
            .strong(),
    )
    .fill(COLOR_SPOTIFY_GREEN)
    .rounding(Rounding::same(20.0))
    .min_size(Vec2::new(80.0, 32.0))
}

pub fn secondary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_string())
            .color(COLOR_TEXT_PRIMARY)
            .strong(),
    )
    .fill(COLOR_CARD)
    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
    .rounding(Rounding::same(20.0))
    .min_size(Vec2::new(80.0, 32.0))
}

pub fn danger_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_string())
            .color(Color32::WHITE)
            .strong(),
    )
    .fill(COLOR_DANGER)
    .rounding(Rounding::same(20.0))
    .min_size(Vec2::new(80.0, 32.0))
}

pub fn genre_badge(ui: &mut egui::Ui, name: &str, selected: bool) -> egui::Response {
    let color = get_genre_color(name);
    let (bg_color, text_color, stroke) = if selected {
        (color, Color32::BLACK, Stroke::new(1.5_f32, Color32::WHITE))
    } else {
        (
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 40),
            color,
            Stroke::new(1.0_f32, color),
        )
    };

    let button = egui::Button::new(
        egui::RichText::new(name)
            .color(text_color)
            .size(12.0)
            .strong(),
    )
    .fill(bg_color)
    .stroke(stroke)
    .rounding(Rounding::same(12.0));

    ui.add(button)
}
