use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use sha2::{Digest, Sha256};

use crate::models::{
    Achievement, Game, LibraryStats, PlaySession, Runner, ScannedGame, Settings, Source,
    TrophySummary,
};

pub struct Database {
    conn: Connection,
}

/// Deterministic id so re-scanning a store never duplicates a game.
pub fn game_id(source: Source, key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_str().as_bytes());
    hasher.update(b":");
    hasher.update(key.to_lowercase().as_bytes());
    format!("{}-{:x}", source.as_str(), hasher.finalize())[..24].to_string()
}

fn join_list(items: &[String]) -> String {
    items.join("\u{1f}")
}

fn split_list(value: Option<String>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split('\u{1f}')
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().to_string())
        .collect()
}

impl Database {
    pub fn open(path: &Path) -> Result<Database> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path).with_context(|| format!("opening {:?}", path))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source TEXT NOT NULL,
                runner TEXT NOT NULL,
                external_id TEXT,
                install_dir TEXT,
                executable TEXT,
                launch_args TEXT,
                proton_version TEXT,
                prefix_path TEXT,
                env_vars TEXT,
                installed INTEGER NOT NULL DEFAULT 0,
                hidden INTEGER NOT NULL DEFAULT 0,
                favorite INTEGER NOT NULL DEFAULT 0,
                description TEXT,
                developer TEXT,
                publisher TEXT,
                release_date TEXT,
                genres TEXT,
                tags TEXT,
                cover_path TEXT,
                hero_path TEXT,
                logo_path TEXT,
                icon_path TEXT,
                trailer_url TEXT,
                playtime_seconds INTEGER NOT NULL DEFAULT 0,
                imported_playtime_seconds INTEGER NOT NULL DEFAULT 0,
                community_playtime_seconds INTEGER,
                last_played_at TEXT,
                added_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                duration_seconds INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS sessions_game_idx ON sessions(game_id);

            CREATE TABLE IF NOT EXISTS achievements (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                api_name TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                icon_url TEXT,
                unlocked INTEGER NOT NULL DEFAULT 0,
                unlocked_at TEXT,
                rarity REAL,
                UNIQUE(game_id, api_name)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_scanned(&self, scanned: &ScannedGame) -> Result<(String, bool)> {
        let key = scanned
            .external_id
            .clone()
            .unwrap_or_else(|| scanned.name.clone());
        let id = game_id(scanned.source, &key);
        let existing: Option<String> = self
            .conn
            .query_row("SELECT id FROM games WHERE id = ?1", params![id], |row| {
                row.get(0)
            })
            .optional()?;

        if existing.is_some() {
            self.conn.execute(
                r#"UPDATE games SET name = ?2, runner = ?3, install_dir = ?4, executable = ?5,
                       installed = ?6, imported_playtime_seconds = MAX(imported_playtime_seconds, ?7),
                       prefix_path = COALESCE(?8, prefix_path)
                   WHERE id = ?1"#,
                params![
                    id,
                    scanned.name,
                    scanned.runner.as_str(),
                    scanned.install_dir,
                    scanned.executable,
                    scanned.installed as i32,
                    scanned.playtime_seconds,
                    scanned.prefix_path,
                ],
            )?;
            return Ok((id, false));
        }

        self.conn.execute(
            r#"INSERT INTO games (id, name, source, runner, external_id, install_dir, executable,
                   installed, imported_playtime_seconds, prefix_path, added_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                id,
                scanned.name,
                scanned.source.as_str(),
                scanned.runner.as_str(),
                scanned.external_id,
                scanned.install_dir,
                scanned.executable,
                scanned.installed as i32,
                scanned.playtime_seconds,
                scanned.prefix_path,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok((id, true))
    }

    pub fn add_manual_game(&self, name: &str, executable: &str, runner: Runner) -> Result<String> {
        let scanned = ScannedGame {
            name: name.to_string(),
            source: Source::Manual,
            runner,
            external_id: Some(executable.to_string()),
            install_dir: Path::new(executable)
                .parent()
                .map(|p| p.to_string_lossy().to_string()),
            executable: Some(executable.to_string()),
            installed: true,
            playtime_seconds: 0,
            prefix_path: None,
        };
        let (id, _) = self.upsert_scanned(&scanned)?;
        Ok(id)
    }

    fn row_to_game(row: &Row<'_>) -> rusqlite::Result<Game> {
        let id: String = row.get("id")?;
        let playtime: i64 = row.get::<_, i64>("playtime_seconds")?
            + row.get::<_, i64>("imported_playtime_seconds")?;
        let session_count: i64 = row.get("session_count").unwrap_or(0);
        let tracked: i64 = row.get("tracked_seconds").unwrap_or(0);
        Ok(Game {
            id,
            name: row.get("name")?,
            source: Source::from_str(&row.get::<_, String>("source")?),
            runner: Runner::from_str(&row.get::<_, String>("runner")?),
            external_id: row.get("external_id")?,
            install_dir: row.get("install_dir")?,
            executable: row.get("executable")?,
            launch_args: row.get("launch_args")?,
            proton_version: row.get("proton_version")?,
            prefix_path: row.get("prefix_path")?,
            env_vars: row.get("env_vars")?,
            installed: row.get::<_, i64>("installed")? != 0,
            hidden: row.get::<_, i64>("hidden")? != 0,
            favorite: row.get::<_, i64>("favorite")? != 0,
            description: row.get("description")?,
            developer: row.get("developer")?,
            publisher: row.get("publisher")?,
            release_date: row.get("release_date")?,
            genres: split_list(row.get("genres")?),
            tags: split_list(row.get("tags")?),
            cover_path: row.get("cover_path")?,
            hero_path: row.get("hero_path")?,
            logo_path: row.get("logo_path")?,
            icon_path: row.get("icon_path")?,
            trailer_url: row.get("trailer_url")?,
            playtime_seconds: playtime,
            session_count,
            average_session_seconds: if session_count > 0 {
                tracked / session_count
            } else {
                0
            },
            community_playtime_seconds: row.get("community_playtime_seconds")?,
            last_played_at: row.get("last_played_at")?,
            added_at: row.get("added_at")?,
        })
    }

    const GAME_SELECT: &'static str = r#"
        SELECT g.*,
               (SELECT COUNT(*) FROM sessions s WHERE s.game_id = g.id AND s.ended_at IS NOT NULL) AS session_count,
               (SELECT COALESCE(SUM(s.duration_seconds), 0) FROM sessions s WHERE s.game_id = g.id) AS tracked_seconds
        FROM games g
    "#;

    pub fn list_games(&self, include_hidden: bool) -> Result<Vec<Game>> {
        let sql = format!(
            "{} {} ORDER BY g.last_played_at DESC NULLS LAST, g.name COLLATE NOCASE ASC",
            Self::GAME_SELECT,
            if include_hidden {
                ""
            } else {
                "WHERE g.hidden = 0"
            }
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let games = stmt
            .query_map([], Self::row_to_game)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(games)
    }

    pub fn get_game(&self, id: &str) -> Result<Option<Game>> {
        let sql = format!("{} WHERE g.id = ?1", Self::GAME_SELECT);
        let game = self
            .conn
            .query_row(&sql, params![id], Self::row_to_game)
            .optional()?;
        Ok(game)
    }

    pub fn update_game_fields(&self, id: &str, patch: &serde_json::Value) -> Result<()> {
        const EDITABLE: &[&str] = &[
            "name",
            "executable",
            "launch_args",
            "proton_version",
            "prefix_path",
            "env_vars",
            "description",
            "developer",
            "publisher",
            "release_date",
            "cover_path",
            "hero_path",
            "logo_path",
            "icon_path",
            "trailer_url",
            "community_playtime_seconds",
            "runner",
        ];
        let object = match patch.as_object() {
            Some(object) => object,
            None => return Ok(()),
        };
        for (key, value) in object {
            if !EDITABLE.contains(&key.as_str()) {
                continue;
            }
            let sql = format!("UPDATE games SET {} = ?2 WHERE id = ?1", key);
            match value {
                serde_json::Value::Null => {
                    self.conn
                        .execute(&sql, params![id, Option::<String>::None])?;
                }
                serde_json::Value::Number(number) => {
                    self.conn.execute(&sql, params![id, number.as_i64()])?;
                }
                other => {
                    let text = other
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| other.to_string());
                    self.conn.execute(&sql, params![id, text])?;
                }
            }
        }
        Ok(())
    }

    pub fn set_lists(&self, id: &str, genres: &[String], tags: &[String]) -> Result<()> {
        self.conn.execute(
            "UPDATE games SET genres = ?2, tags = ?3 WHERE id = ?1",
            params![id, join_list(genres), join_list(tags)],
        )?;
        Ok(())
    }

    pub fn toggle_flag(&self, id: &str, column: &str, value: bool) -> Result<()> {
        let column = match column {
            "favorite" => "favorite",
            "hidden" => "hidden",
            "installed" => "installed",
            _ => return Ok(()),
        };
        self.conn.execute(
            &format!("UPDATE games SET {} = ?2 WHERE id = ?1", column),
            params![id, value as i32],
        )?;
        Ok(())
    }

    pub fn delete_game(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM games WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Stores playtime imported from a store, keeping the larger of the two values.
    pub fn import_playtime(&self, id: &str, seconds: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE games SET imported_playtime_seconds = MAX(imported_playtime_seconds, ?2) WHERE id = ?1",
            params![id, seconds],
        )?;
        Ok(())
    }

    pub fn start_session(&self, game_id: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO sessions (game_id, started_at) VALUES (?1, ?2)",
            params![game_id, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn finish_session(&self, session_id: i64, duration_seconds: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?2, duration_seconds = ?3 WHERE id = ?1",
            params![session_id, now, duration_seconds],
        )?;
        self.conn.execute(
            r#"UPDATE games
               SET playtime_seconds = playtime_seconds + ?2, last_played_at = ?3
               WHERE id = (SELECT game_id FROM sessions WHERE id = ?1)"#,
            params![session_id, duration_seconds, now],
        )?;
        Ok(())
    }

    pub fn sessions_for_game(&self, game_id: &str, limit: i64) -> Result<Vec<PlaySession>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, game_id, started_at, ended_at, duration_seconds FROM sessions
             WHERE game_id = ?1 ORDER BY started_at DESC LIMIT ?2",
        )?;
        let sessions = stmt
            .query_map(params![game_id, limit], |row| {
                Ok(PlaySession {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    duration_seconds: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sessions)
    }

    pub fn replace_achievements(&self, game_id: &str, achievements: &[Achievement]) -> Result<()> {
        for achievement in achievements {
            self.conn.execute(
                r#"INSERT INTO achievements (game_id, api_name, name, description, icon_url, unlocked, unlocked_at, rarity)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                   ON CONFLICT(game_id, api_name) DO UPDATE SET
                       name = excluded.name,
                       description = excluded.description,
                       icon_url = excluded.icon_url,
                       unlocked = excluded.unlocked,
                       unlocked_at = excluded.unlocked_at,
                       rarity = excluded.rarity"#,
                params![
                    game_id,
                    achievement.api_name,
                    achievement.name,
                    achievement.description,
                    achievement.icon_url,
                    achievement.unlocked as i32,
                    achievement.unlocked_at,
                    achievement.rarity,
                ],
            )?;
        }
        Ok(())
    }

    pub fn achievements_for_game(&self, game_id: &str) -> Result<Vec<Achievement>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, game_id, api_name, name, description, icon_url, unlocked, unlocked_at, rarity
             FROM achievements WHERE game_id = ?1 ORDER BY unlocked DESC, rarity ASC",
        )?;
        let achievements = stmt
            .query_map(params![game_id], |row| {
                Ok(Achievement {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    api_name: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    icon_url: row.get(5)?,
                    unlocked: row.get::<_, i64>(6)? != 0,
                    unlocked_at: row.get(7)?,
                    rarity: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(achievements)
    }

    /// Buckets unlocked achievements into PS-style trophy tiers by global rarity.
    pub fn trophy_summary(&self, game_id: Option<&str>) -> Result<TrophySummary> {
        let achievements: Vec<Achievement> = match game_id {
            Some(id) => self.achievements_for_game(id)?,
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, game_id, api_name, name, description, icon_url, unlocked, unlocked_at, rarity FROM achievements",
                )?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(Achievement {
                            id: row.get(0)?,
                            game_id: row.get(1)?,
                            api_name: row.get(2)?,
                            name: row.get(3)?,
                            description: row.get(4)?,
                            icon_url: row.get(5)?,
                            unlocked: row.get::<_, i64>(6)? != 0,
                            unlocked_at: row.get(7)?,
                            rarity: row.get(8)?,
                        })
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                rows
            }
        };

        let mut summary = TrophySummary {
            total: achievements.len() as i64,
            ..TrophySummary::default()
        };
        for achievement in achievements.iter().filter(|a| a.unlocked) {
            summary.unlocked += 1;
            match achievement.rarity.unwrap_or(50.0) {
                rarity if rarity < 5.0 => summary.gold += 1,
                rarity if rarity < 20.0 => summary.silver += 1,
                _ => summary.bronze += 1,
            }
        }
        if summary.total > 0 && summary.unlocked == summary.total {
            summary.platinum = 1;
        }
        summary.progress = if summary.total > 0 {
            (summary.unlocked as f64 / summary.total as f64) * 100.0
        } else {
            0.0
        };
        Ok(summary)
    }

    pub fn library_stats(&self) -> Result<LibraryStats> {
        let mut stats = LibraryStats::default();
        self.conn.query_row(
            r#"SELECT COUNT(*),
                      COALESCE(SUM(installed), 0),
                      COALESCE(SUM(playtime_seconds + imported_playtime_seconds), 0),
                      COUNT(CASE WHEN playtime_seconds + imported_playtime_seconds > 0 THEN 1 END)
               FROM games WHERE hidden = 0"#,
            [],
            |row| {
                stats.total_games = row.get(0)?;
                stats.installed_games = row.get(1)?;
                stats.total_playtime_seconds = row.get(2)?;
                let played: i64 = row.get(3)?;
                stats.average_playtime_seconds = if played > 0 {
                    stats.total_playtime_seconds / played
                } else {
                    0
                };
                Ok(())
            },
        )?;

        self.conn.query_row(
            "SELECT COALESCE(AVG(duration_seconds), 0) FROM sessions WHERE ended_at IS NOT NULL",
            [],
            |row| {
                stats.average_session_seconds = row.get::<_, f64>(0)? as i64;
                Ok(())
            },
        )?;

        self.conn.query_row(
            r#"SELECT COUNT(*), COALESCE(SUM(duration_seconds), 0) FROM sessions
               WHERE started_at >= datetime('now', '-7 days')"#,
            [],
            |row| {
                stats.sessions_last_7_days = row.get(0)?;
                stats.playtime_last_7_days_seconds = row.get(1)?;
                Ok(())
            },
        )?;

        stats.most_played_game = self
            .conn
            .query_row(
                r#"SELECT name FROM games WHERE hidden = 0
                   ORDER BY playtime_seconds + imported_playtime_seconds DESC LIMIT 1"#,
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(stats)
    }

    pub fn load_settings(&self) -> Result<Settings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let mut settings = Settings::default();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "steam_api_key" => settings.steam_api_key = Some(value),
                "steam_id64" => settings.steam_id64 = Some(value),
                "steamgriddb_api_key" => settings.steamgriddb_api_key = Some(value),
                "umu_run_path" => settings.umu_run_path = value,
                "default_proton_version" => settings.default_proton_version = value,
                "prefix_root" => settings.prefix_root = Some(value),
                "extra_library_dirs" => settings.extra_library_dirs = split_list(Some(value)),
                "close_to_tray" => settings.close_to_tray = value == "1",
                "autoplay_trailers" => settings.autoplay_trailers = value == "1",
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let pairs: Vec<(&str, String)> = vec![
            (
                "steam_api_key",
                settings.steam_api_key.clone().unwrap_or_default(),
            ),
            (
                "steam_id64",
                settings.steam_id64.clone().unwrap_or_default(),
            ),
            (
                "steamgriddb_api_key",
                settings.steamgriddb_api_key.clone().unwrap_or_default(),
            ),
            ("umu_run_path", settings.umu_run_path.clone()),
            (
                "default_proton_version",
                settings.default_proton_version.clone(),
            ),
            (
                "prefix_root",
                settings.prefix_root.clone().unwrap_or_default(),
            ),
            (
                "extra_library_dirs",
                join_list(&settings.extra_library_dirs),
            ),
            (
                "close_to_tray",
                if settings.close_to_tray { "1" } else { "0" }.to_string(),
            ),
            (
                "autoplay_trailers",
                if settings.autoplay_trailers { "1" } else { "0" }.to_string(),
            ),
        ];
        for (key, value) in pairs {
            self.conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
        }
        Ok(())
    }
}

pub fn default_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ember")
        .join("library.db")
}

pub fn artwork_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ember")
        .join("artwork")
}
