use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.take_available_width();

            ui.horizontal(|ui| {
                ui.add_space(4.0); // left margin
                ui.strong("Protocols");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0); // right margin
                    // new protocol button
                    if ui.small_button("+").clicked() {
                        todo!();
                    }
                });
            });

            ui.separator();

            // TODO: protocol list
            ui.label("Protocol 1");
        });
}
