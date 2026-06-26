use crate::app::BitLoomApp;
use crate::models::protocol::{Endianness, ProtocolTreeNode};
use eframe::egui;
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder};

const ADD_PROTOCOL_ID_INPUT: &str = "add_protocol_id_input";
const ADD_PROTOCOL_NAME_INPUT: &str = "add_protocol_name_input";

enum ContextMenuActions {
    AddProtocol(String),    // parent_id
    DeleteProtocol(String), // protocol_id
}

pub fn show(app: &mut BitLoomApp, root_ui: &mut egui::Ui) {
    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(200.0)
        .show_inside(root_ui, |ui| {
            ui.take_available_width();

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.add_space(4.0); // left margin
                ui.strong("Protocols");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0); // right margin
                    // new protocol button
                    if ui.small_button("+").clicked() {
                        // if node is selected, set the new protocol's parent to the selected node,
                        // otherwise it's a top-level protocol
                        app.protocol_designer_state.new_protocol_parent_id =
                            app.protocol_designer_state.current_protocol_id.clone();
                        app.protocol_designer_state.is_adding_protocol = true;
                        app.protocol_designer_state.focus_new_protocol_id = true;
                    }
                });
            });

            ui.separator();

            show_protocols_trees(app, ui);

            show_add_protocol_modal(
                ui.ctx().clone(),
                app,
                app.protocol_designer_state.new_protocol_parent_id.clone(),
            );

            show_confirm_delete_modal(
                ui.ctx().clone(),
                app,
                app.protocol_designer_state
                    .delete_target_id
                    .clone()
                    .unwrap_or_default(),
            );
        });
}

/// Show the modal dialog for adding a new protocol. Opens when `state.is_adding_protocol` is true.
fn show_add_protocol_modal(ctx: egui::Context, app: &mut BitLoomApp, parent_id: Option<String>) {
    let state = &mut app.protocol_designer_state;

    if state.is_adding_protocol {
        egui::Modal::new(egui::Id::new("add_protocol_modal")).show(&ctx, |ui| {
            ui.set_width(280.0);

            ui.heading("Add New Protocol");
            ui.add_space(8.0);

            // Form fields
            egui::Grid::new("add_protocol_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Protocol ID:");
                    let id_res = ui.add(
                        egui::TextEdit::singleline(&mut state.new_protocol_id)
                            .id(egui::Id::new(ADD_PROTOCOL_ID_INPUT)),
                    );
                    if state.focus_new_protocol_id {
                        id_res.request_focus();
                        state.focus_new_protocol_id = false;
                    }
                    ui.end_row();

                    ui.label("Name (Optional):");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.new_protocol_name)
                            .id(egui::Id::new(ADD_PROTOCOL_NAME_INPUT)),
                    );
                    ui.end_row();

                    ui.label("Endianness:");
                    egui::ComboBox::from_id_salt("endian_combo")
                        .selected_text(format!("{:?}", state.new_protocol_endianness))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut state.new_protocol_endianness,
                                Endianness::Big,
                                "Big",
                            );
                            ui.selectable_value(
                                &mut state.new_protocol_endianness,
                                Endianness::Little,
                                "Little",
                            );
                        });
                    ui.end_row();
                });

            // error message
            ui.add_space(8.0);
            if let Some(ref msg) = state.error_msg {
                ui.colored_label(egui::Color32::RED, msg);
            }

            ui.separator();
            ui.add_space(4.0);

            // action buttons
            let button_size = egui::Vec2::new(60.0, ui.spacing().interact_size.y);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_sized(button_size, egui::Button::new("Cancel"))
                    .clicked()
                {
                    clear_add_protocol_focus(&ctx);
                    state.new_protocol_id.clear();
                    state.new_protocol_name.clear();
                    state.is_adding_protocol = false;
                    ui.close();
                }

                ui.add_space(4.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Add"))
                    .clicked()
                {
                    let id = state.new_protocol_id.trim();
                    let name = state.new_protocol_name.trim();
                    if id.is_empty() {
                        state.error_msg = Some("Protocol ID cannot be empty.".to_string());
                        return;
                    } else {
                        let name = if name.is_empty() {
                            None
                        } else {
                            Some(name.to_string())
                        };

                        match app.protocol_registry.create_protocol(
                            id,
                            name,
                            state.new_protocol_endianness,
                            parent_id.clone(),
                        ) {
                            Ok(_) => {
                                clear_add_protocol_focus(&ctx);
                                state.new_protocol_id.clear();
                                state.new_protocol_name.clear();
                                state.is_adding_protocol = false;
                                state.error_msg = None;
                                state.protocol_trees = app.protocol_registry.build_protocol_trees();
                                ui.close();
                            }
                            Err(e) => {
                                state.error_msg = Some(format!("Error: {}", e));
                            }
                        }
                    }
                }
            });
        });
    }
}

