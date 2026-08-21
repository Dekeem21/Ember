use std::collections::HashMap;

use serde_json::Value;
use tauri::{AppHandle, State};

use crate::db::game_id as make_game_id;
use crate::launch;
use crate::metadata;
use crate::models::{
    Achievement, Game, LibraryStats, PlaySession, Runner, ScanReport, Settings, Source,
    TrophySummary,
};
use crate::scan;
use crate::state::AppState;

/// Tauri commands report failures as plain strings so the UI can surface them.
type CommandResult<T> = Result<T, String>;

fn to_error<E: std::fmt::Display>(error: E) -> String {
    error.to_string()
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>, include_hidden: bool) -> CommandResult<Vec<Game>> {
    let db = state.db.lock().map_err(to_error)?;
    db.list_games(include_hidden).map_err(to_error)
}

#[tauri::command]
pub fn get_game(state: State<'_, AppState>, game_id: String) -> CommandResult<Option<Game>> {
    let db = state.db.lock().map_err(to_error)?;
    db.get_game(&game_id).map_err(to_error)
}

#[tauri::command]
pub fn library_stats(state: State<'_, AppState>) -> CommandResult<LibraryStats> {
    let db = state.db.lock().map_err(to_error)?;
    db.library_stats().map_err(to_error)
}

#[tauri::command]
pub fn running_games(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    Ok(state.running_games())
}

#[tauri::command]
pub fn scan_library(state: State<'_, AppState>) -> CommandResult<ScanReport> {
    let db = state.db.lock().map_err(to_error)?;
    let settings = db.load_settings().map_err(to_error)?;
    scan::scan_all(&db, &settings).map_err(to_error)
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: String,
) -> CommandResult<()> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = launch::launch(app, state, game_id).await {
            log::error!("launch failed: {error}");
        }
    });
    Ok(())
}

#[tauri::command]
pub fn stop_game(state: State<'_, AppState>, game_id: String) -> CommandResult<()> {
    let game = {
        let db = state.db.lock().map_err(to_error)?;
        db.get_game(&game_id).map_err(to_error)?
    };
    let game = game.ok_or_else(|| format!("unknown game {game_id}"))?;
    launch::stop(&game).map_err(to_error)
}

#[tauri::command]
pub fn update_game(
    state: State<'_, AppState>,
    game_id: String,
    patch: Value,
) -> CommandResult<Option<Game>> {
    let db = state.db.lock().map_err(to_error)?;
    db.update_game_fields(&game_id, &patch).map_err(to_error)?;
    if let Some(genres) = patch.get("genres").and_then(Value::as_array) {
        let genres: Vec<String> = genres
            .iter()
            .filter_map(Value::as_str)
            .map(|value| value.to_string())
            .collect();
        let tags: Vec<String> = patch
            .get("tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(|value| value.to_string())
                    .collect()
            })
            .unwrap_or_default();
        db.set_lists(&game_id, &genres, &tags).map_err(to_error)?;
    }
    db.get_game(&game_id).map_err(to_error)
}

#[tauri::command]
pub fn set_game_flag(
    state: State<'_, AppState>,
    game_id: String,
    flag: String,
    value: bool,
) -> CommandResult<()> {
    let db = state.db.lock().map_err(to_error)?;
    db.toggle_flag(&game_id, &flag, value).map_err(to_error)
}

#[tauri::command]
pub fn delete_game(state: State<'_, AppState>, game_id: String) -> CommandResult<()> {
    let db = state.db.lock().map_err(to_error)?;
    db.delete_game(&game_id).map_err(to_error)
}

#[tauri::command]
pub fn add_manual_game(
    state: State<'_, AppState>,
    name: String,
    executable: String,
    runner: String,
) -> CommandResult<String> {
    let db = state.db.lock().map_err(to_error)?;
    db.add_manual_game(&name, &executable, Runner::from_str(&runner))
        .map_err(to_error)
}

#[tauri::command]
pub fn game_sessions(
    state: State<'_, AppState>,
    game_id: String,
    limit: Option<i64>,
) -> CommandResult<Vec<PlaySession>> {
    let db = state.db.lock().map_err(to_error)?;
    db.sessions_for_game(&game_id, limit.unwrap_or(20))
        .map_err(to_error)
}

#[tauri::command]
pub fn game_achievements(
    state: State<'_, AppState>,
    game_id: String,
) -> CommandResult<Vec<Achievement>> {
    let db = state.db.lock().map_err(to_error)?;
    db.achievements_for_game(&game_id).map_err(to_error)
}

