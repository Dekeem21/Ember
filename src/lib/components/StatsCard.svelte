<script lang="ts">
  import { library } from "$lib/library.svelte";
  import { formatDuration, formatHours } from "$lib/format";

  const game = $derived(library.selected);
  const stats = $derived(library.stats);

  /// Ring fill compares this game against the library average playtime.
  const ratio = $derived.by(() => {
    const average = stats?.averagePlaytimeSeconds ?? 0;
    if (!game || average <= 0) return 0.65;
    return Math.max(0.05, Math.min(1, game.playtimeSeconds / (average * 2)));
  });

  const circumference = 2 * Math.PI * 52;
</script>

<section class="card">
  <h3>Your Stats</h3>

  <div class="body">
    <div class="ring">
      <svg viewBox="0 0 120 120" width="132" height="132">
        <circle cx="60" cy="60" r="52" fill="none" stroke="rgba(255,255,255,0.1)" stroke-width="9" />
        <circle
          cx="60"
          cy="60"
          r="52"
          fill="none"
          stroke="url(#emberGradient)"
          stroke-width="9"
          stroke-linecap="round"
          stroke-dasharray={`${circumference * ratio} ${circumference}`}
          transform="rotate(-90 60 60)"
        />
        <defs>
          <linearGradient id="emberGradient" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#ff3b3b" />
            <stop offset="100%" stop-color="#8c0d13" />
          </linearGradient>
        </defs>
      </svg>
      <div class="ring-label">
        <span>Total Playtime</span>
        <strong>{formatHours(game?.playtimeSeconds ?? 0)}</strong>
      </div>
    </div>

    <ul>
      <li>
        <span class="figure">{game?.sessionCount ?? 0}</span>
        <span class="label">Sessions</span>
      </li>
      <li>
        <span class="figure">{formatDuration(game?.averageSessionSeconds ?? 0)}</span>
        <span class="label">Avg. session</span>
      </li>
      <li>
        <span class="figure">{formatDuration(stats?.averagePlaytimeSeconds ?? 0)}</span>
        <span class="label">Library average</span>
      </li>
    </ul>
  </div>
</section>

<style>
  .card {
    flex: 1;
    min-width: 420px;
    padding: 16px 22px 18px;
    border-radius: var(--radius);
    background: var(--panel);
    border: 1px solid var(--panel-border);
    backdrop-filter: blur(18px);
  }

  h3 {
    margin: 0 0 6px;
    font-size: 16px;
  }

  .body {
    display: flex;
    align-items: center;
    gap: 30px;
  }

  .ring {
    position: relative;
    display: grid;
    place-items: center;
  }

  .ring-label {
    position: absolute;
    text-align: center;
  }

  .ring-label span {
    display: block;
    font-size: 11px;
    color: var(--text-dim);
  }

  .ring-label strong {
    font-size: 20px;
  }

  ul {
    flex: 1;
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 12px;
  }

  li {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .figure {
    min-width: 92px;
    font-size: 21px;
    font-weight: 700;
  }

  .label {
    font-size: 13px;
    color: var(--text-dim);
  }
</style>
