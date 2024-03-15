//#![windows_subsystem = "windows"]
use crate::app::BiliTicket;
use eframe::Theme;

mod app;
mod task;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = false;
    native_options.default_theme = Theme::Light;
    eframe::run_native(
        "Bili_Ticket",
        native_options,
        Box::new(|cc| Box::new(BiliTicket::new(cc))),
    )
    .unwrap();
}