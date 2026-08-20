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

  // Script-based detection: определяем письменность ожидаемого языка и
  // символа. Warning только если символ из ДРУГОЙ письменности — общая
  // кириллица ru/uk больше не даёт ложных «switch to EN» (старая логика
  // понимала только en↔ru и врала для 13 из 15 языков).
  type Script = 'latin' | 'cyrillic' | 'cjk' | 'other';

  const CYRILLIC_LANGS = new Set(['ru', 'uk']);
  const CJK_LANGS = new Set(['ja', 'zh-hk', 'zh-tw', 'ko']);

  function expectedScript(lang: string): Script {
    if (CYRILLIC_LANGS.has(lang)) return 'cyrillic';
    if (CJK_LANGS.has(lang)) return 'cjk';
    // Остальные поддерживаемые языки (en, de, fr, es, ...) — латиница.
    return 'latin';
  }

  function charScript(ch: string): Script {
    if (/[a-zA-Zà-ÿÀ-Ýā-žĀ-Ž]/u.test(ch)) return 'latin';
    if (/[а-яА-ЯёЁїієґЇІЄҐ]/u.test(ch)) return 'cyrillic';
    if (/[\u3040-\u30ff\u3400-\u9fff\uf900-\ufaff\uac00-\ud7af]/u.test(ch)) return 'cjk';
    return 'other';
  }

  let layoutMismatch = $derived.by(() => {
    if (!showLayoutWarnings || !lastTypedChar) return false;
    const cs = charScript(lastTypedChar);
    if (cs === 'other') return false; // цифры/пунктуация — не показатель
    return cs !== expectedScript(expectedLanguage);
  });

  let scriptLabels = $derived.by(() => {
    const expected = expectedScript(expectedLanguage);
    const typed = charScript(lastTypedChar || ' ');
    const names: Record<Script, string> = {
      latin: 'Latin', cyrillic: 'Cyrillic', cjk: 'CJK', other: '—',
    };
    return { typed: names[typed], expected: names[expected] };
  });

  let showCapsWarning = $derived(showCapsLockWarnings && capsLockOn);
</script>

{#if layoutMismatch}
  <div class="warning-card layout-warning">
    <StatusIcon kind="cross" label="Keyboard layout mismatch" />
    <div class="warning-text">
      <strong>{t(uiLang, 'warning.layout_title')}</strong>
      <p>{t(uiLang, 'warning.layout_message')
        .replace('{current}', scriptLabels.typed)
        .replace('{expected}', scriptLabels.expected)}</p>
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
    box-shadow: 0 10px 30px rgba(0,0,0,0.3); animation: slide-in 0.2s ease-out;
  }
  .layout-warning { background: rgba(202,71,84,0.15); border: 1px solid var(--error); }
  .caps-warning { background: rgba(226,183,20,0.15); border: 1px solid var(--main); }
  .warning-icon { font-size: 1.2rem; }
  .warning-text strong { color: var(--text); display: block; }
  .warning-text p { color: var(--sub); font-size: 0.75rem; margin: 0.25rem 0 0; }
  .caps-warning { top: 6.5rem; }
  @keyframes slide-in { from { transform: translateX(110%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
</style>
