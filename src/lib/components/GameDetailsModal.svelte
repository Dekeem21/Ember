<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type Achievement, type Game, type PlaySession } from "$lib/api";
  import { library } from "$lib/library.svelte";
  import { formatDuration, formatHours, formatRelative, runnerLabel, sourceLabel } from "$lib/format";

  let { game, onClose }: { game: Game; onClose: () => void } = $props();

  let tab = $state<"overview" | "compatibility" | "sessions" | "achievements">("overview");
  let sessions = $state<PlaySession[]>([]);
  let achievements = $state<Achievement[]>([]);
  let protonVersions = $state<string[]>([]);

  /// The form is an editable copy; the modal is remounted per game.
  function buildDraft(source: Game) {
    return {
      name: source.name,
      executable: source.executable ?? "",
      launchArgs: source.launchArgs ?? "",
      runner: source.runner,
      protonVersion: source.protonVersion ?? "",
      prefixPath: source.prefixPath ?? "",
      envVars: source.envVars ?? "",
      trailerUrl: source.trailerUrl ?? "",
      communityHours: source.communityPlaytimeSeconds ? source.communityPlaytimeSeconds / 3600 : 0
    };
  }

  let draft = $state(buildDraft(game));

  $effect(() => {
    api.gameSessions(game.id).then((value) => (sessions = value));
    api.gameAchievements(game.id).then((value) => (achievements = value));
    api.protonVersions().then((value) => (protonVersions = value));
  });

  async function pickExecutable() {
    const selected = await open({ multiple: false });
    if (typeof selected === "string") draft.executable = selected;
  }

  async function save() {
    await api.updateGame(game.id, {
      name: draft.name,
      executable: draft.executable || null,
      launch_args: draft.launchArgs || null,
      runner: draft.runner,
      proton_version: draft.protonVersion || null,
      prefix_path: draft.prefixPath || null,
      env_vars: draft.envVars || null,
      trailer_url: draft.trailerUrl || null,
      community_playtime_seconds: draft.communityHours ? Math.round(draft.communityHours * 3600) : null
    });
    await library.reload();
    onClose();
  }
</script>

