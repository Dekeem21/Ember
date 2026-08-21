<script lang="ts">
  import { assetUrl } from "$lib/api";
  import { library } from "$lib/library.svelte";
  import { formatRelative } from "$lib/format";

  const game = $derived(library.selected);
  const trailer = $derived(game?.trailerUrl ?? null);
  let playing = $state(false);

  $effect(() => {
    // Reset the player whenever the selection changes.
    void game?.id;
    playing = false;
  });
</script>

<section class="card">
  <div class="head">
    <h3>Latest News</h3>
    <button onclick={() => library.refreshSelected()}>View All</button>
  </div>

  <div class="media">
    {#if playing && trailer}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video src={trailer} controls autoplay></video>
    {:else}
      {#if assetUrl(game?.heroPath)}
        <img src={assetUrl(game?.heroPath)} alt={game?.name ?? "Artwork"} />
      {:else}
        <div class="placeholder"></div>
      {/if}
      <div class="overlay">
        <p class="title">{game?.name ?? "Nothing selected"}</p>
        <p class="meta">
          {game?.releaseDate ?? "Unknown release"} · {formatRelative(game?.lastPlayedAt ?? null)}
        </p>
      </div>
      <button
        class="play"
        disabled={!trailer}
        title={trailer ? "Play trailer" : "No trailer available"}
        onclick={() => (playing = true)}
        aria-label="Play trailer"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
      </button>
    {/if}
  </div>
</section>

<style>
  .card {
    width: 360px;
    padding: 16px 18px 18px;
    border-radius: var(--radius);
    background: var(--panel);
    border: 1px solid var(--panel-border);
    backdrop-filter: blur(18px);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  h3 {
    margin: 0;
    font-size: 16px;
  }

  .head button {
    font-size: 13px;
    color: var(--text-dim);
  }

  .media {
    position: relative;
    height: 168px;
    border-radius: 14px;
    overflow: hidden;
  }

  .media img,
  .media video {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(140deg, #3a1013, #120a0c);
  }

  .overlay {
    position: absolute;
    inset: auto 0 0 0;
    padding: 14px 16px;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.85));
  }

  .title {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
    line-height: 1.2;
  }

  .meta {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--text-dim);
  }

  .play {
    position: absolute;
    right: 14px;
    bottom: 14px;
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--ember);
    box-shadow: 0 6px 18px rgba(224, 27, 36, 0.5);
  }

  .play:disabled {
    background: rgba(255, 255, 255, 0.16);
    box-shadow: none;
    cursor: not-allowed;
  }
</style>
