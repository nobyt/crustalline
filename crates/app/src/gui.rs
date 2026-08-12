use std::sync::Mutex;

use crate::commands::{self, AppState};

/// Shared Tauri builder setup (state + command registration) used by both the
/// normal GUI entry point and the headless render pipeline, so the two never
/// drift apart (plan §5: headless reuses the same webview/command surface).
pub fn base_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::molecule::load_smiles,
            commands::molecule::get_mol_block,
            commands::molecule::get_svg,
            commands::molecule::export_svg,
            commands::edit::add_atom,
            commands::edit::remove_atom,
            commands::edit::add_bond,
            commands::edit::remove_bond,
            commands::edit::set_bond_order,
            commands::edit::set_formal_charge,
            commands::edit::undo,
            commands::edit::redo,
        ])
}

pub fn run() {
    base_builder()
        .run(tauri::generate_context!())
        .expect("error while running crustalline");
}