fn clear_add_protocol_focus(ctx: &egui::Context) {
    ctx.memory_mut(|memory| {
        memory.surrender_focus(egui::Id::new(ADD_PROTOCOL_ID_INPUT));
        memory.surrender_focus(egui::Id::new(ADD_PROTOCOL_NAME_INPUT));
    });
}

/// Show the modal dialog for confirming protocol deletion. Opens when `state.is_confirming_delete` is true.
fn show_confirm_delete_modal(ctx: egui::Context, app: &mut BitLoomApp, protocol_id: String) {
    let state = &mut app.protocol_designer_state;

    if state.is_confirming_delete {
        egui::Modal::new(egui::Id::new("delete_confirm_modal")).show(&ctx, |ui| {
            ui.set_width(280.0);

            ui.heading("Confirm Delete");
            ui.add_space(8.0);

            ui.label(format!(
                "Are you sure you want to delete protocol '{}' and all its subprotocols?",
                protocol_id
            ));
            ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
            ui.add_space(12.0);

            let button_size = egui::vec2(80.0, ui.spacing().interact_size.y);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_sized(button_size, egui::Button::new("Cancel"))
                    .clicked()
                {
                    state.delete_target_id = None;
                    state.is_confirming_delete = false;
                    ui.close();
                }

                ui.add_space(8.0);

                let delete_btn =
                    egui::Button::new("Delete").fill(egui::Color32::from_rgb(180, 0, 0));
                if ui.add_sized(button_size, delete_btn).clicked() {
                    if let Some(id) = state.delete_target_id.take() {
                        let _ = app.protocol_registry.remove_protocol(&id);
                        state.protocol_trees = app.protocol_registry.build_protocol_trees();
                    }
                    state.is_confirming_delete = false;
                    ui.close();
                }
            });
        });
    }
}

/// Show all protocols in the sidebar tree.
fn show_protocols_trees(app: &mut BitLoomApp, ui: &mut egui::Ui) {
    let protocol_trees = &app.protocol_designer_state.protocol_trees;
    let treeview_state = &mut app.protocol_designer_state.treeview_state;
    let current_id = &mut app.protocol_designer_state.current_protocol_id;
    let mut sidebar_actions = Vec::new();

    let (response, actions) = TreeView::new(ui.make_persistent_id("protocol_tree")).show_state(
        ui,
        treeview_state,
        |builder| {
            for protocol_tree in protocol_trees {
                show_protocol_tree(builder, protocol_tree, &mut sidebar_actions);
            }
        },
    );

    for action in actions {
        match action {
            egui_ltreeview::Action::SetSelected(selected_ids) => {
                if let Some(selected_id) = selected_ids.first() {
                    *current_id = Some(selected_id.clone());
                    let (fields, inherited_field_count) = app
                        .protocol_registry
                        .resolve_fields(selected_id)
                        .unwrap_or_default();
                    app.protocol_designer_state.fields = fields;
                    app.protocol_designer_state.inherited_field_count = inherited_field_count;
                    app.protocol_designer_state.selected_field_index = None;
                } else {
                    *current_id = None;
                }
            }
            _ => {}
        }
    }

    for action in sidebar_actions {
        match action {
            ContextMenuActions::AddProtocol(parent_id) => {
                app.protocol_designer_state.is_adding_protocol = true;
                app.protocol_designer_state.new_protocol_parent_id = Some(parent_id);
                app.protocol_designer_state.focus_new_protocol_id = true;
            }
            ContextMenuActions::DeleteProtocol(protocol_id) => {
                app.protocol_designer_state.is_confirming_delete = true;
                app.protocol_designer_state.delete_target_id = Some(protocol_id);
            }
        }
    }
}

/// Show a protocol and all its children in the sidebar tree.
fn show_protocol_tree(
    builder: &mut TreeViewBuilder<String>,
    protocol_tree: &ProtocolTreeNode,
    action_queue: &mut Vec<ContextMenuActions>,
) {
    let id = &protocol_tree.protocol_id;
    // use protocol name as label if exists, otherwise use id
    let display_label = protocol_tree.protocol_name.as_ref().unwrap_or(id);

    let mut node = if protocol_tree.children.is_empty() {
        NodeBuilder::leaf(id.clone()).label(display_label.clone())
    } else {
        NodeBuilder::dir(id.clone()).label(display_label.clone())
    };

    node = node.context_menu(|ui| {
        if ui.button("Add subprotocol").clicked() {
            action_queue.push(ContextMenuActions::AddProtocol(id.clone()));
            ui.close();
        }

        if ui.button("Delete").clicked() {
            action_queue.push(ContextMenuActions::DeleteProtocol(id.clone()));
            ui.close();
        }
    });

    let is_open = builder.node(node);

    if !protocol_tree.children.is_empty() {
        if !is_open {
            builder.close_dir();
            return;
        }

        for child in &protocol_tree.children {
            show_protocol_tree(builder, child, action_queue);
        }
        builder.close_dir();
    }
}
