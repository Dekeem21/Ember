use serde::{Deserialize, Serialize};

/// Where a game came from. Determines how it is launched and re-scanned.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Steam,
    Heroic,
    Lutris,
    #[default]
    Manual,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Steam => "steam",
            Source::Heroic => "heroic",
            Source::Lutris => "lutris",
            Source::Manual => "manual",
        }
    }

    pub fn from_str(value: &str) -> Source {
        match value {
            "steam" => Source::Steam,
            "heroic" => Source::Heroic,
            "lutris" => Source::Lutris,
            _ => Source::Manual,
        }
    }
}

/// How the game binary is executed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    /// Native Linux binary or shell script.
    #[default]
    Native,
    /// Windows binary executed through umu-launcher (Proton).
    Umu,
    /// Handed over to the Steam client via `steam://rungameid`.
    Steam,
    /// Handed over to the Heroic client via `heroic://launch`.
    Heroic,
    /// Handed over to the Lutris client via `lutris:rungameid`.
    Lutris,
}

impl Runner {
    pub fn as_str(self) -> &'static str {
        match self {
            Runner::Native => "native",
            Runner::Umu => "umu",
            Runner::Steam => "steam",
            Runner::Heroic => "heroic",
            Runner::Lutris => "lutris",
        }
    }

    pub fn from_str(value: &str) -> Runner {
        match value {
            "umu" => Runner::Umu,
            "steam" => Runner::Steam,
            "heroic" => Runner::Heroic,
            "lutris" => Runner::Lutris,
            _ => Runner::Native,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    pub source: Source,
    pub runner: Runner,
    /// Store identifier within the source (Steam appid, Heroic app_name, Lutris slug...).
    pub external_id: Option<String>,
    pub install_dir: Option<String>,
    pub executable: Option<String>,
    pub launch_args: Option<String>,
    /// `GE-Proton9-x`, `UMU-Latest`, ... passed to umu-launcher as `PROTONPATH`.
    pub proton_version: Option<String>,
    pub prefix_path: Option<String>,
    /// Extra `KEY=value` environment pairs, one per line.
    pub env_vars: Option<String>,
    pub installed: bool,
    pub hidden: bool,
    pub favorite: bool,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
    pub logo_path: Option<String>,
    pub icon_path: Option<String>,
    pub trailer_url: Option<String>,
    /// Seconds accumulated by Ember plus imported playtime.
    pub playtime_seconds: i64,
    pub session_count: i64,
    /// Average length of a play session in seconds.
    pub average_session_seconds: i64,
    /// Community "main story" length in seconds, when known.
    pub community_playtime_seconds: Option<i64>,
    pub last_played_at: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedGame {
    pub name: String,
    pub source: Source,
    pub runner: Runner,
    pub external_id: Option<String>,
    pub install_dir: Option<String>,
    pub executable: Option<String>,
    pub installed: bool,
    pub playtime_seconds: i64,
    pub prefix_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaySession {
    pub id: i64,
    pub game_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub game_id: String,
    pub api_name: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
    /// Global unlock percentage, used to bucket achievements into trophy tiers.
    pub rarity: Option<f64>,
}

/// Playnite-style trophy roll-up derived from achievement rarity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophySummary {
    pub platinum: i64,
    pub gold: i64,
    pub silver: i64,
    pub bronze: i64,
    pub unlocked: i64,
    pub total: i64,
    pub progress: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStats {
    pub total_games: i64,
    pub installed_games: i64,
    pub total_playtime_seconds: i64,
    /// Mean playtime across games that have ever been played.
    pub average_playtime_seconds: i64,
    pub average_session_seconds: i64,
    pub sessions_last_7_days: i64,
    pub playtime_last_7_days_seconds: i64,
    pub most_played_game: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub steam_api_key: Option<String>,
    pub steam_id64: Option<String>,
    pub steamgriddb_api_key: Option<String>,
    pub umu_run_path: String,
    pub default_proton_version: String,
    pub prefix_root: Option<String>,
    pub extra_library_dirs: Vec<String>,
    pub close_to_tray: bool,
    pub autoplay_trailers: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            steam_api_key: None,
            steam_id64: None,
            steamgriddb_api_key: None,
            umu_run_path: "umu-run".to_string(),
            default_proton_version: "UMU-Latest".to_string(),
            prefix_root: None,
            extra_library_dirs: Vec::new(),
            close_to_tray: false,
            autoplay_trailers: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub added: usize,
    pub updated: usize,
    pub sources: Vec<SourceReport>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReport {
    pub source: Source,
    pub found: usize,
}
