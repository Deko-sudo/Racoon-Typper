<script lang="ts">
  /**
   * Universal SVG icon component.
   *
   * All icons use Lucide-compatible geometry (24×24 viewBox, stroke-only) so they
   * render crisply at any size and inherit the parent's `color` via `currentColor`.
   * Stroke width scales with size by default (no fixed stroke-width) which keeps
   * proportions balanced both for tiny (0.65rem) badges and large (2rem) cards.
   *
   * `size` accepts any CSS length (e.g. '2rem', '16px', '1em').
   */
  let {
    name,
    size = '1em',
  }: {
    name: 'flame' | 'trophy' | 'lock' | 'check' | 'circle' | 'warn' | 'x';
    size?: string;
  } = $props();
</script>

<svg
  class="icon icon-{name}"
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  width={size}
  height={size}
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  {#if name === 'flame'}
    <!-- Lucide flame -->
    <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z" />
  {:else if name === 'trophy'}
    <!-- Lucide trophy -->
    <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6" />
    <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18" />
    <path d="M4 22h16" />
    <path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22" />
    <path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22" />
    <path d="M18 2H6v7a6 6 0 0 0 12 0V2Z" />
  {:else if name === 'lock'}
    <!-- Lucide lock -->
    <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
  {:else if name === 'check'}
    <!-- Lucide check -->
    <polyline points="20 6 9 17 4 12" />
  {:else if name === 'circle'}
    <!-- Lucide circle -->
    <circle cx="12" cy="12" r="10" />
  {:else if name === 'warn'}
    <!-- Lucide triangle-alert -->
    <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
    <line x1="12" x2="12" y1="9" y2="13" />
    <line x1="12" x2="12.01" y1="17" y2="17" />
  {:else if name === 'x'}
    <!-- Lucide x -->
    <path d="M18 6 6 18" />
    <path d="m6 6 12 12" />
  {/if}
</svg>

<style>
  .icon { display: inline-block; flex-shrink: 0; vertical-align: middle; }
</style>
