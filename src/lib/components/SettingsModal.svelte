<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type Settings, type UmuStatus } from "$lib/api";
  import { library } from "$lib/library.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let draft = $state<Settings>(
    library.settings ?? {
      steamApiKey: null,
      steamId64: null,
      steamgriddbApiKey: null,
      umuRunPath: "umu-run",
      defaultProtonVersion: "UMU-Latest",
      prefixRoot: null,
      extraLibraryDirs: [],
      closeToTray: false,
      autoplayTrailers: false
    }
  );
  let protonVersions = $state<string[]>([]);
  let umu = $state<UmuStatus | null>(null);
  let saved = $state(false);

  $effect(() => {
    api.protonVersions().then((versions) => (protonVersions = versions));
    api.umuStatus().then((status) => (umu = status));
  });

  async function pickDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      draft.extraLibraryDirs = [...draft.extraLibraryDirs, selected];
    }
  }

  async function save() {
    await library.saveSettings($state.snapshot(draft));
    saved = true;
    setTimeout(() => (saved = false), 1500);
  }
</script>

<div class="backdrop">
  <div class="modal">
    <header>
      <h2>Settings</h2>
      <button class="close" onclick={onClose} aria-label="Close">✕</button>
    </header>

    <div class="body">
      <section>
        <h3>Proton / umu-launcher</h3>
        <label>
          <span>umu-run path</span>
          <input bind:value={draft.umuRunPath} placeholder="umu-run" />
        </label>
        <p class="status" class:ok={umu?.available}>
          {#if umu?.available}
            Detected {umu.version || "umu-launcher"}
          {:else}
            umu-run not found — install it with <code>sudo pacman -S umu-launcher</code>
          {/if}
        </p>
        <label>
          <span>Default Proton build</span>
          <select bind:value={draft.defaultProtonVersion}>
            {#each protonVersions as version (version)}
              <option value={version}>{version}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Wine prefix root</span>
          <input
            value={draft.prefixRoot ?? ""}
            oninput={(event) => (draft.prefixRoot = event.currentTarget.value || null)}
            placeholder="~/.local/share/ember/prefixes"
          />
        </label>
      </section>

      <section>
        <h3>Library sources</h3>
        <div class="dirs">
          {#each draft.extraLibraryDirs as dir, index (dir)}
            <div class="dir">
              <span>{dir}</span>
              <button
                onclick={() =>
                  (draft.extraLibraryDirs = draft.extraLibraryDirs.filter((_, i) => i !== index))}
                aria-label="Remove folder">✕</button
              >
            </div>
          {/each}
        </div>
        <button class="secondary" onclick={pickDirectory}>Add game folder…</button>
        <p class="hint">
          Steam, Heroic and Lutris libraries are detected automatically. Extra folders are scanned
          for Windows and native executables.
        </p>
      </section>

      <section>
        <h3>Metadata & achievements</h3>
        <label>
          <span>Steam Web API key</span>
          <input
            value={draft.steamApiKey ?? ""}
            oninput={(event) => (draft.steamApiKey = event.currentTarget.value || null)}
            placeholder="Used for playtime and achievements"
          />
        </label>
        <label>
          <span>SteamID64</span>
          <input
            value={draft.steamId64 ?? ""}
            oninput={(event) => (draft.steamId64 = event.currentTarget.value || null)}
          />
        </label>
        <label>
          <span>SteamGridDB API key</span>
          <input
            value={draft.steamgriddbApiKey ?? ""}
            oninput={(event) => (draft.steamgriddbApiKey = event.currentTarget.value || null)}
            placeholder="Optional fallback artwork"
          />
        </label>
        <div class="row">
          <button class="secondary" onclick={() => api.syncSteamPlaytime().then(() => library.reload())}>
            Import Steam playtime
          </button>
          <button class="secondary" onclick={() => library.refreshArtwork()}>
            Download missing artwork
          </button>
        </div>
      </section>

      <section>
        <h3>Behaviour</h3>
        <label class="toggle">
          <input type="checkbox" bind:checked={draft.autoplayTrailers} />
          <span>Autoplay trailers on the dashboard</span>
        </label>
        <label class="toggle">
          <input type="checkbox" bind:checked={draft.closeToTray} />
          <span>Keep running in the background while a game is open</span>
        </label>
      </section>
    </div>

    <footer>
      {#if saved}<span class="saved">Saved</span>{/if}
      <button class="secondary" onclick={onClose}>Cancel</button>
      <button class="primary" onclick={save}>Save settings</button>
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
    width: min(760px, 92vw);
    max-height: 86vh;
    display: flex;
    flex-direction: column;
    border-radius: 20px;
    background: rgba(18, 12, 14, 0.98);
    border: 1px solid var(--panel-border);
  }

  header,
  footer {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 18px 24px;
  }

  header {
    justify-content: space-between;
    border-bottom: 1px solid var(--panel-border);
  }

  footer {
    justify-content: flex-end;
    border-top: 1px solid var(--panel-border);
  }

  h2 {
    margin: 0;
    font-size: 20px;
  }

  h3 {
    margin: 0 0 12px;
    font-size: 14px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ember-bright);
  }

  .body {
    overflow-y: auto;
    padding: 20px 24px;
    display: grid;
    gap: 26px;
  }

  label {
    display: grid;
    grid-template-columns: 200px 1fr;
    align-items: center;
    gap: 12px;
    margin-bottom: 10px;
    font-size: 14px;
  }

  label.toggle {
    grid-template-columns: 20px 1fr;
    color: var(--text-dim);
  }

  .row {
    display: flex;
    gap: 10px;
    margin-top: 8px;
  }

  .dirs {
    display: grid;
    gap: 6px;
    margin-bottom: 10px;
  }

  .dir {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.35);
    font-size: 13px;
  }

  .status {
    margin: 0 0 12px;
    font-size: 12px;
    color: #ffb4b4;
  }

  .status.ok {
    color: #7ce29a;
  }

  .hint {
    margin: 10px 0 0;
    font-size: 12px;
    color: var(--text-dim);
  }

  .primary,
  .secondary {
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

  .saved {
    margin-right: auto;
    font-size: 13px;
    color: #7ce29a;
  }

  .close {
    font-size: 16px;
    color: var(--text-dim);
  }
</style>
