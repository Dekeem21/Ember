<script lang="ts">
  import { library } from "$lib/library.svelte";

  let {
    onOpenLibrary,
    onSettings,
    onQuit
  }: { onOpenLibrary: () => void; onSettings: () => void; onQuit: () => void } = $props();

  const scanning = $derived(library.busy !== null);
</script>

<nav class="dock">
  <button class="item active" onclick={onOpenLibrary} title="Library" aria-label="Library">
    <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
      <path d="M12 3 3 10v11h6v-6h6v6h6V10z" />
    </svg>
  </button>

  <button class="item" title="Notifications" aria-label="Notifications">
    <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8">
      <path d="M18 8a6 6 0 1 0-12 0c0 7-3 8-3 8h18s-3-1-3-8" />
      <path d="M13.7 21a2 2 0 0 1-3.4 0" />
    </svg>
    {#if library.error}<span class="dot"></span>{/if}
  </button>

  <button class="item" onclick={() => library.scan()} title="Scan library" aria-label="Scan library">
    <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8">
      <path d="M21 12a9 9 0 1 1-2.6-6.4" />
      <path d="M21 3v6h-6" />
    </svg>
    {#if scanning}<span class="dot busy"></span>{/if}
  </button>

  <button
    class="item"
    onclick={() => library.refreshArtwork()}
    title="Download missing artwork"
    aria-label="Download missing artwork"
  >
    <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8">
      <rect x="3" y="4" width="18" height="16" rx="3" />
      <path d="m5 17 5-5 4 4 2-2 3 3" />
      <circle cx="9" cy="9" r="1.6" />
    </svg>
  </button>

  <button class="item" onclick={onSettings} title="Compatibility settings" aria-label="Compatibility settings">
    <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8">
      <path d="M6 8h12a3 3 0 0 1 3 3v2a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3v-2a3 3 0 0 1 3-3Z" />
      <path d="M8 12h.01M12 12h.01M16 12h.01" stroke-linecap="round" />
    </svg>
  </button>

  <span class="divider"></span>

  <button class="item" onclick={onQuit} title="Quit" aria-label="Quit">
    <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8">
      <path d="M12 3v9" stroke-linecap="round" />
      <path d="M6.5 6.5a8 8 0 1 0 11 0" stroke-linecap="round" />
    </svg>
  </button>
</nav>

<style>
  .dock {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    border-radius: 999px;
    background: rgba(16, 11, 13, 0.78);
    border: 1px solid var(--panel-border);
    backdrop-filter: blur(20px);
  }

  .item {
    position: relative;
    display: grid;
    place-items: center;
    width: 46px;
    height: 46px;
    border-radius: 50%;
    color: rgba(255, 255, 255, 0.82);
    transition: background 0.15s ease;
  }

  .item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .item.active {
    color: var(--ember-bright);
  }

  .dot {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ember-bright);
  }

  .dot.busy {
    background: #3ad46b;
  }

  .divider {
    width: 1px;
    height: 26px;
    margin: 0 6px;
    background: rgba(255, 255, 255, 0.18);
  }
</style>
