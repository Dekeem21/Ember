<script lang="ts">
  import { library } from "$lib/library.svelte";
  import { api, type TrophySummary } from "$lib/api";

  let summary = $state<TrophySummary | null>(null);

  $effect(() => {
    const game = library.selected;
    if (!game) {
      summary = null;
      return;
    }
    api.trophySummary(game.id).then((value) => (summary = value));
  });

  async function sync() {
    const game = library.selected;
    if (!game) return;
    const result = await library.withBusy("Syncing achievements…", () =>
      api.syncAchievements(game.id)
    );
    if (result) summary = result;
  }

  const tiers = $derived([
    { key: "platinum", color: "#9fd8ff", value: summary?.platinum ?? 0 },
    { key: "gold", color: "#ffc850", value: summary?.gold ?? 0 },
    { key: "silver", color: "#d7d7de", value: summary?.silver ?? 0 },
    { key: "bronze", color: "#d08a4e", value: summary?.bronze ?? 0 }
  ]);
</script>

<aside class="card">
  <div class="head">
    <h3>Trophies</h3>
    <button onclick={sync} title="Sync achievements from Steam">Sync</button>
  </div>

  <div class="tiers">
    {#each tiers as tier (tier.key)}
      <div class="tier">
        <svg viewBox="0 0 24 24" width="22" height="22" fill={tier.color}>
          <path
            d="M7 3h10v2h3v3a4 4 0 0 1-3.4 3.95A5 5 0 0 1 13 14.9V17h3v2H8v-2h3v-2.1a5 5 0 0 1-3.6-2.95A4 4 0 0 1 4 8V5h3Zm-1 4v1a2 2 0 0 0 1.2 1.83A6.9 6.9 0 0 1 7 8V7Zm12 0v1c0 .63-.07 1.24-.2 1.83A2 2 0 0 0 19 8V7Z"
          />
        </svg>
        <span>{tier.value}</span>
      </div>
    {/each}
  </div>

  <div class="progress">
    <span>Progress</span>
    <span class="value">{Math.round(summary?.progress ?? 0)}%</span>
  </div>
  <div class="bar"><i style:width={`${summary?.progress ?? 0}%`}></i></div>
  <p class="hint">{summary?.unlocked ?? 0} of {summary?.total ?? 0} achievements</p>
</aside>

<style>
  .card {
    width: 300px;
    padding: 18px 20px;
    border-radius: var(--radius);
    background: var(--panel);
    border: 1px solid var(--panel-border);
    backdrop-filter: blur(18px);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h3 {
    margin: 0;
    font-size: 17px;
  }

  .head button {
    font-size: 12px;
    color: var(--text-dim);
  }

  .head button:hover {
    color: var(--ember-bright);
  }

  .tiers {
    display: flex;
    justify-content: space-between;
    margin: 16px 0 14px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--panel-border);
  }

  .tier {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
  }

  .progress {
    display: flex;
    justify-content: space-between;
    font-size: 14px;
    color: var(--text-dim);
  }

  .value {
    color: #fff;
    font-weight: 600;
  }

  .bar {
    margin-top: 8px;
    height: 6px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.12);
    overflow: hidden;
  }

  .bar i {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--ember-deep), var(--ember-bright));
  }

  .hint {
    margin: 10px 0 0;
    font-size: 12px;
    color: var(--text-dim);
  }
</style>
