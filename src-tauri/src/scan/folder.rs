use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::models::{Runner, ScannedGame, Source};

/// Installers and helper binaries that ship next to real game executables.
const NOISE: &[&str] = &[
    "unins",
    "setup",
    "vcredist",
    "directx",
    "dxsetup",
    "dotnet",
    "crashreport",
    "crashhandler",
    "launcher_helper",
    "redist",
    "touchup",
];

/// Walks user-provided directories and treats each Windows/native executable
/// as a candidate game, the way Playnite's manual folder import does.
pub fn scan(dirs: &[String]) -> Result<Vec<ScannedGame>> {
    let mut games = Vec::new();
    for dir in dirs {
        let root = Path::new(dir);
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(4)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let lowered = file_name.to_lowercase();
            if NOISE.iter().any(|noise| lowered.contains(noise)) {
                continue;
            }
            let runner = if lowered.ends_with(".exe") {
                Runner::Umu
            } else if is_native_executable(path) {
                Runner::Native
            } else {
                continue;
            };

            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(file_name)
                .replace(['_', '-'], " ");

            games.push(ScannedGame {
                name,
                source: Source::Manual,
                runner,
                external_id: Some(path.to_string_lossy().to_string()),
                install_dir: path
                    .parent()
                    .map(|parent| parent.to_string_lossy().to_string()),
                executable: Some(path.to_string_lossy().to_string()),
                installed: true,
                playtime_seconds: 0,
                prefix_path: None,
            });
        }
    }
    Ok(games)
}

#[cfg(unix)]
fn is_native_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if metadata.permissions().mode() & 0o111 == 0 {
        return false;
    }
    // Skip shared objects and other non-entrypoint files.
    !path.to_string_lossy().to_lowercase().ends_with(".so")
}

#[cfg(not(unix))]
fn is_native_executable(_path: &Path) -> bool {
    false
}
