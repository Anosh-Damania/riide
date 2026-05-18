#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod editor;
mod io;
mod state;
mod ui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Riide IDE",
        options,
        Box::new(|_cc| Box::new(app::RiideApp::default())),
    )
    .unwrap();
}