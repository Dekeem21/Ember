import { listen } from "@tauri-apps/api/event";
import { api, type Game, type LibraryStats, type Settings, type TrophySummary } from "./api";

interface GameEvent {
  gameId: string;
  name: string;
  durationSeconds: number;
}

/// Single source of truth for the dashboard, shared by every component.
class LibraryStore {
  games = $state<Game[]>([]);
  selectedId = $state<string | null>(null);
  stats = $state<LibraryStats | null>(null);
  settings = $state<Settings | null>(null);
  trophies = $state<TrophySummary | null>(null);
  running = $state<string[]>([]);
  query = $state("");
  busy = $state<string | null>(null);
  error = $state<string | null>(null);
  ready = $state(false);

  get filtered(): Game[] {
    const query = this.query.trim().toLowerCase();
    if (!query) return this.games;
    return this.games.filter(
      (game) =>
        game.name.toLowerCase().includes(query) ||
        game.genres.some((genre) => genre.toLowerCase().includes(query)) ||
        game.developer?.toLowerCase().includes(query)
    );
  }

  get selected(): Game | null {
    const games = this.filtered;
    return games.find((game) => game.id === this.selectedId) ?? games[0] ?? null;
  }

  get isSelectedRunning(): boolean {
    return this.selected ? this.running.includes(this.selected.id) : false;
  }

  async init() {
    await this.reload();
    this.settings = await api.getSettings();
    this.ready = true;

    await listen<GameEvent>("game-started", (event) => {
      if (!this.running.includes(event.payload.gameId)) {
        this.running = [...this.running, event.payload.gameId];
      }
    });
    await listen<GameEvent>("game-stopped", async (event) => {
      this.running = this.running.filter((id) => id !== event.payload.gameId);
      await this.reload();
    });
  }

  async reload() {
    try {
      const [games, stats, trophies, running] = await Promise.all([
        api.listGames(false),
        api.libraryStats(),
        api.trophySummary(),
        api.runningGames()
      ]);
      this.games = games;
      this.stats = stats;
      this.trophies = trophies;
      this.running = running;
      if (!this.selectedId && games.length > 0) this.selectedId = games[0].id;
    } catch (error) {
      this.error = String(error);
    }
  }

  select(gameId: string) {
    this.selectedId = gameId;
  }

  async withBusy<T>(label: string, action: () => Promise<T>): Promise<T | null> {
    this.busy = label;
    this.error = null;
    try {
      return await action();
    } catch (error) {
      this.error = String(error);
      return null;
    } finally {
      this.busy = null;
    }
  }

  async scan() {
    const report = await this.withBusy("Scanning library…", () => api.scanLibrary());
    await this.reload();
    return report;
  }

  async refreshArtwork() {
    await this.withBusy("Downloading artwork…", () => api.refreshMissingMetadata());
    await this.reload();
  }

  async refreshSelected() {
    const game = this.selected;
    if (!game) return;
    await this.withBusy("Refreshing metadata…", () => api.refreshMetadata(game.id));
    await this.reload();
  }

  async play() {
    const game = this.selected;
    if (!game) return;
    await this.withBusy("Launching…", () => api.launchGame(game.id));
  }

  async stop() {
    const game = this.selected;
    if (!game) return;
    await this.withBusy("Stopping…", () => api.stopGame(game.id));
  }

  async toggleFavorite(game: Game) {
    await api.setGameFlag(game.id, "favorite", !game.favorite);
    await this.reload();
  }

  async hide(game: Game) {
    await api.setGameFlag(game.id, "hidden", true);
    await this.reload();
  }

  async saveSettings(settings: Settings) {
    this.settings = await api.saveSettings(settings);
  }
}

export const library = new LibraryStore();
