use std::path::{Path, PathBuf};

use anyhow::Result;

use super::vdf;
use crate::models::{Runner, ScannedGame, Source};

/// Steam tooling that shows up as an "app" but is not a game.
const IGNORED_APPIDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "1070560", // Steam Linux Runtime 1.0
    "1391110", // Steam Linux Runtime 2.0 (soldier)
    "1628350", // Steam Linux Runtime 3.0 (sniper)
    "1493710", // Proton Experimental
];

pub fn steam_roots() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return Vec::new(),
    };
    [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
    ]
    .into_iter()
    .filter(|path| path.join("steamapps").is_dir())
    .collect()
}

/// Every `steamapps` directory Steam knows about, including extra library disks.
pub fn library_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for root in steam_roots() {
        let steamapps = root.join("steamapps");
        if steamapps.is_dir() && !dirs.contains(&steamapps) {
            dirs.push(steamapps.clone());
        }
        let library_file = steamapps.join("libraryfolders.vdf");
        let Ok(contents) = std::fs::read_to_string(&library_file) else {
            continue;
        };
        let parsed = vdf::parse(&contents);
        let Some(folders) = parsed.get("libraryfolders") else {
            continue;
        };
        for (_, entry) in folders.entries() {
            let path = match entry {
                vdf::Value::String(path) => Some(path.clone()),
                vdf::Value::Object(_) => entry
                    .get("path")
                    .and_then(vdf::Value::as_str)
                    .map(|s| s.to_string()),
            };
            if let Some(path) = path {
                let steamapps = PathBuf::from(path).join("steamapps");
                if steamapps.is_dir() && !dirs.contains(&steamapps) {
                    dirs.push(steamapps);
                }
            }
        }
    }
    dirs
}

pub fn scan() -> Result<Vec<ScannedGame>> {
    let mut games = Vec::new();
    for steamapps in library_dirs() {
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_manifest = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"));
            if !is_manifest {
                continue;
            }
            if let Some(game) = parse_manifest(&path, &steamapps) {
                games.push(game);
            }
        }
    }
    Ok(games)
}

fn parse_manifest(path: &Path, steamapps: &Path) -> Option<ScannedGame> {
    let contents = std::fs::read_to_string(path).ok()?;
    let parsed = vdf::parse(&contents);
    let state = parsed.get("AppState")?;
    let appid = state.get("appid").and_then(vdf::Value::as_str)?.to_string();
    if IGNORED_APPIDS.contains(&appid.as_str()) {
        return None;
    }
    let name = state.get("name").and_then(vdf::Value::as_str)?.to_string();
    let install_dir = state
        .get("installdir")
        .and_then(vdf::Value::as_str)
        .map(|dir| steamapps.join("common").join(dir));

    Some(ScannedGame {
        name,
        source: Source::Steam,
        // Steam owns its own Proton prefix, so launching goes back through Steam.
        runner: Runner::Steam,
        external_id: Some(appid),
        install_dir: install_dir
            .as_ref()
            .map(|dir| dir.to_string_lossy().to_string()),
        executable: None,
        installed: install_dir.as_ref().is_some_and(|dir| dir.is_dir()),
        playtime_seconds: 0,
        prefix_path: None,
    })
}

/// `~/.steam/steam/appcache/librarycache/<appid>/` holds artwork Steam already downloaded.
pub fn local_artwork(appid: &str) -> Option<(Option<PathBuf>, Option<PathBuf>, Option<PathBuf>)> {
    let root = steam_roots().into_iter().next()?;
    let cache = root.join("appcache/librarycache").join(appid);
    let pick = |names: &[&str]| -> Option<PathBuf> {
        names
            .iter()
            .map(|name| cache.join(name))
            .find(|path| path.is_file())
    };
    if !cache.is_dir() {
        return None;
    }
    Some((
        pick(&["library_600x900.jpg", "library_600x900_2x.jpg"]),
        pick(&["library_hero.jpg", "library_hero_blur.jpg"]),
        pick(&["logo.png", "library_logo.png"]),
    ))
}
