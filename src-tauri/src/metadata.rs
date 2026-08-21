use std::path::PathBuf;

use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};
use serde::Deserialize;
use serde_json::Value;

use crate::db::artwork_dir;
use crate::models::{Achievement, Game, Settings, Source};
use crate::scan::steam as steam_scan;

const USER_AGENT: &str = "ember-launcher/0.1";

fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(20))
        .build()?)
}

#[derive(Debug, Default, Clone)]
pub struct FetchedMetadata {
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub trailer_url: Option<String>,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
    pub logo_path: Option<String>,
}

/// Resolves the Steam appid for a game, falling back to a store search by name.
pub async fn resolve_steam_appid(game: &Game) -> Result<Option<String>> {
    if game.source == Source::Steam {
        return Ok(game.external_id.clone());
    }
    let url = format!(
        "https://steamcommunity.com/actions/SearchApps/{}",
        urlencoding::encode(&game.name)
    );
    let response = client()?.get(url).send().await?;
    let results: Vec<Value> = response.json().await.unwrap_or_default();
    Ok(results
        .first()
        .and_then(|entry| entry.get("appid"))
        .and_then(Value::as_str)
        .map(|appid| appid.to_string()))
}

/// Pulls description, credits, genres, trailer and artwork for a game.
pub async fn fetch_for_game(game: &Game, settings: &Settings) -> Result<FetchedMetadata> {
    let mut metadata = FetchedMetadata::default();
    let appid = resolve_steam_appid(game).await.unwrap_or(None);

    if let Some(appid) = appid.as_deref() {
        if let Ok(details) = fetch_steam_details(appid).await {
            metadata.description = details.description;
            metadata.developer = details.developer;
            metadata.publisher = details.publisher;
            metadata.release_date = details.release_date;
            metadata.genres = details.genres;
            metadata.trailer_url = details.trailer_url;
        }

        // Steam already caches artwork locally for installed games.
        if let Some((cover, hero, logo)) = steam_scan::local_artwork(appid) {
            metadata.cover_path = cover.map(path_string);
            metadata.hero_path = hero.map(path_string);
            metadata.logo_path = logo.map(path_string);
        }

        let cdn = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}");
        if metadata.cover_path.is_none() {
            metadata.cover_path =
                download(&format!("{cdn}/library_600x900_2x.jpg"), &game.id, "cover").await;
        }
        if metadata.hero_path.is_none() {
            metadata.hero_path =
                download(&format!("{cdn}/library_hero.jpg"), &game.id, "hero").await;
        }
        if metadata.logo_path.is_none() {
            metadata.logo_path = download(&format!("{cdn}/logo.png"), &game.id, "logo").await;
        }
    }

    if metadata.cover_path.is_none() || metadata.hero_path.is_none() {
        if let Some(key) = settings.steamgriddb_api_key.as_deref() {
            if let Ok(grid) = fetch_steamgriddb(&game.name, key, &game.id).await {
                metadata.cover_path = metadata.cover_path.or(grid.cover_path);
                metadata.hero_path = metadata.hero_path.or(grid.hero_path);
                metadata.logo_path = metadata.logo_path.or(grid.logo_path);
            }
        }
    }

    Ok(metadata)
}

fn path_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

async fn fetch_steam_details(appid: &str) -> Result<FetchedMetadata> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={appid}&l=english");
    let response: Value = client()?.get(url).send().await?.json().await?;
    let data = response
        .get(appid)
        .and_then(|entry| entry.get("data"))
        .ok_or_else(|| anyhow!("no store data for {appid}"))?;

    let list = |key: &str| -> Option<String> {
        data.get(key)
            .and_then(Value::as_array)
            .map(|array| {
                array
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|value| !value.is_empty())
    };

    let trailer_url = data
        .get("movies")
        .and_then(Value::as_array)
        .and_then(|movies| movies.first())
        .and_then(|movie| {
            movie
                .get("mp4")
                .and_then(|mp4| mp4.get("max").or_else(|| mp4.get("480")))
                .and_then(Value::as_str)
        })
        .map(|url| url.replace("http://", "https://"));

    Ok(FetchedMetadata {
        description: data
            .get("short_description")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        developer: list("developers"),
        publisher: list("publishers"),
        release_date: data
            .get("release_date")
            .and_then(|release| release.get("date"))
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        genres: data
            .get("genres")
            .and_then(Value::as_array)
            .map(|genres| {
                genres
                    .iter()
                    .filter_map(|genre| genre.get("description").and_then(Value::as_str))
                    .map(|value| value.to_string())
                    .collect()
            })
            .unwrap_or_default(),
        trailer_url,
        ..Default::default()
    })
}

