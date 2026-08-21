<script lang="ts">
  import { library } from "$lib/library.svelte";
  import { formatClock } from "$lib/format";

  let {
    tab = $bindable<"games" | "media">("games"),
    onSettings
  }: { tab: "games" | "media"; onSettings: () => void } = $props();

  let searching = $state(false);
  let clock = $state(formatClock(new Date()));

  $effect(() => {
    const timer = setInterval(() => (clock = formatClock(new Date())), 10_000);
    return () => clearInterval(timer);
  });
</script>

<header>
  <nav>
    <button class:active={tab === "games"} onclick={() => (tab = "games")}>Games</button>
    <button class:active={tab === "media"} onclick={() => (tab = "media")}>Media</button>
  </nav>

  <div class="right">
    {#if searching}
      <input
        class="search"
        placeholder="Search your library…"
        bind:value={library.query}
        onblur={() => (searching = library.query.length > 0)}
      />
    {/if}
    <button class="icon" title="Search" onclick={() => (searching = !searching)} aria-label="Search">
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" stroke-linecap="round" />
      </svg>
    </button>
    <button class="icon" title="Settings" onclick={onSettings} aria-label="Settings">
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3.2" />
        <path
          d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1.11-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06A2 2 0 1 1 4.17 16.9l.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09A1.7 1.7 0 0 0 4.6 8.9a1.7 1.7 0 0 0-.34-1.87l-.06-.06A2 2 0 1 1 7.03 4.14l.06.06a1.7 1.7 0 0 0 1.87.34H9a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.55 1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87V10a1.7 1.7 0 0 0 1.55 1H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.51 1Z"
        />
      </svg>
    </button>
    <div class="avatar">
      <span class="online"></span>
    </div>
    <span class="clock">{clock}</span>
  </div>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 34px 0;
  }

  nav {
    display: flex;
    gap: 26px;
  }

  nav button {
    font-size: 21px;
    font-weight: 600;
    color: var(--text-dim);
    padding-bottom: 8px;
    border-bottom: 3px solid transparent;
    transition: color 0.15s ease;
  }

  nav button.active {
    color: #fff;
    border-color: var(--ember);
  }

  .right {
    display: flex;
    align-items: center;
    gap: 18px;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    color: rgba(255, 255, 255, 0.85);
    transition: background 0.15s ease;
  }

  .icon:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .search {
    width: 240px;
  }

  .avatar {
    position: relative;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: linear-gradient(140deg, #4a3a3a, #1b1416);
    border: 2px solid rgba(255, 255, 255, 0.25);
  }

  .online {
    position: absolute;
    top: 0;
    right: 0;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #3ad46b;
    border: 2px solid var(--ink);
  }

  .clock {
    font-size: 20px;
    font-weight: 500;
    letter-spacing: 0.02em;
  }
</style>
