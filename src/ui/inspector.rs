use crate::{
    app::BitLoomApp,
    models::{
        field::{FieldLength, FieldRule},
        protocol::ProtocolLength,
    },
};
use eframe::egui;

pub fn show(app: &mut BitLoomApp, root_ui: &mut egui::Ui) {
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

            show_selection_details(app, ui);
        });
}

fn show_selection_details(app: &mut BitLoomApp, ui: &mut egui::Ui) {
    let state = &app.protocol_designer_state;

    if let Some(field_index) = state.selected_field_index {
        if let Some(field) = state.fields.get(field_index) {
            let field_origin = state
                .current_protocol_id
                .as_deref()
                .and_then(|protocol_id| field_origin(app, protocol_id, field_index));
            let field = field.clone();
            let inherited = field_index < state.inherited_field_count;
            let offset = field_offset(&state.fields, field_index);

            let action = show_field_details(
                ui,
                &field,
                field_index,
                offset,
                inherited,
                field_origin.as_ref(),
            );
            match action {
                Some(InspectorAction::JumpToProtocol(protocol_id)) => {
                    select_protocol(app, &protocol_id);
                }
                Some(InspectorAction::EditFieldName {
                    protocol_id,
                    field_id,
                    name,
                }) => {
                    if app
                        .protocol_registry
                        .edit_protocol_field(&protocol_id, &field_id, |field| {
                            field.name = name;
                            Ok(())
                        })
                        .is_ok()
                    {
                        refresh_current_protocol_fields(app, Some(field_index));
                    }
                }
                Some(InspectorAction::EditFieldId {
                    protocol_id,
                    old_field_id,
                    new_field_id,
                }) => {
                    if !new_field_id.is_empty()
                        && app
                            .protocol_registry
                            .update_protocol_field_id(&protocol_id, &old_field_id, &new_field_id)
                            .is_ok()
                    {
                        refresh_current_protocol_fields(app, Some(field_index));
                    }
                }
                None => {}
            }
            return;
        }
    }

    if let Some(protocol_id) = &state.current_protocol_id {
        if let Some(protocol) = app.protocol_registry.get_protocol(protocol_id) {
            ui.strong("Protocol");
            ui.add_space(6.0);

            detail_row(ui, "ID", &protocol.id);
            detail_row(ui, "Name", protocol.name.as_deref().unwrap_or("-"));
            detail_row(ui, "Endianness", &format!("{:?}", protocol.endianness));
            detail_row(ui, "Fields", &state.fields.len().to_string());
            detail_row(
                ui,
                "Total length",
                &format_protocol_length(app.protocol_registry.get_total_length(protocol_id)),
            );
            return;
        }
    }

    ui.label("Select a protocol or field to inspect it.");
}

fn show_field_details(
    ui: &mut egui::Ui,
    field: &FieldRule,
    field_index: usize,
    offset: u32,
    inherited: bool,
    field_origin: Option<&FieldOrigin>,
) -> Option<InspectorAction> {
    let mut action = None;

    ui.strong("Field");
    ui.add_space(4.0);

    if inherited {
        detail_row(ui, "ID", &field.id);
        detail_row(ui, "Name", field.name.as_deref().unwrap_or("-"));
    } else if let Some(origin) = field_origin {
        if let Some(new_field_id) = editable_text_row(ui, "ID", &field.id) {
            action = Some(InspectorAction::EditFieldId {
                protocol_id: origin.protocol_id.clone(),
                old_field_id: field.id.clone(),
                new_field_id,
            });
        }

        if let Some(name) = editable_optional_text_row(ui, "Name", field.name.as_deref()) {
            action = Some(InspectorAction::EditFieldName {
                protocol_id: origin.protocol_id.clone(),
                field_id: field.id.clone(),
                name,
            });
        }
    } else {
        detail_row(ui, "ID", &field.id);
        detail_row(ui, "Name", field.name.as_deref().unwrap_or("-"));
    }
    detail_row(ui, "Type", &field.field_type.to_string());
    detail_row(ui, "Length", &field.length.to_string());
    detail_row(ui, "Offset", &offset.to_string());
    detail_row(ui, "Index", &field_index.to_string());
    if inherited {
        if let Some(origin) = field_origin {
            if clickable_detail_row(ui, "Inherited from", &origin.display_name).clicked() {
                action = Some(InspectorAction::JumpToProtocol(origin.protocol_id.clone()));
            }
        } else {
            detail_row(ui, "Inherited from", "-");
        }
    } else {
        detail_row(
            ui,
            "Defined in",
            field_origin
                .map(|origin| origin.display_name.as_str())
                .unwrap_or("Current protocol"),
        );
    }

    action
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    property_row(ui, label, |ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(value);
            },
        );
    });
}