#[tauri::command]
pub fn trophy_summary(
    state: State<'_, AppState>,
    game_id: Option<String>,
) -> CommandResult<TrophySummary> {
    let db = state.db.lock().map_err(to_error)?;
    db.trophy_summary(game_id.as_deref()).map_err(to_error)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<Settings> {
    let db = state.db.lock().map_err(to_error)?;
    db.load_settings().map_err(to_error)
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> CommandResult<Settings> {
    let db = state.db.lock().map_err(to_error)?;
    db.save_settings(&settings).map_err(to_error)?;
    db.load_settings().map_err(to_error)
}

#[tauri::command]
pub fn proton_versions() -> CommandResult<Vec<String>> {
    Ok(launch::available_proton_versions())
}

#[tauri::command]
pub fn env_presets() -> CommandResult<HashMap<String, String>> {
    Ok(launch::known_env_presets()
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect())
}

/// Reports whether umu-launcher is installed, so the UI can nudge the user.
#[tauri::command]
pub fn umu_status(state: State<'_, AppState>) -> CommandResult<Value> {
    let settings = {
        let db = state.db.lock().map_err(to_error)?;
        db.load_settings().map_err(to_error)?
    };
    let output = std::process::Command::new(&settings.umu_run_path)
        .arg("--version")
        .output();
    let (available, version) = match output {
        Ok(output) => (
            true,
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ),
        Err(_) => (false, String::new()),
    };
    Ok(serde_json::json!({
        "available": available,
        "version": version,
        "path": settings.umu_run_path,
    }))
}

#[tauri::command]
pub async fn refresh_metadata(
    state: State<'_, AppState>,
    game_id: String,
) -> CommandResult<Option<Game>> {
    let (game, settings) = {
        let db = state.db.lock().map_err(to_error)?;
        let game = db
            .get_game(&game_id)
            .map_err(to_error)?
            .ok_or_else(|| format!("unknown game {game_id}"))?;
        (game, db.load_settings().map_err(to_error)?)
    };

    let fetched = metadata::fetch_for_game(&game, &settings)
        .await
        .map_err(to_error)?;

    let patch = serde_json::json!({
        "description": fetched.description,
        "developer": fetched.developer,
        "publisher": fetched.publisher,
        "release_date": fetched.release_date,
        "trailer_url": fetched.trailer_url,
        "cover_path": fetched.cover_path,
        "hero_path": fetched.hero_path,
        "logo_path": fetched.logo_path,
    });
    let patch = Value::Object(
        patch
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, value)| !value.is_null())
            .collect(),
    );

    let db = state.db.lock().map_err(to_error)?;
    db.update_game_fields(&game_id, &patch).map_err(to_error)?;
    if !fetched.genres.is_empty() {
        db.set_lists(&game_id, &fetched.genres, &game.tags)
            .map_err(to_error)?;
    }
    db.get_game(&game_id).map_err(to_error)
}

/// Fetches metadata for every game that is still missing artwork.
#[tauri::command]
pub async fn refresh_missing_metadata(state: State<'_, AppState>) -> CommandResult<usize> {
    let (games, settings) = {
        let db = state.db.lock().map_err(to_error)?;
        (
            db.list_games(true).map_err(to_error)?,
            db.load_settings().map_err(to_error)?,
        )
    };

    let mut updated = 0;
    for game in games.into_iter().filter(|game| game.cover_path.is_none()) {
        let Ok(fetched) = metadata::fetch_for_game(&game, &settings).await else {
            continue;
        };
        let patch = serde_json::json!({
            "description": fetched.description,
            "developer": fetched.developer,
            "publisher": fetched.publisher,
            "release_date": fetched.release_date,
            "trailer_url": fetched.trailer_url,
            "cover_path": fetched.cover_path,
            "hero_path": fetched.hero_path,
            "logo_path": fetched.logo_path,
        });
        let patch = Value::Object(
            patch
                .as_object()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|(_, value)| !value.is_null())
                .collect(),
        );
        let db = state.db.lock().map_err(to_error)?;
        db.update_game_fields(&game.id, &patch).map_err(to_error)?;
        if !fetched.genres.is_empty() {
            db.set_lists(&game.id, &fetched.genres, &game.tags)
                .map_err(to_error)?;
        }
        updated += 1;
    }
    Ok(updated)
}

/// Imports lifetime playtime from the Steam Web API for owned Steam games.
#[tauri::command]
pub async fn sync_steam_playtime(state: State<'_, AppState>) -> CommandResult<usize> {
    let settings = {
        let db = state.db.lock().map_err(to_error)?;
        db.load_settings().map_err(to_error)?
    };
    let playtimes = metadata::fetch_steam_playtime(&settings)
        .await
        .map_err(to_error)?;

    let db = state.db.lock().map_err(to_error)?;
    let mut updated = 0;
    for (appid, seconds) in playtimes {
        if seconds == 0 {
            continue;
        }
        let id = make_game_id(Source::Steam, &appid);
        if db.get_game(&id).map_err(to_error)?.is_none() {
            continue;
        }
        db.import_playtime(&id, seconds).map_err(to_error)?;
        updated += 1;
    }
    Ok(updated)
}

#[tauri::command]
pub async fn sync_achievements(
    state: State<'_, AppState>,
    game_id: String,
) -> CommandResult<TrophySummary> {
    let (game, settings) = {
        let db = state.db.lock().map_err(to_error)?;
        let game = db
            .get_game(&game_id)
            .map_err(to_error)?
            .ok_or_else(|| format!("unknown game {game_id}"))?;
        (game, db.load_settings().map_err(to_error)?)
    };

    let appid = metadata::resolve_steam_appid(&game)
        .await
        .map_err(to_error)?
        .ok_or_else(|| format!("no Steam appid for {}", game.name))?;
    let achievements = metadata::fetch_achievements(&game, &appid, &settings)
        .await
        .map_err(to_error)?;

    let db = state.db.lock().map_err(to_error)?;
    db.replace_achievements(&game_id, &achievements)
        .map_err(to_error)?;
    db.trophy_summary(Some(&game_id)).map_err(to_error)
}
