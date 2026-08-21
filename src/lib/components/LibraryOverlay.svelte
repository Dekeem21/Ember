<script lang="ts">
  import { assetUrl, type Game } from "$lib/api";
  import { library } from "$lib/library.svelte";
  import { formatHours, formatRelative, sourceLabel } from "$lib/format";

  let { onClose }: { onClose: () => void } = $props();

  type SortKey = "recent" | "name" | "playtime";
  let sort = $state<SortKey>("recent");
  let sourceFilter = $state<"all" | Game["source"]>("all");
  let installedOnly = $state(false);

  const games = $derived.by(() => {
    let games = [...library.filtered];
    if (sourceFilter !== "all") games = games.filter((game) => game.source === sourceFilter);
    if (installedOnly) games = games.filter((game) => game.installed);
    return games.sort((a, b) => {
      if (sort === "name") return a.name.localeCompare(b.name);
      if (sort === "playtime") return b.playtimeSeconds - a.playtimeSeconds;
      return (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? "");
    });
  });
</script>

<div class="overlay">
  <header>
    <h2>Library <span>{games.length} games</span></h2>
    <div class="controls">
      <input placeholder="Filter…" bind:value={library.query} />
      <select bind:value={sourceFilter}>
        <option value="all">All sources</option>
        <option value="steam">Steam</option>
        <option value="heroic">Heroic</option>
        <option value="lutris">Lutris</option>
        <option value="manual">Local</option>
      </select>
      <select bind:value={sort}>
        <option value="recent">Recently played</option>
        <option value="name">Name</option>
        <option value="playtime">Playtime</option>
      </select>
      <label class="check">
        <input type="checkbox" bind:checked={installedOnly} /> Installed only
      </label>
      <button class="close" onclick={onClose}>Close</button>
    </div>
  </header>

  <div class="grid">
    {#each games as game (game.id)}
      <button
        class="tile"
        class:selected={library.selected?.id === game.id}
        onclick={() => library.select(game.id)}
        ondblclick={() => {
          library.select(game.id);
          library.play();
        }}
      >
        <div class="art">
          {#if assetUrl(game.coverPath)}
            <img src={assetUrl(game.coverPath)} alt={game.name} />
          {:else}
            <span class="letter">{game.name[0]}</span>
          {/if}
          {#if !game.installed}<span class="tag">Not installed</span>{/if}
        </div>
        <p class="name">{game.name}</p>
        <p class="meta">
          {sourceLabel[game.source]} · {formatHours(game.playtimeSeconds)} · {formatRelative(
            game.lastPlayedAt
          )}
        </p>
      </button>
    {/each}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 20;
    padding: 26px 34px;
    background: rgba(6, 4, 5, 0.94);
    backdrop-filter: blur(14px);
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
  }

  h2 {
    margin: 0;
    font-size: 24px;
  }

  h2 span {
    margin-left: 10px;
    font-size: 13px;
    font-weight: 400;
    color: var(--text-dim);
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-dim);
  }

  .close {
    padding: 8px 18px;
    border-radius: 999px;
    background: var(--ember);
  }

  .grid {
    flex: 1;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
    gap: 18px;
    padding-bottom: 20px;
  }

  .tile {
    text-align: left;
  }

  .art {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: 12px;
    overflow: hidden;
    background: linear-gradient(140deg, #2a1a1d, #120c0e);
    border: 2px solid transparent;
    display: grid;
    place-items: center;
  }

  .tile.selected .art {
    border-color: var(--ember);
    box-shadow: 0 0 22px rgba(224, 27, 36, 0.4);
  }

  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .letter {
    font-size: 44px;
    font-weight: 800;
    color: rgba(255, 255, 255, 0.35);
  }

  .tag {
    position: absolute;
    left: 8px;
    bottom: 8px;
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 10px;
    background: rgba(0, 0, 0, 0.7);
    color: var(--text-dim);
  }

  .name {
    margin: 10px 0 2px;
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta {
    margin: 0;
    font-size: 11px;
    color: var(--text-dim);
  }
</style>
