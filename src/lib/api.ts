import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

export type Source = "steam" | "heroic" | "lutris" | "manual";
export type Runner = "native" | "umu" | "steam" | "heroic" | "lutris";

export interface Game {
  id: string;
  name: string;
  source: Source;
  runner: Runner;
  externalId: string | null;
  installDir: string | null;
  executable: string | null;
  launchArgs: string | null;
  protonVersion: string | null;
  prefixPath: string | null;
  envVars: string | null;
  installed: boolean;
  hidden: boolean;
  favorite: boolean;
  description: string | null;
  developer: string | null;
  publisher: string | null;
  releaseDate: string | null;
  genres: string[];
  tags: string[];
  coverPath: string | null;
  heroPath: string | null;
  logoPath: string | null;
  iconPath: string | null;
  trailerUrl: string | null;
  playtimeSeconds: number;
  sessionCount: number;
  averageSessionSeconds: number;
  communityPlaytimeSeconds: number | null;
  lastPlayedAt: string | null;
  addedAt: string;
}

export interface PlaySession {
  id: number;
  gameId: string;
  startedAt: string;
  endedAt: string | null;
  durationSeconds: number;
}

export interface Achievement {
  id: number;
  gameId: string;
  apiName: string;
  name: string;
  description: string | null;
  iconUrl: string | null;
  unlocked: boolean;
  unlockedAt: string | null;
  rarity: number | null;
}

export interface TrophySummary {
  platinum: number;
  gold: number;
  silver: number;
  bronze: number;
  unlocked: number;
  total: number;
  progress: number;
}

export interface LibraryStats {
  totalGames: number;
  installedGames: number;
  totalPlaytimeSeconds: number;
  averagePlaytimeSeconds: number;
  averageSessionSeconds: number;
  sessionsLast7Days: number;
  playtimeLast7DaysSeconds: number;
  mostPlayedGame: string | null;
}

export interface Settings {
  steamApiKey: string | null;
  steamId64: string | null;
  steamgriddbApiKey: string | null;
  umuRunPath: string;
  defaultProtonVersion: string;
  prefixRoot: string | null;
  extraLibraryDirs: string[];
  closeToTray: boolean;
  autoplayTrailers: boolean;
}

export interface ScanReport {
  added: number;
  updated: number;
  sources: { source: Source; found: number }[];
  errors: string[];
}

export interface UmuStatus {
  available: boolean;
  version: string;
  path: string;
}

export const api = {
  listGames: (includeHidden = false) => invoke<Game[]>("list_games", { includeHidden }),
  getGame: (gameId: string) => invoke<Game | null>("get_game", { gameId }),
  libraryStats: () => invoke<LibraryStats>("library_stats"),
  runningGames: () => invoke<string[]>("running_games"),
  scanLibrary: () => invoke<ScanReport>("scan_library"),
  launchGame: (gameId: string) => invoke<void>("launch_game", { gameId }),
  stopGame: (gameId: string) => invoke<void>("stop_game", { gameId }),
  updateGame: (gameId: string, patch: Record<string, unknown>) =>
    invoke<Game | null>("update_game", { gameId, patch }),
  setGameFlag: (gameId: string, flag: "favorite" | "hidden" | "installed", value: boolean) =>
    invoke<void>("set_game_flag", { gameId, flag, value }),
  deleteGame: (gameId: string) => invoke<void>("delete_game", { gameId }),
  addManualGame: (name: string, executable: string, runner: Runner) =>
    invoke<string>("add_manual_game", { name, executable, runner }),
  gameSessions: (gameId: string, limit = 20) =>
    invoke<PlaySession[]>("game_sessions", { gameId, limit }),
  gameAchievements: (gameId: string) => invoke<Achievement[]>("game_achievements", { gameId }),
  trophySummary: (gameId?: string) => invoke<TrophySummary>("trophy_summary", { gameId }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  protonVersions: () => invoke<string[]>("proton_versions"),
  envPresets: () => invoke<Record<string, string>>("env_presets"),
  umuStatus: () => invoke<UmuStatus>("umu_status"),
  refreshMetadata: (gameId: string) => invoke<Game | null>("refresh_metadata", { gameId }),
  refreshMissingMetadata: () => invoke<number>("refresh_missing_metadata"),
  syncSteamPlaytime: () => invoke<number>("sync_steam_playtime"),
  syncAchievements: (gameId: string) => invoke<TrophySummary>("sync_achievements", { gameId })
};

/// Local artwork lives outside the bundle, so it needs the asset protocol.
export function assetUrl(path: string | null | undefined): string | null {
  if (!path) return null;
  if (path.startsWith("http://") || path.startsWith("https://")) return path;
  return convertFileSrc(path);
}
