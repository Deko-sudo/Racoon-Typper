<script lang="ts">
  import { t } from '../lib/i18n';

  let {
    uiLang = 'en',
    matchCount = 0,
    onQuery,
    onClose,
  }: {
    uiLang?: string;
    matchCount: number;
    onQuery: (query: string) => void;
    onClose: () => void;
  } = $props();

  let query = $state('');
  let inputEl: HTMLInputElement | null = $state(null);

  $effect(() => {
    inputEl?.focus();
  });

  function submit() {
    onQuery(query);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      submit();
    }
  }
</script>

<div class="vim-search" role="search" aria-label={t(uiLang, 'vim.search_label')}>
  <span class="search-prefix">/</span>
  <input
    type="text"
    placeholder={t(uiLang, 'vim.search_placeholder')}
    bind:value={query}
    bind:this={inputEl}
    onkeydown={handleKeydown}
  />
  {#if matchCount > 0}
    <span class="search-count">{matchCount}</span>
  {/if}
  <button class="search-close" aria-label={t(uiLang, 'vim.search_close')} onclick={onClose}>×</button>
</div>

<style>
  .vim-search {
    position: fixed; top: 0.75rem; right: 0.75rem; z-index: 60;
    display: flex; align-items: center; gap: 0.4rem;
    background: var(--color-modal-surface); border: 1px solid var(--color-border-strong);
    border-radius: 6px; padding: 0.35rem 0.6rem; box-shadow: var(--shadow-elevated);
  }
  .search-prefix { color: var(--main); font-weight: 700; }
  .vim-search input {
    background: transparent; border: none; outline: none; color: var(--text);
    font-family: inherit; font-size: 0.8rem; width: 180px;
  }
  .search-count { color: var(--sub); font-size: 0.7rem; }
  .search-close {
    background: transparent; border: none; color: var(--sub); cursor: pointer;
    font-size: 0.9rem; line-height: 1; padding: 0 0.1rem;
  }
  .search-close:hover { color: var(--error); }
</style>