async fn fetch_steamgriddb(name: &str, api_key: &str, game_id: &str) -> Result<FetchedMetadata> {
    let http = client()?;
    let search: Value = http
        .get(format!(
            "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
            urlencoding::encode(name)
        ))
        .bearer_auth(api_key)
        .send()
        .await?
        .json()
        .await?;
    let sgdb_id = search
        .get("data")
        .and_then(Value::as_array)
        .and_then(|array| array.first())
        .and_then(|entry| entry.get("id"))
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow!("no SteamGridDB match for {name}"))?;

    let first_url = |json: &Value| -> Option<String> {
        json.get("data")
            .and_then(Value::as_array)
            .and_then(|array| array.first())
            .and_then(|entry| entry.get("url"))
            .and_then(Value::as_str)
            .map(|url| url.to_string())
    };

    let mut metadata = FetchedMetadata::default();
    for (endpoint, kind) in [("grids", "cover"), ("heroes", "hero"), ("logos", "logo")] {
        let json: Value = http
            .get(format!(
                "https://www.steamgriddb.com/api/v2/{endpoint}/game/{sgdb_id}"
            ))
            .bearer_auth(api_key)
            .send()
            .await?
            .json()
            .await?;
        let Some(url) = first_url(&json) else {
            continue;
        };
        let saved = download(&url, game_id, kind).await;
        match kind {
            "cover" => metadata.cover_path = saved,
            "hero" => metadata.hero_path = saved,
            _ => metadata.logo_path = saved,
        }
    }
    Ok(metadata)
}

/// Downloads an image into the artwork cache, returning its local path.
pub async fn download(url: &str, game_id: &str, kind: &str) -> Option<String> {
    let extension = url.rsplit('.').next().unwrap_or("jpg");
    let extension = if extension.len() > 4 {
        "jpg"
    } else {
        extension
    };
    let dir = artwork_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{game_id}-{kind}.{extension}"));

    let response = client().ok()?.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() < 1024 {
        return None;
    }
    std::fs::write(&path, &bytes).ok()?;
    Some(path.to_string_lossy().to_string())
}

#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGames,
}

#[derive(Debug, Deserialize)]
struct OwnedGames {
    #[serde(default)]
    games: Vec<OwnedGame>,
}

#[derive(Debug, Deserialize)]
struct OwnedGame {
    appid: i64,
    #[serde(default)]
    playtime_forever: i64,
}

/// Imports `playtime_forever` from the Steam Web API, in seconds keyed by appid.
pub async fn fetch_steam_playtime(settings: &Settings) -> Result<Vec<(String, i64)>> {
    let (Some(key), Some(steam_id)) = (
        settings.steam_api_key.as_deref(),
        settings.steam_id64.as_deref(),
    ) else {
        return Ok(Vec::new());
    };
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={key}&steamid={steam_id}&include_appinfo=false&include_played_free_games=true"
    );
    let response: OwnedGamesResponse = client()?.get(url).send().await?.json().await?;
    Ok(response
        .response
        .games
        .into_iter()
        .map(|game| (game.appid.to_string(), game.playtime_forever * 60))
        .collect())
}

/// Fetches achievements plus their global rarity for a Steam game.
pub async fn fetch_achievements(
    game: &Game,
    appid: &str,
    settings: &Settings,
) -> Result<Vec<Achievement>> {
    let (Some(key), Some(steam_id)) = (
        settings.steam_api_key.as_deref(),
        settings.steam_id64.as_deref(),
    ) else {
        return Ok(Vec::new());
    };
    let http = client()?;

    let schema: Value = http
        .get(format!(
            "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key={key}&appid={appid}"
        ))
        .send()
        .await?
        .json()
        .await?;
    let schema_achievements = schema
        .pointer("/game/availableGameStats/achievements")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let player: Value = http
        .get(format!(
            "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?key={key}&steamid={steam_id}&appid={appid}"
        ))
        .send()
        .await?
        .json()
        .await?;
    let player_achievements = player
        .pointer("/playerstats/achievements")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let global: Value = http
        .get(format!(
            "https://api.steampowered.com/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v2/?gameid={appid}"
        ))
        .send()
        .await?
        .json()
        .await?;
    let global_percentages = global
        .pointer("/achievementpercentages/achievements")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let rarity_for = |api_name: &str| -> Option<f64> {
        global_percentages
            .iter()
            .find(|entry| entry.get("name").and_then(Value::as_str) == Some(api_name))
            .and_then(|entry| entry.get("percent"))
            .and_then(|percent| percent.as_f64().or_else(|| percent.as_str()?.parse().ok()))
    };

    let mut achievements = Vec::new();
    for entry in schema_achievements {
        let Some(api_name) = entry.get("name").and_then(Value::as_str) else {
            continue;
        };
        let player_entry = player_achievements
            .iter()
            .find(|player| player.get("apiname").and_then(Value::as_str) == Some(api_name));
        let unlocked = player_entry
            .and_then(|player| player.get("achieved"))
            .and_then(Value::as_i64)
            .unwrap_or(0)
            != 0;
        let unlocked_at = player_entry
            .and_then(|player| player.get("unlocktime"))
            .and_then(Value::as_i64)
            .filter(|time| *time > 0)
            .and_then(|time| Utc.timestamp_opt(time, 0).single())
            .map(|time| time.to_rfc3339());

        achievements.push(Achievement {
            id: 0,
            game_id: game.id.clone(),
            api_name: api_name.to_string(),
            name: entry
                .get("displayName")
                .and_then(Value::as_str)
                .unwrap_or(api_name)
                .to_string(),
            description: entry
                .get("description")
                .and_then(Value::as_str)
                .map(|value| value.to_string()),
            icon_url: entry
                .get("icon")
                .and_then(Value::as_str)
                .map(|value| value.to_string()),
            unlocked,
            unlocked_at,
            rarity: rarity_for(api_name),
        });
    }
    Ok(achievements)
}
