<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { assetUrl } from "$lib/api";
  import { library } from "$lib/library.svelte";
  import CoverStrip from "$lib/components/CoverStrip.svelte";
  import Dock from "$lib/components/Dock.svelte";
  import GameDetailsModal from "$lib/components/GameDetailsModal.svelte";
  import HeroDetails from "$lib/components/HeroDetails.svelte";
  import LibraryOverlay from "$lib/components/LibraryOverlay.svelte";
  import NowPlaying from "$lib/components/NowPlaying.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import StatsCard from "$lib/components/StatsCard.svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import TrailerCard from "$lib/components/TrailerCard.svelte";
  import TrophyCard from "$lib/components/TrophyCard.svelte";

  let tab = $state<"games" | "media">("games");
  let showLibrary = $state(false);
  let showSettings = $state(false);
  let showDetails = $state(false);

  const game = $derived(library.selected);
  const background = $derived(assetUrl(game?.heroPath) ?? assetUrl(game?.coverPath));

  $effect(() => {
    library.init();
  });
</script>

<main>
  <div class="backdrop" style:background-image={background ? `url(${background})` : "none"}></div>
  <div class="scrim"></div>

  <TopBar bind:tab onSettings={() => (showSettings = true)} />

  {#if tab === "games"}
    <div class="top">
      <CoverStrip onOpenLibrary={() => (showLibrary = true)} />
      <TrophyCard />
    </div>

    <div class="middle">
      <HeroDetails onEdit={() => (showDetails = true)} />
    </div>

    <div class="bottom">
      <NowPlaying />
      <div class="cards">
        <TrailerCard />
        <StatsCard />
      </div>
    </div>
  {:else}
    <div class="media-view">
      {#if game?.trailerUrl}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video src={game.trailerUrl} controls></video>
      {:else}
        <p>No trailer stored for {game?.name ?? "this game"}. Refresh its metadata to fetch one.</p>
      {/if}
    </div>
  {/if}

  <div class="dock-wrap">
    <Dock
      onOpenLibrary={() => (showLibrary = true)}
      onSettings={() => (showSettings = true)}
      onQuit={() => getCurrentWindow().close()}
    />
  </div>

  {#if library.busy}
    <div class="toast">{library.busy}</div>
  {:else if library.error}
    <button class="toast error" onclick={() => (library.error = null)}>{library.error}</button>
  {/if}
</main>

{#if showLibrary}
  <LibraryOverlay onClose={() => (showLibrary = false)} />
{/if}
{#if showSettings}
  <SettingsModal onClose={() => (showSettings = false)} />
{/if}
{#if showDetails && game}
  <GameDetailsModal {game} onClose={() => (showDetails = false)} />
{/if}

<style>
  main {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .backdrop {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center;
    transform: scale(1.04);
    transition: background-image 0.4s ease;
  }

  .scrim {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(120% 90% at 70% 20%, rgba(120, 10, 14, 0.45), transparent 60%),
      linear-gradient(90deg, rgba(6, 4, 5, 0.95) 22%, rgba(6, 4, 5, 0.45) 60%, rgba(6, 4, 5, 0.9)),
      linear-gradient(rgba(6, 4, 5, 0.55), rgba(6, 4, 5, 0.92));
  }

  main > :global(*:not(.backdrop):not(.scrim)) {
    position: relative;
    z-index: 1;
  }

  .top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    padding-right: 34px;
  }

  .middle {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 0 34px;
  }

  .bottom {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 22px;
    padding: 0 34px 96px;
  }

  .cards {
    display: flex;
    align-items: stretch;
    gap: 22px;
  }

  .dock-wrap {
    position: absolute;
    left: 50%;
    bottom: 18px;
    transform: translateX(-50%);
    z-index: 3;
  }

  .media-view {
    flex: 1;
    display: grid;
    place-items: center;
    padding: 40px;
  }

  .media-view video {
    max-width: 80%;
    max-height: 70vh;
    border-radius: 16px;
  }

  .toast {
    position: absolute;
    top: 76px;
    left: 50%;
    transform: translateX(-50%);
    padding: 10px 22px;
    border-radius: 999px;
    background: rgba(16, 11, 13, 0.92);
    border: 1px solid var(--panel-border);
    font-size: 13px;
    z-index: 4;
  }

  .toast.error {
    border-color: var(--ember);
    color: #ffb4b4;
  }
</style>
