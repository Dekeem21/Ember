<script lang="ts">
  import { assetUrl } from "$lib/api";
  import { library } from "$lib/library.svelte";

  let { onEdit }: { onEdit: () => void } = $props();

  const game = $derived(library.selected);
  const running = $derived(library.isSelectedRunning);
  let menuOpen = $state(false);
</script>

{#if game}
  <div class="hero">
    <span class="chip">{game.genres[0] ?? (game.installed ? "Installed" : "Not installed")}</span>

    {#if assetUrl(game.logoPath)}
      <img class="logo" src={assetUrl(game.logoPath)} alt={game.name} />
    {:else}
      <h1>{game.name}</h1>
    {/if}

    <p class="blurb">{game.description ?? "No description yet — refresh metadata to fetch one."}</p>

    <div class="actions">
      <button class="play" onclick={() => (running ? library.stop() : library.play())}>
        {running ? "Stop Game" : "Play Game"}
      </button>

      <div class="more">
        <button class="dots" onclick={() => (menuOpen = !menuOpen)} aria-label="More actions">
          ···
        </button>
        {#if menuOpen}
          <div class="menu">
            <button
              onclick={() => {
                menuOpen = false;
                onEdit();
              }}>Details & settings</button
            >
            <button
              onclick={() => {
                menuOpen = false;
                library.refreshSelected();
              }}>Refresh metadata</button
            >
            <button
              onclick={() => {
                menuOpen = false;
                library.toggleFavorite(game);
              }}>{game.favorite ? "Remove favourite" : "Add to favourites"}</button
            >
            <button
              onclick={() => {
                menuOpen = false;
                library.hide(game);
              }}>Hide from library</button
            >
          </div>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <div class="hero empty">
    <h1>No games yet</h1>
    <p class="blurb">Run a library scan to import Steam, Heroic, Lutris and local games.</p>
    <button class="play" onclick={() => library.scan()}>Scan library</button>
  </div>
{/if}

<style>
  .hero {
    max-width: 540px;
  }

  .chip {
    display: inline-block;
    padding: 5px 12px;
    border: 1px solid var(--ember);
    border-radius: 8px;
    color: #fff;
    font-size: 13px;
  }

  h1 {
    margin: 18px 0 12px;
    font-size: 58px;
    line-height: 0.95;
    font-weight: 800;
    letter-spacing: -0.02em;
    text-shadow: 0 8px 40px rgba(0, 0, 0, 0.7);
  }

  .logo {
    display: block;
    margin: 18px 0 12px;
    max-width: 440px;
    max-height: 170px;
    object-fit: contain;
    filter: drop-shadow(0 10px 30px rgba(0, 0, 0, 0.6));
  }

  .blurb {
    max-width: 470px;
    margin: 0 0 26px;
    font-size: 15px;
    line-height: 1.5;
    color: rgba(255, 255, 255, 0.82);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .play {
    padding: 16px 46px;
    border-radius: 999px;
    font-size: 18px;
    font-weight: 600;
    background: linear-gradient(120deg, var(--ember), #a3121a);
    box-shadow: 0 10px 30px rgba(224, 27, 36, 0.35);
    transition: transform 0.15s ease;
  }

  .play:hover {
    transform: translateY(-2px);
  }

  .more {
    position: relative;
  }

  .dots {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid var(--panel-border);
    font-size: 20px;
    line-height: 1;
  }

  .menu {
    position: absolute;
    bottom: 62px;
    left: 0;
    width: 220px;
    padding: 6px;
    border-radius: 14px;
    background: rgba(16, 11, 13, 0.96);
    border: 1px solid var(--panel-border);
    display: grid;
    z-index: 5;
  }

  .menu button {
    padding: 10px 12px;
    text-align: left;
    border-radius: 10px;
    font-size: 14px;
  }

  .menu button:hover {
    background: rgba(224, 27, 36, 0.22);
  }
</style>
