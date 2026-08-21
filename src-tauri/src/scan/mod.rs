pub mod folder;
pub mod heroic;
pub mod lutris;
pub mod steam;
pub mod vdf;

use anyhow::Result;

use crate::db::Database;
use crate::models::{ScanReport, ScannedGame, Settings, Source, SourceReport};

/// Runs every store scanner and merges the results into the library.
pub fn scan_all(db: &Database, settings: &Settings) -> Result<ScanReport> {
    let mut report = ScanReport::default();

    let sources: Vec<(Source, Result<Vec<ScannedGame>>)> = vec![
        (Source::Steam, steam::scan()),
        (Source::Heroic, heroic::scan()),
        (Source::Lutris, lutris::scan()),
        (Source::Manual, folder::scan(&settings.extra_library_dirs)),
    ];

    for (source, result) in sources {
        match result {
            Ok(games) => {
                report.sources.push(SourceReport {
                    source,
                    found: games.len(),
                });
                for game in games {
                    match db.upsert_scanned(&game) {
                        Ok((_, true)) => report.added += 1,
                        Ok((_, false)) => report.updated += 1,
                        Err(error) => report.errors.push(format!("{}: {error}", game.name)),
                    }
                }
            }
            Err(error) => report
                .errors
                .push(format!("{} scan failed: {error}", source.as_str())),
        }
    }

    Ok(report)
}
