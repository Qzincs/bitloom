use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, root_ui: &mut egui::Ui) {
    egui::Panel::right("inspector")
        .resizable(true)
        .default_size(200.0)
        .size_range(150.0..=400.0)
        .show_inside(root_ui, |ui| {
            ui.take_available_width();

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.add_space(4.0); // left margin
                ui.strong("Inspector");
            });

            ui.separator();

            // TODO: protocol list
            ui.label("Field 1: Value");
        });
}
