<script lang="ts">
  import { library } from "$lib/library.svelte";
  import { formatRelative } from "$lib/format";

  const runningGame = $derived(library.games.find((game) => library.running.includes(game.id)));
  const lastPlayed = $derived(library.games.find((game) => game.lastPlayedAt));
</script>

<div class="pill">
  <span class="icon">
    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
      <path d="M9 18V5l10-2v13" />
      <circle cx="7" cy="18" r="2.6" />
      <circle cx="17" cy="16" r="2.6" />
    </svg>
  </span>
  <div class="text">
    <p class="label">{runningGame ? "Now Playing" : "Last Played"}</p>
    <p class="value">
      {runningGame?.name ?? lastPlayed?.name ?? "Nothing yet"}
      {#if !runningGame && lastPlayed}
        <span class="when">· {formatRelative(lastPlayed.lastPlayedAt)}</span>
      {/if}
    </p>
  </div>
  <button
    class="control"
    onclick={() => (runningGame ? library.stop() : library.play())}
    aria-label={runningGame ? "Stop" : "Play"}
  >
    {#if runningGame}
      <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M7 6h4v12H7zm6 0h4v12h-4z" /></svg>
    {:else}
      <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
    {/if}
  </button>
</div>

<style>
  .pill {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 380px;
    padding: 12px 14px;
    border-radius: 999px;
    background: var(--panel);
    border: 1px solid var(--panel-border);
    backdrop-filter: blur(18px);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    color: rgba(255, 255, 255, 0.8);
  }

  .text {
    flex: 1;
    min-width: 0;
  }

  .label {
    margin: 0;
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  .value {
    margin: 2px 0 0;
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .when {
    font-weight: 400;
    color: var(--text-dim);
  }

  .control {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.12);
  }

  .control:hover {
    background: var(--ember);
  }
</style>
