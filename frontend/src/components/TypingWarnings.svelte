<script lang="ts">
  // TypingWarnings — определение неверной раскладки и Caps Lock.
  import { t } from '../lib/i18n';
  import Icon from './Icon.svelte';
  import StatusIcon from './StatusIcon.svelte';

  let {
    expectedLanguage = 'en',
    lastTypedChar = '',
    capsLockOn = false,
    showLayoutWarnings = true,
    showCapsLockWarnings = true,
    uiLang = 'en',
  }: {
    expectedLanguage?: string;
    lastTypedChar?: string;
    capsLockOn?: boolean;
    showLayoutWarnings?: boolean;
    showCapsLockWarnings?: boolean;
    uiLang?: string;
  } = $props();

  // Layout detection
  const EN_REGEX = /^[a-zA-Z]$/;
  const RU_REGEX = /^[а-яА-ЯёЁ]$/;

  let layoutMismatch = $derived.by(() => {
    if (!showLayoutWarnings || !lastTypedChar) return false;
    if (expectedLanguage === 'ru') {
      // Expected RU, but typed EN
      return EN_REGEX.test(lastTypedChar);
    } else {
      // Expected EN, but typed RU
      return RU_REGEX.test(lastTypedChar);
    }
  });

  let showCapsWarning = $derived(showCapsLockWarnings && capsLockOn);
</script>

{#if layoutMismatch}
  <div class="warning-card layout-warning">
    <StatusIcon kind="cross" label="Keyboard layout mismatch" />
    <div class="warning-text">
      <strong>{t(uiLang, 'warning.layout_title')}</strong>
      <p>{t(uiLang, 'warning.layout_message')
        .replace('{current}', expectedLanguage === 'ru' ? 'EN' : 'RU')
        .replace('{expected}', expectedLanguage === 'ru' ? 'RU' : 'EN')}</p>
    </div>
  </div>
{/if}

{#if showCapsWarning}
  <div class="warning-card caps-warning">
    <span class="warning-icon"><Icon name="warn" size="1.2rem" /></span>
    <div class="warning-text">
      <strong>{t(uiLang, 'warning.caps_title')}</strong>
      <p>{t(uiLang, 'warning.caps_message')}</p>
    </div>
  </div>
{/if}

<style>
  .warning-card {
    position: fixed; top: 1rem; right: 1rem; z-index: 200;
    display: flex; gap: 0.5rem; align-items: flex-start;
    padding: 0.75rem 1rem; border-radius: 8px;
    font-size: 0.875rem; max-width: 360px;
    box-shadow: var(--shadow-elevated); animation: slide-in 0.2s ease-out;
  }
  .layout-warning { background: color-mix(in srgb, var(--color-error) 15%, var(--color-surface-raised)); border: 1px solid var(--color-error); }
  .caps-warning { background: color-mix(in srgb, var(--color-warning) 15%, var(--color-surface-raised)); border: 1px solid var(--color-warning); }
  .warning-icon { font-size: 1.2rem; }
  .warning-text strong { color: var(--text); display: block; }
  .warning-text p { color: var(--sub); font-size: 0.75rem; margin: 0.25rem 0 0; }
  .caps-warning { top: 6.5rem; }
  @keyframes slide-in { from { transform: translateX(110%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
</style>
