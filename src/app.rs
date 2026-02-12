use eframe::egui;

#[derive(PartialEq)]
pub enum ViewPage {
    ProtocolDesigner,
    PacketBuilder,
}

pub struct BitLoomApp {
    pub current_page: ViewPage,
}

impl BitLoomApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            current_page: ViewPage::ProtocolDesigner,
        }
    }
}

impl eframe::App for BitLoomApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::top_panel::show(self, ctx);
        crate::ui::sidebar::show(self, ctx);
        crate::ui::hex_view::show(self, ctx);
        crate::ui::inspector::show(self, ctx);
        crate::ui::protocol_designer::show(self, ctx);
    }
}
