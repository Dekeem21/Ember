use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tauri::{AppHandle, Emitter};
use tokio::process::Command;

use crate::models::{Game, Runner, Settings};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub game_id: String,
    pub name: String,
    pub duration_seconds: i64,
}

/// Builds the command line for a game, including the umu-launcher environment
/// used for Windows titles.
pub fn build_command(game: &Game, settings: &Settings) -> Result<Command> {
    let mut command = match game.runner {
        Runner::Native => {
            let executable = game
                .executable
                .clone()
                .ok_or_else(|| anyhow!("no executable configured for {}", game.name))?;
            Command::new(executable)
        }
        Runner::Umu => {
            let executable = game
                .executable
                .clone()
                .ok_or_else(|| anyhow!("no executable configured for {}", game.name))?;
            let mut command = Command::new(&settings.umu_run_path);
            command.arg(executable);
            let proton = game
                .proton_version
                .clone()
                .unwrap_or_else(|| settings.default_proton_version.clone());
            command.env("PROTONPATH", proton);
            command.env("GAMEID", umu_game_id(game));
            if let Some(store) = umu_store(game) {
                command.env("STORE", store);
            }
            command.env("WINEPREFIX", prefix_for(game, settings));
            command
        }
        Runner::Steam => {
            let appid = game
                .external_id
                .clone()
                .ok_or_else(|| anyhow!("missing Steam appid for {}", game.name))?;
            let mut command = Command::new("steam");
            command.arg(format!("steam://rungameid/{appid}"));
            command
        }
        Runner::Heroic => {
            let app_name = game
                .external_id
                .clone()
                .ok_or_else(|| anyhow!("missing Heroic app name for {}", game.name))?;
            let mut command = Command::new("xdg-open");
            command.arg(format!("heroic://launch/{app_name}"));
            command
        }
        Runner::Lutris => {
            let slug = game
                .external_id
                .clone()
                .ok_or_else(|| anyhow!("missing Lutris slug for {}", game.name))?;
            let mut command = Command::new("lutris");
            command.arg(format!("lutris:rungame/{slug}"));
            command
        }
    };

    if let Some(args) = game
        .launch_args
        .as_ref()
        .filter(|args| !args.trim().is_empty())
    {
        command.args(split_args(args));
    }
    if let Some(dir) = game
        .install_dir
        .as_ref()
        .filter(|dir| PathBuf::from(dir).is_dir())
    {
        command.current_dir(dir);
    }
    for (key, value) in parse_env(game.env_vars.as_deref()) {
        command.env(key, value);
    }
    command.stdout(Stdio::null()).stderr(Stdio::null());
    Ok(command)
}

/// umu uses the game id to pick up per-game Proton fixes from its protonfixes
/// database; Steam titles use `umu-<appid>`.
fn umu_game_id(game: &Game) -> String {
    match (game.source, game.external_id.as_deref()) {
        (crate::models::Source::Steam, Some(appid)) => format!("umu-{appid}"),
        _ => "umu-default".to_string(),
    }
}

fn umu_store(game: &Game) -> Option<&'static str> {
    match game.source {
        crate::models::Source::Steam => Some("steam"),
        crate::models::Source::Heroic => Some("egs"),
        _ => None,
    }
}

fn prefix_for(game: &Game, settings: &Settings) -> PathBuf {
    if let Some(prefix) = game.prefix_path.as_ref().filter(|p| !p.trim().is_empty()) {
        return PathBuf::from(prefix);
    }
    let root = settings
        .prefix_root
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("ember")
                .join("prefixes")
        });
    root.join(&game.id)
}

