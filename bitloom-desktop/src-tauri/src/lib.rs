// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use bitloom_core::protocol::{Endianness, ProtocolRegistry};
use std::sync::Mutex;
use tauri::State;

struct AppState {
    registry: Mutex<ProtocolRegistry>,
}

impl AppState {
    fn new() -> Self {
        let mut registry = ProtocolRegistry::new();
        registry
            .create_protocol(
                "ethernet",
                Some("Ethernet II".to_string()),
                Endianness::Big,
                None,
            )
            .expect("failed to create ethernet protocol");
        registry
            .create_protocol(
                "ipv4",
                Some("IPv4".to_string()),
                Endianness::Big,
                Some("ethernet".to_string()),
            )
            .expect("failed to create ipv4 protocol");
        registry
            .create_protocol(
                "ipv6",
                Some("IPv6".to_string()),
                Endianness::Big,
                Some("ethernet".to_string()),
            )
            .expect("failed to create ipv6 protocol");

        Self {
            registry: Mutex::new(registry),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_protocol_tree(
    state: State<'_, AppState>,
) -> Result<Vec<bitloom_core::protocol::ProtocolTreeNode>, String> {
    let registry = state.registry.lock().map_err(|error| error.to_string())?;
    Ok(registry.build_protocol_trees())
}

#[tauri::command]
fn create_protocol(
    state: State<'_, AppState>,
    id: String,
    parent_id: Option<String>,
) -> Result<(), String> {
    let mut registry = state.registry.lock().map_err(|error| error.to_string())?;
    registry.create_protocol(&id, Some(id.clone()), Endianness::Big, parent_id)
}

#[tauri::command]
fn update_protocol_remark(
    state: State<'_, AppState>,
    id: String,
    remark: String,
) -> Result<(), String> {
    let mut registry = state.registry.lock().map_err(|error| error.to_string())?;
    registry.edit_protocol(&id, |protocol| {
        protocol.remark = Some(remark);
        Ok(())
    })
}

#[tauri::command]
fn delete_protocol(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut registry = state.registry.lock().map_err(|error| error.to_string())?;
    registry.remove_protocol(&id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_protocol_tree,
            create_protocol,
            update_protocol_remark,
            delete_protocol
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
