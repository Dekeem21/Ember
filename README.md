# Ember

A Playnite-style game launcher for Linux (built and tuned on CachyOS/Arch), written with Tauri v2 + Svelte 5.
Windows games run through [umu-launcher](https://github.com/Open-Wine-Components/umu-launcher) with Proton.

## Features

- **Library scanning** — Steam (`libraryfolders.vdf` + `appmanifest_*.acf`), Heroic (Epic/GOG/Amazon), Lutris (`pga.db`) and arbitrary folders of native binaries or `.exe` files.
- **Launching** — native binaries, `umu-run` with a per-game `PROTONPATH`/`WINEPREFIX`/`GAMEID`, or handoff to the Steam/Heroic/Lutris clients.
- **Playtime** — every launch is tracked as a session: total playtime, session count, average session length, average playtime across the library, last played, plus imported Steam lifetime playtime.
- **Artwork** — grid/cover, hero and logo art pulled from local Steam cache, the Steam CDN, or SteamGridDB, cached under the Ember data dir.
- **Metadata & trailers** — description, developer, publisher, release date and genres from the Steam store API; trailers play in-app.
- **Achievements** — Steam achievement schema + player unlocks + global rarity, summarised as a platinum/gold/silver/bronze trophy card.
- **Per-game compatibility** — Proton build picker (discovers `compatibilitytools.d` and `steamapps/common`), prefix path, launch args and custom environment variables.

## Requirements

- Rust stable, Node 20+
- WebKitGTK 4.1 + GTK3 development packages (`webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `librsvg`)
- `umu-launcher` for Windows titles (`paru -S umu-launcher` on CachyOS)

## Development

```bash
npm install
npm run tauri dev      # dev app
npm run check          # svelte-check
cd src-tauri && cargo clippy --all-targets && cargo test
npm run tauri build    # deb / rpm / AppImage bundles
```

## Configuration

Settings live in the app (gear icon) and are stored in SQLite at `~/.local/share/ember/library.db`:

- `umu-run` path and default Proton build
- Wine prefix root (defaults to `~/.local/share/ember/prefixes`)
- Extra game folders to scan
- Steam Web API key + SteamID64 (playtime & achievements) and a SteamGridDB key (artwork fallback)

API keys are stored locally in the SQLite database and never leave the machine except in calls to the respective APIs.
