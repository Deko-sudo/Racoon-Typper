<script lang="ts">
  import { t } from '../lib/i18n';

  let {
    uiLang = 'en',
    onClose,
  }: {
    uiLang?: string;
    onClose: () => void;
  } = $props();

  const sections = $derived.by((): Array<{ title: string; rows: Array<[string, string]> }> => [
    {
      title: t(uiLang, 'cheatsheet.navigation'),
      rows: [
        ['?', t(uiLang, 'cheatsheet.toggle')],
        ['Esc', t(uiLang, 'cheatsheet.close')],
      ],
    },
    {
      title: t(uiLang, 'cheatsheet.vim'),
      rows: [
        ['h / l', t(uiLang, 'vim.hint_prev') + ' / ' + t(uiLang, 'vim.hint_next')],
        ['j / k', t(uiLang, 'vim.hint_down') + ' / ' + t(uiLang, 'vim.hint_up')],
        ['gg / G', t(uiLang, 'vim.hint_top') + ' / ' + t(uiLang, 'vim.hint_bottom')],
        ['r', t(uiLang, 'vim.hint_restart')],
      ],
    },
    {
      title: t(uiLang, 'cheatsheet.test'),
      rows: [
        ['Backspace', t(uiLang, 'cheatsheet.backspace')],
        ['Ctrl+C / Ctrl+V', t(uiLang, 'cheatsheet.clipboard')],
      ],
    },
    {
      title: t(uiLang, 'cheatsheet.global'),
      rows: [
        ['Caps Lock', t(uiLang, 'cheatsheet.capslock')],
      ],
    },
  ]);
</script>

<div
  class="cheatsheet-overlay"
  role="dialog"
  aria-modal="true"
  aria-label={t(uiLang, 'cheatsheet.title')}
  tabindex="-1"
>
  <button class="backdrop" aria-label={t(uiLang, 'cheatsheet.close')} onclick={onClose}></button>
  <div class="cheatsheet-panel" role="document">
    <header class="cheatsheet-header">
      <h2>{t(uiLang, 'cheatsheet.title')}</h2>
      <button class="close-btn" aria-label={t(uiLang, 'cheatsheet.close')} onclick={onClose}>×</button>
    </header>
    {#each sections as section}
      <section class="cheatsheet-section">
        <h3>{section.title}</h3>
        <table>
          <tbody>
            {#each section.rows as [key, description]}
              <tr>
                <td class="key-cell"><kbd>{key}</kbd></td>
                <td class="desc-cell">{description}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </section>
    {/each}
  </div>
</div>

<style>
  .cheatsheet-overlay {
    position: fixed; inset: 0; z-index: 100;
    display: flex; align-items: center; justify-content: center;
    background: color-mix(in srgb, var(--color-overlay) 70%, transparent);
    padding: 1rem;
  }
  .backdrop {
    position: absolute; inset: 0; border: none; background: transparent; cursor: default;
  }
  .cheatsheet-panel {
    position: relative;
    width: min(560px, 100%); max-height: 85vh; overflow-y: auto;
    background: var(--color-modal-surface); border: 1px solid var(--color-border-strong);
    border-radius: 12px; padding: 1.25rem 1.5rem; box-shadow: var(--shadow-elevated);
  }
  .cheatsheet-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; }
  .cheatsheet-header h2 { color: var(--main); font-size: 1.25rem; }
  .close-btn {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    width: 1.75rem; height: 1.75rem; border-radius: 4px; cursor: pointer; font-size: 1rem; line-height: 1;
  }
  .close-btn:hover { color: var(--main); border-color: var(--main); }
  .cheatsheet-section { margin-bottom: 1rem; }
  .cheatsheet-section h3 { color: var(--main); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 0.4rem; }
  table { width: 100%; border-collapse: collapse; }
  tr { border-bottom: 1px solid var(--color-border); }
  tr:last-child { border-bottom: none; }
  td { padding: 0.4rem 0.5rem; font-size: 0.8rem; }
  .key-cell { width: 130px; }
  kbd {
    display: inline-block; min-width: 1.6em; text-align: center;
    padding: 0.15rem 0.45rem; background: var(--bg-sub); color: var(--text);
    border: 1px solid var(--color-border-strong); border-bottom-width: 2px;
    border-radius: 4px; font-family: inherit; font-size: 0.75rem; font-weight: 700;
  }
  .desc-cell { color: var(--sub); }
</style>
