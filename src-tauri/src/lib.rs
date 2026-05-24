mod commands;
mod engine;
mod levels;
mod progress;
mod state;

#[cfg(test)]
mod solutions_test;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::execute_command,
            commands::get_ruleset,
            commands::reset_ruleset,
            commands::get_level_list,
            commands::load_level,
            commands::check_tests,
            commands::mark_level_complete,
            commands::get_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
