use std::path::PathBuf;

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};

use crate::models::{Runner, ScannedGame, Source};

fn pga_paths() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return Vec::new(),
    };
    [
        home.join(".local/share/lutris/pga.db"),
        home.join(".var/app/net.lutris.Lutris/data/lutris/pga.db"),
    ]
    .into_iter()
    .filter(|path| path.is_file())
    .collect()
}

pub fn scan() -> Result<Vec<ScannedGame>> {
    let mut games = Vec::new();
    for path in pga_paths() {
        let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut statement = connection.prepare(
            "SELECT slug, name, directory, installed, COALESCE(playtime, 0) FROM games WHERE name IS NOT NULL",
        )?;
        let rows = statement.query_map([], |row| {
            let slug: String = row.get(0)?;
            let name: String = row.get(1)?;
            let directory: Option<String> = row.get(2)?;
            let installed: i64 = row.get(3).unwrap_or(0);
            // Lutris stores playtime as fractional hours.
            let playtime_hours: f64 = row.get::<_, f64>(4).unwrap_or(0.0);
            Ok(ScannedGame {
                name,
                source: Source::Lutris,
                runner: Runner::Lutris,
                external_id: Some(slug),
                install_dir: directory,
                executable: None,
                installed: installed != 0,
                playtime_seconds: (playtime_hours * 3600.0) as i64,
                prefix_path: None,
            })
        })?;
        for row in rows {
            games.push(row?);
        }
    }
    Ok(games)
}
