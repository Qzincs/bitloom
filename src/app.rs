use crate::{
    models::{
        field::FieldRule,
        protocol::{Endianness, ProtocolRegistry, ProtocolTreeNode},
    },
    ui::protocol_designer::EditableColumn,
};
use eframe::egui;

#[derive(PartialEq)]
pub enum ViewPage {
    ProtocolDesigner,
    PacketBuilder,
}

pub struct BitLoomApp {
    pub current_page: ViewPage,
    pub protocol_designer_state: ProtocolDesignerState,

    pub protocol_registry: ProtocolRegistry,
}

pub struct ProtocolDesignerState {
    pub protocol_trees: Vec<ProtocolTreeNode>,
    pub current_protocol_id: Option<String>,
    pub treeview_state: egui_ltreeview::TreeViewState<String>,

    // Modal state for adding a new protocol
    pub is_adding_protocol: bool,
    pub new_protocol_id: String,
    pub new_protocol_name: String,
    pub new_protocol_endianness: Endianness,
    pub new_protocol_parent_id: Option<String>,
    pub error_msg: Option<String>,

    // Modal state for confirming deletion
    pub is_confirming_delete: bool,
    pub delete_target_id: Option<String>,

    pub fields: Vec<FieldRule>, // The fields copy for the currently selected protocol
    pub inherited_field_count: usize, // Number of inherited fields (not directly defined in the current protocol)
    pub focus_new_row: bool,          // Whether to auto-focus the new row after adding
    pub editing_cell: Option<(usize, EditableColumn)>, // (field index, column)
    pub selected_field_index: Option<usize>, // Currently selected row in the fields table
}

impl Default for ProtocolDesignerState {
    fn default() -> Self {
        Self {
            protocol_trees: Vec::new(),
            current_protocol_id: None,
            treeview_state: egui_ltreeview::TreeViewState::default(),
            is_adding_protocol: false,
            new_protocol_id: String::new(),
            new_protocol_name: String::new(),
            new_protocol_endianness: Endianness::Big,
            new_protocol_parent_id: None,
            error_msg: None,
            is_confirming_delete: false,
            delete_target_id: None,
            fields: Vec::new(),
            inherited_field_count: 0,
            focus_new_row: false,
            editing_cell: None,
            selected_field_index: None,
        }
    }
}

impl BitLoomApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            current_page: ViewPage::ProtocolDesigner,
            protocol_registry: ProtocolRegistry::new(),
            protocol_designer_state: ProtocolDesignerState::default(),
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
