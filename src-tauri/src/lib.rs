mod commands;
mod db;
mod launch;
mod metadata;
mod models;
mod scan;
mod state;

use db::{default_db_path, Database};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let database = Database::open(&default_db_path()).expect("failed to open the Ember library");
    let app_state = AppState::new(database);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::get_game,
            commands::library_stats,
            commands::running_games,
            commands::scan_library,
            commands::launch_game,
            commands::stop_game,
            commands::update_game,
            commands::set_game_flag,
            commands::delete_game,
            commands::add_manual_game,
            commands::game_sessions,
            commands::game_achievements,
            commands::trophy_summary,
            commands::get_settings,
            commands::save_settings,
            commands::proton_versions,
            commands::env_presets,
            commands::umu_status,
            commands::refresh_metadata,
            commands::refresh_missing_metadata,
            commands::sync_steam_playtime,
            commands::sync_achievements,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ember");
}
