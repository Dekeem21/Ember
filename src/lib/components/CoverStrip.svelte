<script lang="ts">
  import { assetUrl, type Game } from "$lib/api";
  import { library } from "$lib/library.svelte";
  import { sourceLabel } from "$lib/format";

  let { onOpenLibrary }: { onOpenLibrary: () => void } = $props();

  const visible = $derived(library.filtered.slice(0, 5));
  const selected = $derived(library.selected);

  function initials(game: Game) {
    return game.name
      .split(/\s+/)
      .slice(0, 2)
      .map((word) => word[0])
      .join("")
      .toUpperCase();
  }
</script>

<section class="strip">
  <div class="covers">
    {#each visible as game (game.id)}
      <button
        class="cover"
        class:active={selected?.id === game.id}
        onclick={() => library.select(game.id)}
        title={game.name}
      >
        {#if assetUrl(game.coverPath)}
          <img src={assetUrl(game.coverPath)} alt={game.name} />
        {:else}
          <span class="fallback">{initials(game)}</span>
        {/if}
        {#if library.running.includes(game.id)}
          <span class="running"></span>
        {/if}
      </button>
    {/each}

    <button class="cover grid" onclick={onOpenLibrary} title="All games" aria-label="All games">
      <span class="tile"></span>
      <span class="tile"></span>
      <span class="tile"></span>
      <span class="tile"></span>
    </button>
  </div>

  {#if selected}
    <div class="caption">
      <span class="badge">{sourceLabel[selected.source]}</span>
      <span class="name">{selected.name}</span>
      {#if selected.favorite}<span class="fav">★</span>{/if}
    </div>
  {/if}
</section>

<style>
  .strip {
    padding: 22px 34px 0;
  }

  .covers {
    display: flex;
    align-items: flex-start;
    gap: 14px;
  }

  .cover {
    position: relative;
    width: 118px;
    height: 118px;
    border-radius: 14px;
    overflow: hidden;
    background: rgba(28, 20, 22, 0.85);
    border: 2px solid transparent;
    transition:
      transform 0.18s ease,
      border-color 0.18s ease;
  }

  .cover:hover {
    transform: translateY(-3px);
  }

  .cover.active {
    width: 174px;
    height: 174px;
    border-color: var(--ember);
    box-shadow: 0 0 26px rgba(224, 27, 36, 0.45);
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .fallback {
    display: grid;
    place-items: center;
    width: 100%;
    height: 100%;
    font-size: 26px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.5);
  }

  .running {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #3ad46b;
    box-shadow: 0 0 10px #3ad46b;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
    place-content: center;
    padding: 34px;
    background: rgba(12, 8, 10, 0.8);
  }

  .grid .tile {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    background: #fff;
  }

  .grid .tile:nth-child(2),
  .grid .tile:nth-child(3) {
    background: rgba(255, 255, 255, 0.6);
  }

  .caption {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 14px 0 0 132px;
    font-size: 18px;
    font-weight: 600;
  }

  .badge {
    padding: 3px 10px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.14);
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .fav {
    color: var(--ember-bright);
  }
</style>