<div class="backdrop">
  <div class="modal">
    <header>
      <div>
        <h2>{game.name}</h2>
        <p class="sub">
          {sourceLabel[game.source]} · {runnerLabel[game.runner]} · {formatHours(game.playtimeSeconds)} played
          · {formatRelative(game.lastPlayedAt)}
        </p>
      </div>
      <button class="close" onclick={onClose} aria-label="Close">✕</button>
    </header>

    <nav>
      {#each ["overview", "compatibility", "sessions", "achievements"] as const as key (key)}
        <button class:active={tab === key} onclick={() => (tab = key)}>{key}</button>
      {/each}
    </nav>

    <div class="body">
      {#if tab === "overview"}
        <label><span>Title</span><input bind:value={draft.name} /></label>
        <label>
          <span>Executable</span>
          <div class="inline">
            <input bind:value={draft.executable} placeholder="/path/to/game.exe" />
            <button class="secondary" onclick={pickExecutable}>Browse…</button>
          </div>
        </label>
        <label><span>Launch arguments</span><input bind:value={draft.launchArgs} /></label>
        <label><span>Trailer URL</span><input bind:value={draft.trailerUrl} /></label>
        <label>
          <span>Community playtime (h)</span>
          <input type="number" min="0" step="0.5" bind:value={draft.communityHours} />
        </label>
        <p class="meta">
          {game.developer ?? "Unknown developer"} · {game.releaseDate ?? "Unknown release"}
          {#if game.genres.length}· {game.genres.join(", ")}{/if}
        </p>
        <p class="desc">{game.description ?? ""}</p>
        <div class="row">
          <button class="secondary" onclick={() => library.refreshSelected()}>Refresh metadata</button>
          <button class="secondary" onclick={() => library.toggleFavorite(game)}>
            {game.favorite ? "Unfavourite" : "Favourite"}
          </button>
          <button
            class="danger"
            onclick={async () => {
              await api.deleteGame(game.id);
              await library.reload();
              onClose();
            }}>Remove from library</button
          >
        </div>
      {:else if tab === "compatibility"}
        <label>
          <span>Runner</span>
          <select bind:value={draft.runner}>
            <option value="native">Native Linux</option>
            <option value="umu">Proton via umu-launcher</option>
            <option value="steam">Steam client</option>
            <option value="heroic">Heroic client</option>
            <option value="lutris">Lutris client</option>
          </select>
        </label>
        <label>
          <span>Proton build</span>
          <select bind:value={draft.protonVersion}>
            <option value="">Use default ({library.settings?.defaultProtonVersion})</option>
            {#each protonVersions as version (version)}
              <option value={version}>{version}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Wine prefix</span>
          <input bind:value={draft.prefixPath} placeholder="Defaults to a per-game prefix" />
        </label>
        <label class="tall">
          <span>Environment variables</span>
          <textarea rows="6" bind:value={draft.envVars} placeholder={"MANGOHUD=1\nDXVK_HUD=fps"}
          ></textarea>
        </label>
        <p class="hint">
          Windows games run as <code>umu-run &lt;exe&gt;</code> with <code>PROTONPATH</code>,
          <code>GAMEID</code> and <code>WINEPREFIX</code> set from these values.
        </p>
      {:else if tab === "sessions"}
        <ul class="sessions">
          {#each sessions as session (session.id)}
            <li>
              <span>{new Date(session.startedAt).toLocaleString()}</span>
              <span class="duration">{formatDuration(session.durationSeconds)}</span>
            </li>
          {:else}
            <li class="empty">No sessions recorded yet.</li>
          {/each}
        </ul>
        <p class="hint">
          Average session {formatDuration(game.averageSessionSeconds)} across {game.sessionCount} sessions.
        </p>
      {:else}
        <div class="row">
          <button class="secondary" onclick={() => api.syncAchievements(game.id)}>Sync from Steam</button>
        </div>
        <ul class="achievements">
          {#each achievements as achievement (achievement.id)}
            <li class:unlocked={achievement.unlocked}>
              {#if achievement.iconUrl}
                <img src={achievement.iconUrl} alt="" />
              {/if}
              <div>
                <p class="name">{achievement.name}</p>
                <p class="desc small">{achievement.description ?? ""}</p>
              </div>
              <span class="rarity">
                {achievement.rarity ? `${achievement.rarity.toFixed(1)}%` : ""}
              </span>
            </li>
          {:else}
            <li class="empty">No achievements imported yet.</li>
          {/each}
        </ul>
      {/if}
    </div>

    <footer>
      <button class="secondary" onclick={onClose}>Cancel</button>
      <button class="primary" onclick={save}>Save</button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: grid;
    place-items: center;
    background: rgba(4, 3, 4, 0.7);
    backdrop-filter: blur(8px);
  }

  .modal {
    width: min(820px, 94vw);
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    border-radius: 20px;
    background: rgba(18, 12, 14, 0.98);
    border: 1px solid var(--panel-border);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 20px 24px 12px;
  }

  h2 {
    margin: 0;
    font-size: 22px;
  }

  .sub {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--text-dim);
  }

  nav {
    display: flex;
    gap: 6px;
    padding: 0 24px;
    border-bottom: 1px solid var(--panel-border);
  }

  nav button {
    padding: 10px 14px;
    font-size: 13px;
    text-transform: capitalize;
    color: var(--text-dim);
    border-bottom: 2px solid transparent;
  }

  nav button.active {
    color: #fff;
    border-color: var(--ember);
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
  }

  label {
    display: grid;
    grid-template-columns: 180px 1fr;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    font-size: 14px;
  }

  label.tall {
    align-items: start;
  }

  .inline {
    display: flex;
    gap: 8px;
  }

  .inline input {
    flex: 1;
  }

  textarea {
    width: 100%;
    resize: vertical;
    font-family: "JetBrains Mono", monospace;
    font-size: 13px;
  }

  .row {
    display: flex;
    gap: 10px;
    margin: 14px 0;
  }

  .meta,
  .hint {
    font-size: 12px;
    color: var(--text-dim);
  }

  .desc {
    font-size: 14px;
    line-height: 1.5;
  }

  .desc.small {
    font-size: 12px;
    color: var(--text-dim);
    margin: 2px 0 0;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 8px;
  }

  .sessions li,
  .achievements li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    font-size: 13px;
  }

  .sessions li {
    justify-content: space-between;
  }

  .achievements li {
    opacity: 0.5;
  }

  .achievements li.unlocked {
    opacity: 1;
    border-left: 3px solid var(--ember);
  }

  .achievements img {
    width: 38px;
    height: 38px;
    border-radius: 8px;
  }

  .achievements .name {
    margin: 0;
    font-weight: 600;
  }

  .achievements div {
    flex: 1;
  }

  .rarity {
    color: var(--text-dim);
  }

  .duration {
    font-weight: 600;
  }

  .empty {
    justify-content: center;
    color: var(--text-dim);
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 16px 24px;
    border-top: 1px solid var(--panel-border);
  }

  .primary,
  .secondary,
  .danger {
    padding: 10px 20px;
    border-radius: 999px;
    font-size: 14px;
  }

  .primary {
    background: var(--ember);
  }

  .secondary {
    background: rgba(255, 255, 255, 0.1);
  }

  .danger {
    background: rgba(224, 27, 36, 0.18);
    color: #ff9e9e;
  }

  .close {
    font-size: 16px;
    color: var(--text-dim);
  }
</style>
