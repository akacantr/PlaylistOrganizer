#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::Vec2;
use playlist_organizer::app::PlaylistOrganizerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1040.0, 740.0))
            .with_min_inner_size(Vec2::new(840.0, 580.0))
            .with_title("Spotify Playlist Genre Organizer"),
        ..Default::default()
    };

    eframe::run_native(
        "Spotify Playlist Genre Organizer",
        native_options,
        Box::new(|cc| Ok(Box::new(PlaylistOrganizerApp::new(cc)))),
    )
}
