use crate::app::{BitLoomApp, ViewPage};
use eframe::egui;

pub fn show(app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {}
                if ui.button("Open").clicked() {}
            });
            ui.menu_button("Help", |ui| if ui.button("About").clicked() {});
        });
    });

    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut app.current_page,
                ViewPage::ProtocolDesigner,
                "Protocol Designer",
            );
            ui.selectable_value(
                &mut app.current_page,
                ViewPage::PacketBuilder,
                "Packet Builder",
            );
        });
    });
}
