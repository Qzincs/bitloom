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

fn show_selection_details(app: &BitLoomApp, ui: &mut egui::Ui) {
    let state = &app.protocol_designer_state;

    if let Some(field_index) = state.selected_field_index {
        if let Some(field) = state.fields.get(field_index) {
            let field_origin = state
                .current_protocol_id
                .as_deref()
                .and_then(|protocol_id| field_origin(app, protocol_id, field_index));
            show_field_details(
                ui,
                field,
                field_index,
                field_offset(&state.fields, field_index),
                field_index < state.inherited_field_count,
                field_origin.as_deref(),
            );
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
    field_origin: Option<&str>,
) {
    ui.strong("Field");
    ui.add_space(6.0);

    detail_row(ui, "ID", &field.id);
    detail_row(ui, "Name", field.name.as_deref().unwrap_or("-"));
    detail_row(ui, "Type", &field.field_type.to_string());
    detail_row(ui, "Length", &field.length.to_string());
    detail_row(ui, "Offset", &offset.to_string());
    detail_row(ui, "Index", &field_index.to_string());
    if inherited {
        detail_row(ui, "Inherited from", field_origin.unwrap_or("-"));
    } else {
        detail_row(ui, "Defined in", field_origin.unwrap_or("Current protocol"));
    }
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
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

fn field_origin(app: &BitLoomApp, protocol_id: &str, field_index: usize) -> Option<String> {
    let mut first_index_in_protocol = 0;

    for protocol in app.protocol_registry.get_inheritance_chain(protocol_id) {
        let next_index = first_index_in_protocol + protocol.fields.len();
        if field_index < next_index {
            return Some(protocol_display_name(
                protocol.id.as_str(),
                protocol.name.as_deref(),
            ));
        }
        first_index_in_protocol = next_index;
    }

    None
}

fn protocol_display_name(id: &str, name: Option<&str>) -> String {
    match name {
        Some(name) => format!("{} ({})", name, id),
        None => id.to_string(),
    }
}
