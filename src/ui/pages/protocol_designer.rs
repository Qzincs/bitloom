use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| ui.label("Filed table here"));
}
