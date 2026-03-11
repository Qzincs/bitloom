use crate::{
    app::BitLoomApp,
    models::field::{FieldLength, FieldRule, FieldType},
};
use eframe::egui::{self, Button};
use egui_extras::{Column, TableBuilder};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EditableColumn {
    Id,
    Name,
    Length,
}

pub fn show(app: &mut BitLoomApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.strong("Fields");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                // new field button
                let no_proto_selected = app.protocol_designer_state.current_protocol_id.is_none();
                let btn = ui.add_enabled(!no_proto_selected, egui::Button::new("+").small());
                if !no_proto_selected && btn.clicked() {
                    let state = &mut app.protocol_designer_state;
                    if let Some(current_proto_id) = &state.current_protocol_id {
                        let mut new_field = FieldRule::default();
                        new_field.id = format!("field_{}", state.fields.len());

                        match app
                            .protocol_registry
                            .add_protocol_field(current_proto_id, new_field.clone())
                        {
                            Ok(_) => {
                                state.fields.push(new_field);
                                // Auto-focus the ID cell of the new row
                                let new_row_idx = state.fields.len() - 1;
                                state.editing_cell = Some((new_row_idx, EditableColumn::Id));
                                state.focus_new_row = true;
                            }
                            Err(err) => {
                                // TODO: show error to user
                            }
                        }
                    }
                }
            });
        });

        ui.separator();

        show_fields_table(app, ui);
    });
}

