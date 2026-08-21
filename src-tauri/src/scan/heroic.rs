use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

use crate::models::{Runner, ScannedGame, Source};

fn config_roots() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return Vec::new(),
    };
    [
        home.join(".config/heroic"),
        home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic"),
    ]
    .into_iter()
    .filter(|path| path.is_dir())
    .collect()
}

/// Heroic keeps one cache file per store; they all expose a `library` array
/// with `app_name`, `title`, `is_installed` and an `install` object.
const LIBRARY_FILES: &[&str] = &[
    "store_cache/legendary_library.json",
    "store_cache/gog_library.json",
    "store_cache/nile_library.json",
    "legendaryConfig/legendary/installed.json",
    "gog_store/library.json",
];

pub fn scan() -> Result<Vec<ScannedGame>> {
    let mut games = Vec::new();
    for root in config_roots() {
        for relative in LIBRARY_FILES {
            let path = root.join(relative);
            let Ok(contents) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(json) = serde_json::from_str::<Value>(&contents) else {
                continue;
            };
            collect(&json, &mut games);
        }
    }
    games.sort_by(|a, b| a.name.cmp(&b.name));
    games.dedup_by(|a, b| a.external_id == b.external_id);
    Ok(games)
}

fn collect(json: &Value, games: &mut Vec<ScannedGame>) {
    let entries: Vec<&Value> = match json.get("library").and_then(Value::as_array) {
        Some(array) => array.iter().collect(),
        None => match json {
            Value::Array(array) => array.iter().collect(),
            Value::Object(map) => map.values().collect(),
            _ => Vec::new(),
        },
    };

    for entry in entries {
        let Some(app_name) = entry
            .get("app_name")
            .or_else(|| entry.get("appName"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let name = entry
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(app_name)
            .to_string();
        let install = entry.get("install");
        let install_dir = install
            .and_then(|install| install.get("install_path"))
            .or_else(|| entry.get("install_path"))
            .and_then(Value::as_str)
            .map(|path| path.to_string());
        let installed = entry
            .get("is_installed")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| {
                install_dir
                    .as_ref()
                    .is_some_and(|dir| PathBuf::from(dir).is_dir())
            });

        if games
            .iter()
            .any(|game| game.external_id.as_deref() == Some(app_name))
        {
            continue;
        }

        games.push(ScannedGame {
            name,
            source: Source::Heroic,
            runner: Runner::Heroic,
            external_id: Some(app_name.to_string()),
            install_dir,
            executable: None,
            installed,
            playtime_seconds: 0,
            prefix_path: None,
        });
    }
}
