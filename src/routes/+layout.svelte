<script lang="ts">
  let { children } = $props();
</script>

{@render children()}

<style>
  :global(:root) {
    --ember: #e01b24;
    --ember-bright: #ff2b34;
    --ember-deep: #7d0d13;
    --ink: #08060a;
    --ink-soft: rgba(18, 12, 14, 0.72);
    --panel: rgba(20, 14, 16, 0.66);
    --panel-border: rgba(255, 255, 255, 0.08);
    --text: #f5f1f1;
    --text-dim: rgba(245, 241, 241, 0.62);
    --radius: 18px;
    color-scheme: dark;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
    background: var(--ink);
    color: var(--text);
    font-family: "Inter", "Segoe UI", system-ui, sans-serif;
    overflow: hidden;
    user-select: none;
  }

  :global(button) {
    font: inherit;
    color: inherit;
    cursor: pointer;
    border: none;
    background: none;
  }

  :global(input),
  :global(select),
  :global(textarea) {
    font: inherit;
    color: inherit;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--panel-border);
    border-radius: 10px;
    padding: 8px 12px;
    outline: none;
  }

  :global(input:focus),
  :global(select:focus),
  :global(textarea:focus) {
    border-color: var(--ember);
  }

  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: rgba(255, 255, 255, 0.16);
    border-radius: 8px;
  }
</style>
