use crate::app::BitLoomApp;
use eframe::egui;

pub fn show(_app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label(
            _app.protocol_designer_state
                .current_protocol_id
                .as_deref()
                .unwrap_or("No protocol selected"),
        );
    });
}
