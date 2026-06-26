use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, root_ui: &mut egui::Ui) {
    egui::Panel::bottom("hex_view")
        .resizable(true)
        .default_size(200.0)
        .show_inside(root_ui, |ui| {
            ui.take_available_height();

            ui.label("Hex View");
        });
}
