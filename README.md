# BitLoom

BitLoom is a desktop tool for designing and testing custom binary protocols.
It is intended to bring protocol definitions, packet construction, sending, and scripting into one workspace.

The project is in early development and is currently being migrated from a
Rust/egui prototype to Tauri and Svelte.

## Current Status

BitLoom currently has an independent Rust core and an early desktop editor
prototype. Development is focused on completing the protocol design workflow
and connecting the interface to real protocol data.

## Roadmap

- Complete the protocol editor
- Build and inspect packet instances
- Add sending and scripting
- Add project files and multi-document workflows

## Project Structure

```text
bitloom/
├── bitloom-core/       # Protocol model and validation logic
├── bitloom-desktop/    # Tauri + Svelte desktop application
└── src/                # Original egui prototype
```
