<script lang="ts">
  /**
   * Universal SVG icon component.
   * Uses `currentColor` so every icon inherits the parent's `color` CSS property,
   * making it theme-aware without any hardcoded palette.
   *
   * Size is controlled via the `size` prop (default 1em) or the parent font-size.
   */
  let {
    name,
    size = '1em',
  }: {
    name: 'flame' | 'trophy' | 'lock' | 'check' | 'circle' | 'warn';
    size?: string;
  } = $props();
</script>

<!-- biome-ignore lint/a11y/noSvgWithoutTitle: icons are decorative with aria-label on parent -->
<svg
  class="icon icon-{name}"
  viewBox="0 0 20 20"
  width={size}
  height={size}
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'flame'}
    <!-- Flame / fire icon -->
    <path d="M10 2c0 0-5 6-5 10a5 5 0 0 0 10 0c0-4-5-10-5-10z" fill="currentColor" stroke="none" />
    <path d="M10 6c0 0-3 4-3 6.5a3 3 0 0 0 6 0C13 10 10 6 10 6z" fill="var(--bg-sub, #15181d)" stroke="none" />
  {:else if name === 'trophy'}
    <!-- Trophy / cup icon -->
    <path d="M5 3h10v2a5 5 0 0 1-2.5 4.33L14 16H6l1.5-6.67A5 5 0 0 1 5 5V3z" fill="currentColor" stroke="none" />
    <line x1="4" y1="3" x2="16" y2="3" stroke="currentColor" />
    <line x1="10" y1="16" x2="10" y2="18" />
    <line x1="7" y1="18" x2="13" y2="18" />
  {:else if name === 'lock'}
    <!-- Lock (closed padlock) icon -->
    <rect x="4" y="9" width="12" height="8" rx="1.5" fill="currentColor" stroke="none" />
    <path d="M6 9V6a4 4 0 0 1 8 0v3" />
  {:else if name === 'check'}
    <!-- Checkmark icon -->
    <polyline points="4.5 10.5 8 14 15.5 6" />
  {:else if name === 'circle'}
    <!-- Circle (outline) icon -->
    <circle cx="10" cy="10" r="7" />
  {:else if name === 'warn'}
    <!-- Warning triangle icon -->
    <path d="M10 3.5L2.5 16.5h15L10 3.5z" />
    <line x1="10" y1="8" x2="10" y2="12" stroke-width="2" />
    <circle cx="10" cy="14" r="0.75" fill="currentColor" stroke="none" />
  {/if}
</svg>