fn clickable_detail_row(ui: &mut egui::Ui, label: &str, value: &str) -> egui::Response {
    property_row(ui, label, |ui| ui.link(value))
}

fn editable_text_row(ui: &mut egui::Ui, label: &str, current_value: &str) -> Option<String> {
    let mut next_value = current_value.to_string();
    let response = property_row(ui, label, |ui| {
        ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y],
            egui::TextEdit::singleline(&mut next_value).horizontal_align(egui::Align::LEFT),
        )
    });

    if response.changed() {
        Some(next_value)
    } else {
        None
    }
}

fn editable_optional_text_row(
    ui: &mut egui::Ui,
    label: &str,
    current_value: Option<&str>,
) -> Option<Option<String>> {
    editable_text_row(ui, label, current_value.unwrap_or_default())
        .map(|value| if value.is_empty() { None } else { Some(value) })
}

fn property_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_value: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    const LABEL_WIDTH: f32 = 96.0;
    const VALUE_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.set_width(ui.available_width());
        ui.add_sized(
            [LABEL_WIDTH, ui.spacing().interact_size.y],
            egui::Label::new(label),
        );
        ui.add_space(VALUE_GAP);
        add_value(ui)
    })
    .inner
}

fn field_offset(fields: &[FieldRule], field_index: usize) -> u32 {
    fields
        .iter()
        .take(field_index)
        .map(|field| match field.length {
            FieldLength::Fixed(bits) => bits,
            FieldLength::Variable => 0,
        })
        .sum()
}

fn format_protocol_length(length: ProtocolLength) -> String {
    match length {
        ProtocolLength::Fixed(bits) => format!("{} bits", bits),
        ProtocolLength::Variable(prefix_bits) => format!("{}+ bits", prefix_bits),
    }
}

fn field_origin(app: &BitLoomApp, protocol_id: &str, field_index: usize) -> Option<FieldOrigin> {
    let mut first_index_in_protocol = 0;

    for protocol in app.protocol_registry.get_inheritance_chain(protocol_id) {
        let next_index = first_index_in_protocol + protocol.fields.len();
        if field_index < next_index {
            return Some(FieldOrigin {
                protocol_id: protocol.id.clone(),
                display_name: protocol_display_name(protocol.id.as_str(), protocol.name.as_deref()),
            });
        }
        first_index_in_protocol = next_index;
    }

    None
}

struct FieldOrigin {
    protocol_id: String,
    display_name: String,
}

enum InspectorAction {
    JumpToProtocol(String),
    EditFieldId {
        protocol_id: String,
        old_field_id: String,
        new_field_id: String,
    },
    EditFieldName {
        protocol_id: String,
        field_id: String,
        name: Option<String>,
    },
}

fn protocol_display_name(id: &str, name: Option<&str>) -> String {
    match name {
        Some(name) => format!("{} ({})", name, id),
        None => id.to_string(),
    }
}

fn select_protocol(app: &mut BitLoomApp, protocol_id: &str) {
    let (fields, inherited_field_count) = app
        .protocol_registry
        .resolve_fields(protocol_id)
        .unwrap_or_default();
    let state = &mut app.protocol_designer_state;

    state.current_protocol_id = Some(protocol_id.to_string());
    state
        .treeview_state
        .set_one_selected(protocol_id.to_string());
    state.fields = fields;
    state.inherited_field_count = inherited_field_count;
    state.selected_field_index = None;
}

fn refresh_current_protocol_fields(app: &mut BitLoomApp, selected_field_index: Option<usize>) {
    let Some(protocol_id) = app.protocol_designer_state.current_protocol_id.clone() else {
        return;
    };

    let (fields, inherited_field_count) = app
        .protocol_registry
        .resolve_fields(&protocol_id)
        .unwrap_or_default();
    let state = &mut app.protocol_designer_state;

    state.fields = fields;
    state.inherited_field_count = inherited_field_count;
    state.selected_field_index = selected_field_index;
}
