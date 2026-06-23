<script lang="ts">
  // TypingWarnings — определение неверной раскладки и Caps Lock.

  let {
    expectedLanguage = 'en',
    lastTypedChar = '',
    capsLockOn = false,
    showLayoutWarnings = true,
    showCapsLockWarnings = true,
  }: {
    expectedLanguage?: string;
    lastTypedChar?: string;
    capsLockOn?: boolean;
    showLayoutWarnings?: boolean;
    showCapsLockWarnings?: boolean;
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
    <span class="warning-icon">❌</span>
    <div class="warning-text">
      <strong>Неверная раскладка</strong>
      <p>Используется {expectedLanguage === 'ru' ? 'EN' : 'RU'} раскладка.
      Переключитесь на {expectedLanguage === 'ru' ? 'RU' : 'EN'}.</p>
    </div>
  </div>
{/if}

{#if showCapsWarning}
  <div class="warning-card caps-warning">
    <span class="warning-icon">⚠</span>
    <div class="warning-text">
      <strong>Caps Lock включён</strong>
      <p>Caps Lock может снизить точность ввода.</p>
    </div>
  </div>
{/if}

<style>
  .warning-card {
    display: flex; gap: 0.5rem; align-items: flex-start;
    padding: 0.75rem 1rem; border-radius: 8px; margin-bottom: 0.5rem;
    font-size: 0.875rem; max-width: 400px;
  }
  .layout-warning { background: rgba(202,71,84,0.15); border: 1px solid var(--error); }
  .caps-warning { background: rgba(226,183,20,0.15); border: 1px solid var(--main); }
  .warning-icon { font-size: 1.2rem; }
  .warning-text strong { color: var(--text); display: block; }
  .warning-text p { color: var(--sub); font-size: 0.75rem; margin: 0.25rem 0 0; }
</style>