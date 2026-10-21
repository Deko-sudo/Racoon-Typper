<script lang="ts">
  let {
    kind,
    label,
  }: {
    kind: 'check' | 'cross';
    label: string;
  } = $props();
</script>

<span class="status-icon {kind}" role="img" aria-label={label}>
  <svg viewBox="0 0 20 20" aria-hidden="true">
    {#if kind === 'check'}
      <path d="m4.5 10.5 3.3 3.3 7.7-8" />
    {:else}
      <path d="m5.5 5.5 9 9m0-9-9 9" />
    {/if}
  </svg>
</span>

<style>
  .status-icon {
    width: 1.2rem;
    height: 1.2rem;
    flex: 0 0 1.2rem;
    display: inline-grid;
    place-items: center;
    border: 1.5px solid currentColor;
    border-radius: 0.3rem;
  }
  .status-icon.check {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 14%, transparent);
  }
  .status-icon.cross {
    color: var(--error);
    background: color-mix(in srgb, var(--error) 14%, transparent);
  }
  svg { width: 0.9rem; height: 0.9rem; }
  path {
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
