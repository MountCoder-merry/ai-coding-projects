mod commands;
mod errors;
mod filesystem;
mod history;
mod organizer;
mod rule_engine;
mod rule_store;
mod rules;
mod scanner;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::get_app_info,
            commands::scan_folder,
            commands::get_desktop_folder,
            commands::generate_plan,
            commands::apply_plan,
            commands::undo_transaction,
            commands::load_history,
            commands::validate_rule,
            commands::parse_rule_mock,
            commands::load_rules,
            commands::save_rule,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Smart Organizer");
}