fn show_fields_table(app: &mut BitLoomApp, ui: &mut egui::Ui) {
    let BitLoomApp {
        protocol_designer_state: state,
        protocol_registry: registry,
        ..
    } = app;

    let Some(current_proto) = state.current_protocol_id.clone() else {
        ui.centered_and_justified(|ui| {
            ui.label("Please select a protocol from the sidebar to view its fields.");
        });
        return;
    };

    let fields_len = state.fields.len();
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(60.0).clip(true)) // ID
        .column(Column::auto().at_least(60.0).clip(true)) // Name
        .column(Column::initial(110.0).at_least(80.0).clip(true)) // Type
        .column(Column::initial(70.0).at_least(50.0).clip(true)) // Length
        .column(Column::remainder().at_least(50.0).clip(true)) // Offset
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("ID");
            });
            header.col(|ui| {
                ui.strong("Name");
            });
            header.col(|ui| {
                ui.strong("Type");
            });
            header.col(|ui| {
                ui.strong("Length");
            });
            header.col(|ui| {
                ui.strong("Offset");
            });
        })
        .body(|body| {
            let mut offset = 0;
            body.rows(text_height + 8.0, fields_len, |mut row| {
                let row_idx = row.index();
                row.set_selected(state.selected_field_index == Some(row_idx));
                let field = &mut state.fields[row_idx];

                // Render inherited fields as disabled and non-editable
                if row_idx < state.inherited_field_count {
                    let render_inherited_field = |ui: &mut egui::Ui, text: String| {
                        ui.scope(|ui| {
                            ui.disable();
                            ui.add_sized(
                                [ui.available_width(), ui.spacing().interact_size.y],
                                egui::Button::selectable(false, text),
                            );
                        });
                    };
                    // ID
                    row.col(|ui| {
                        render_inherited_field(ui, field.id.clone());
                    });
                    // Name
                    row.col(|ui| {
                        let name = field.name.as_deref().unwrap_or("-");
                        render_inherited_field(ui, name.to_string());
                    });
                    // Type
                    row.col(|ui| {
                        let type_text = field.field_type.to_string();
                        ui.scope(|ui| {
                            ui.disable();
                            egui::ComboBox::from_id_salt(format!("type_inh_{}", row_idx))
                                .selected_text(type_text)
                                .width(ui.available_width())
                                .show_ui(ui, |_| {});
                        });
                    });
                    // Length
                    row.col(|ui| {
                        render_inherited_field(ui, field.length.to_string());
                    });
                    // Offset
                    row.col(|ui| {
                        ui.add_space(4.0);
                        ui.label(offset.to_string());
                    });
                } else {
                    let field_id = field.id.clone(); // 用于 Registry 定位的原始 ID
                    let current_proto_id = state.current_protocol_id.as_ref().unwrap();

                    // ID
                    row.col(|ui| {
                        let is_editing = state.editing_cell == Some((row_idx, EditableColumn::Id));
                        if is_editing {
                            let res = ui.text_edit_singleline(&mut field.id);
                            if res.changed() {
                                let new_field_id = field.id.clone();
                                let _ = app.protocol_registry.edit_protocol_field(
                                    current_proto_id,
                                    &field_id,
                                    |f| {
                                        f.id = new_field_id;
                                        Ok(())
                                    },
                                );
                            }
                            if res.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                state.editing_cell = None;
                            }
                            if state.focus_new_row && row_idx == fields_len - 1 {
                                res.request_focus();
                                state.focus_new_row = false;
                            }
                        } else {
                            // normal label with double-click to edit
                            let resp = ui
                                .add_sized(
                                    [ui.available_width(), ui.spacing().interact_size.y],
                                    egui::Button::selectable(false, &field.id),
                                )
                                .on_hover_text("Double-click to edit");
                            if resp.double_clicked() {
                                state.editing_cell = Some((row_idx, EditableColumn::Id));
                            }
                        }
                    });

                    // Name
                    row.col(|ui| {
                        let is_editing =
                            state.editing_cell == Some((row_idx, EditableColumn::Name));
                        if is_editing {
                            let mut name_text = field.name.clone().unwrap_or_default();
                            let res = ui.text_edit_singleline(&mut name_text);
                            if res.changed() {
                                field.name = if name_text.is_empty() {
                                    None
                                } else {
                                    Some(name_text.clone())
                                };
                                let _ = app.protocol_registry.edit_protocol_field(
                                    current_proto_id,
                                    &field_id,
                                    |f| {
                                        f.name = field.name.clone();
                                        Ok(())
                                    },
                                );
                            }
                            if res.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                state.editing_cell = None;
                            }
                        } else {
                            let name_text = field.name.as_deref().unwrap_or("");
                            let resp = ui
                                .add_sized(
                                    [ui.available_width(), ui.spacing().interact_size.y],
                                    egui::Button::selectable(false, name_text),
                                )
                                .on_hover_text("Double-click to edit");
                            if resp.double_clicked() {
                                state.editing_cell = Some((row_idx, EditableColumn::Name));
                            }
                        }
                    });

                    // Type (dropdown)
                    let current_type_name = field.field_type.to_string();
                    row.col(|ui| {
                        let res = egui::ComboBox::from_id_salt(format!("type_{}", row_idx))
                            .selected_text(current_type_name.clone())
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(current_type_name == "Fixed", "Fixed")
                                    .clicked()
                                {
                                    field.field_type = FieldType::Fixed(0);
                                }
                                if ui
                                    .selectable_label(current_type_name == "Enum", "Enum")
                                    .clicked()
                                {
                                    field.field_type = FieldType::Enum(vec![]);
                                }
                                if ui
                                    .selectable_label(current_type_name == "Range", "Range")
                                    .clicked()
                                {
                                    field.field_type = FieldType::Range {
                                        min: 0,
                                        max: 255,
                                        is_signed: false,
                                    };
                                }
                                if ui
                                    .selectable_label(
                                        current_type_name == "Expression",
                                        "Expression",
                                    )
                                    .clicked()
                                {
                                    field.field_type = FieldType::Expr(String::new());
                                }
                                if ui
                                    .selectable_label(current_type_name == "Input", "Input")
                                    .clicked()
                                {
                                    field.field_type = FieldType::Input;
                                }
                            })
                            .response;

                        if res.changed() {
                            let new_type = field.field_type.clone();
                            let _ = app.protocol_registry.edit_protocol_field(
                                current_proto_id,
                                &field_id,
                                |f| {
                                    f.field_type = new_type;
                                    Ok(())
                                },
                            );
                        }
                    });

                    // Length
                    row.col(|ui| {
                        // TODO: variable length
                        if let FieldLength::Fixed(mut len) = field.length {
                            let w = ui.available_width();
                            if ui
                                .add_sized(
                                    [w, ui.spacing().interact_size.y],
                                    egui::DragValue::new(&mut len).speed(1.0),
                                )
                                .changed()
                            {
                                field.length = FieldLength::Fixed(len);
                                let _ = app.protocol_registry.edit_protocol_field(
                                    current_proto_id,
                                    &field_id,
                                    |f| {
                                        f.length = FieldLength::Fixed(len);
                                        Ok(())
                                    },
                                );
                            }
                        } else {
                            ui.label("Variable");
                        }
                    });

                    // Offset
                    row.col(|ui| {
                        ui.add_space(4.0);
                        ui.label(offset.to_string());
                    });
                }

                offset += match field.length {
                    FieldLength::Fixed(len) => len,
                    FieldLength::Variable => 0,
                };

                if row.response().clicked() {
                    state.selected_field_index = Some(row_idx);
                }
            })
        });

    // Click on empty space below rows to deselect
    let remaining = ui.available_rect_before_wrap();
    if remaining.height() > 0.0 {
        let resp = ui.interact(
            remaining,
            ui.id().with("deselect_area"),
            egui::Sense::click(),
        );
        if resp.clicked() {
            state.selected_field_index = None;
        }
    }
}
