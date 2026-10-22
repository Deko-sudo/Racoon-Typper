<script lang="ts">
  import { t, UI_LANGUAGES } from '../lib/i18n';
  import { togglePracticeLanguage } from '../lib/onboarding';
  import type { LanguageCode } from '../lib/types/index';

  let { languages, onChange, uiLang = 'en', allowEmpty = true }: {
    languages: LanguageCode[];
    onChange: (languages: LanguageCode[]) => void;
    uiLang?: string;
    allowEmpty?: boolean;
  } = $props();
</script>

<fieldset>
  <legend>{t(uiLang, 'onboarding.step_practice_lang')}</legend>
  <p>{t(uiLang, 'onboarding.practice_languages_hint')}</p>
  <div class="lang-grid">
    {#each UI_LANGUAGES as [code, label] (code)}
      <label class:selected={languages.includes(code as LanguageCode)}>
        <input
          type="checkbox"
          checked={languages.includes(code as LanguageCode)}
          disabled={!allowEmpty && languages.length === 1 && languages.includes(code as LanguageCode)}
          onchange={() => onChange(togglePracticeLanguage(languages, code as LanguageCode))}
        />
        <span>{label}</span>
      </label>
    {/each}
  </div>
</fieldset>

<style>
  fieldset { border: 0; padding: 0; margin: 0; min-width: 0; }
  legend { font-size: 0.95rem; color: var(--text); text-transform: uppercase; letter-spacing: 0.04em; }
  p { color: var(--sub); font-size: 0.85rem; margin: 0.5rem 0 1rem; }
  .lang-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr)); gap: 0.5rem; }
  label { display: flex; align-items: center; gap: 0.5rem; padding: 0.55rem 0.5rem; background: var(--bg); color: var(--text); border: 1px solid transparent; border-radius: 6px; cursor: pointer; font-size: 0.85rem; }
  label:hover { border-color: var(--sub); }
  label:focus-within { outline: 2px solid var(--main); outline-offset: 2px; }
  label.selected { border-color: var(--main); color: var(--main); font-weight: bold; }
  input { accent-color: var(--main); margin: 0; flex-shrink: 0; }
</style>
