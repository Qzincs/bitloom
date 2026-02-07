use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("hex_view")
        .resizable(true)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.take_available_height();

            ui.label("Hex View");
        });
}