fn parse_env(env_vars: Option<&str>) -> Vec<(String, String)> {
    env_vars
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

/// Splits a launch-argument string on spaces while honouring quotes.
fn split_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for character in args.chars() {
        match character {
            '"' | '\'' if quote.is_none() => quote = Some(character),
            c if Some(c) == quote => quote = None,
            c if c.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// Launches a game, records a play session and emits lifecycle events.
pub async fn launch(app: AppHandle, state: AppState, game_id: String) -> Result<()> {
    let (game, settings) = {
        let db = state.db.lock().expect("db lock");
        let game = db
            .get_game(&game_id)?
            .ok_or_else(|| anyhow!("unknown game {game_id}"))?;
        (game, db.load_settings()?)
    };

    if state.is_running(&game.id) {
        bail!("{} is already running", game.name);
    }

    let mut command = build_command(&game, &settings)?;
    let mut child = command.spawn()?;

    let session_id = {
        let db = state.db.lock().expect("db lock");
        db.start_session(&game.id)?
    };
    state.mark_running(&game.id);
    let started = Instant::now();
    let _ = app.emit(
        "game-started",
        GameEvent {
            game_id: game.id.clone(),
            name: game.name.clone(),
            duration_seconds: 0,
        },
    );

    let _ = child.wait().await;

    // Store clients hand the game off to a detached process, so keep watching
    // the install directory until nothing is running from it any more.
    if matches!(game.runner, Runner::Steam | Runner::Heroic | Runner::Lutris) {
        if let Some(install_dir) = game.install_dir.clone() {
            wait_for_directory_idle(&install_dir).await;
        }
    }

    let duration = started.elapsed().as_secs() as i64;
    {
        let db = state.db.lock().expect("db lock");
        db.finish_session(session_id, duration)?;
    }
    state.mark_stopped(&game.id);
    let _ = app.emit(
        "game-stopped",
        GameEvent {
            game_id: game.id.clone(),
            name: game.name.clone(),
            duration_seconds: duration,
        },
    );
    Ok(())
}

async fn wait_for_directory_idle(install_dir: &str) {
    let grace = Duration::from_secs(60);
    let poll = Duration::from_secs(5);
    let deadline = Instant::now() + grace;
    let mut seen_running = false;

    loop {
        let running = processes_under(install_dir);
        if running {
            seen_running = true;
        } else if seen_running || Instant::now() > deadline {
            return;
        }
        tokio::time::sleep(poll).await;
    }
}

fn processes_under(install_dir: &str) -> bool {
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    system.refresh_processes(ProcessesToUpdate::All, true);
    system.processes().values().any(|process| {
        process
            .exe()
            .map(|exe| exe.to_string_lossy().starts_with(install_dir))
            .unwrap_or(false)
            || process
                .cwd()
                .map(|cwd| cwd.to_string_lossy().starts_with(install_dir))
                .unwrap_or(false)
    })
}

/// Best-effort kill of everything spawned from the game's install directory.
pub fn stop(game: &Game) -> Result<()> {
    let Some(install_dir) = game.install_dir.clone() else {
        bail!("no install directory known for {}", game.name);
    };
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    system.refresh_processes(ProcessesToUpdate::All, true);
    for process in system.processes().values() {
        let matches = process
            .exe()
            .map(|exe| exe.to_string_lossy().starts_with(&install_dir))
            .unwrap_or(false);
        if matches {
            process.kill();
        }
    }
    Ok(())
}

/// Proton builds usable as `PROTONPATH`, discovered from the usual install roots.
pub fn available_proton_versions() -> Vec<String> {
    let mut versions = vec!["UMU-Latest".to_string(), "GE-Proton".to_string()];
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return versions,
    };
    let roots = [
        home.join(".steam/root/compatibilitytools.d"),
        home.join(".local/share/Steam/compatibilitytools.d"),
        home.join(".local/share/Steam/steamapps/common"),
        PathBuf::from("/usr/share/steam/compatibilitytools.d"),
    ];
    for root in roots {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_lowercase().contains("proton") && !versions.contains(&name) {
                versions.push(name);
            }
        }
    }
    versions
}

/// Environment variables Ember understands per game, surfaced in the UI.
pub fn known_env_presets() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("DXVK_HUD", "fps"),
        ("PROTON_ENABLE_NVAPI", "1"),
        ("PROTON_USE_WINED3D", "1"),
        ("WINEDLLOVERRIDES", "winmm=n,b"),
        ("MANGOHUD", "1"),
        ("OBS_VKCAPTURE", "1"),
    ])
}
