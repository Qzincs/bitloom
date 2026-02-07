use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::SidePanel::right("inspector")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.take_available_width();

            ui.horizontal(|ui| {
                ui.add_space(4.0); // left margin
                ui.strong("Inspector");
            });

            ui.separator();

            // TODO: protocol list
            ui.label("Field 1: Value");
        });
}
