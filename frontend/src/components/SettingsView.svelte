<script lang="ts">
  import type { AppSettings, ThemeInfo } from '../lib/types/index';
  import { t, UI_LANGUAGES } from '../lib/i18n';
  import ProfileTransferPanel from './ProfileTransferPanel.svelte';

  let {
    settings,
    themes,
    activeTheme,
    uiLang = 'en',
    onSelectTheme,
    onUpdateSetting,
  }: {
    settings: AppSettings | null;
    themes: ThemeInfo[];
    activeTheme: string;
    uiLang?: string;
    onSelectTheme: (name: string) => void;
    onUpdateSetting: (key: string, value: unknown) => void;
  } = $props();

  let themeSearch = $state('');
  let filteredThemes = $derived.by(() => {
    const query = themeSearch.trim().toLowerCase();
    if (!query) return themes;
    return themes.filter((theme) =>
      theme.name.toLowerCase().includes(query)
      || theme.display_name.toLowerCase().includes(query),
    );
  });
</script>

<div class="list-view">
  <h2>{t(uiLang, 'settings.title')}</h2>
  {#if settings}
    <div class="settings-form">
      <div class="setting-row">
        <label for="setting-ui-lang">{t(uiLang, 'settings.ui_language')}</label>
        <select id="setting-ui-lang" value={settings.ui_language} onchange={(e) => onUpdateSetting('ui_language', e.currentTarget.value)}>
          {#each UI_LANGUAGES as [code, name]}
            <option value={code} selected={code === settings.ui_language}>{name}</option>
          {/each}
        </select>
      </div>
      <div class="setting-row">
        <label for="setting-theme">{t(uiLang, 'settings.theme')}</label>
        <select id="setting-theme" value={settings.theme} onchange={(e) => onSelectTheme(e.currentTarget.value)}>
          {#each themes as t2}
            <option value={t2.name} selected={t2.name === settings.theme}>{t2.display_name}</option>
          {/each}
        </select>
      </div>
      <div class="setting-row">
        <label for="setting-font-size">{t(uiLang, 'settings.font_size')}</label>
        <input id="setting-font-size" type="number" value={settings.font_size} onchange={(e) => onUpdateSetting('font_size', parseInt(e.currentTarget.value))} />
      </div>
      <div class="setting-row">
        <label for="setting-caret">{t(uiLang, 'settings.caret_style')}</label>
        <select id="setting-caret" value={settings.caret_style} onchange={(e) => onUpdateSetting('caret_style', e.currentTarget.value)}>
          <option value="underline">Underline</option>
          <option value="block">Block</option>
          <option value="solid">Solid</option>
          <option value="off">Off</option>
        </select>
      </div>
      <div class="setting-row">
        <label for="setting-live-wpm">{t(uiLang, 'settings.show_live_wpm')}</label>
        <input id="setting-live-wpm" type="checkbox" checked={settings.show_live_wpm} onchange={(e) => onUpdateSetting('show_live_wpm', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-accuracy">{t(uiLang, 'settings.show_accuracy')}</label>
        <input id="setting-accuracy" type="checkbox" checked={settings.show_accuracy} onchange={(e) => onUpdateSetting('show_accuracy', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-hand-guide">{t(uiLang, 'settings.hand_guide')}</label>
        <input id="setting-hand-guide" type="checkbox" checked={settings.show_hand_guide} onchange={(e) => onUpdateSetting('show_hand_guide', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-capslock">{t(uiLang, 'settings.capslock_warnings')}</label>
        <input id="setting-capslock" type="checkbox" checked={settings.show_capslock_warnings} onchange={(e) => onUpdateSetting('show_capslock_warnings', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-sound">{t(uiLang, 'settings.sound_enabled')}</label>
        <input id="setting-sound" type="checkbox" checked={settings.sound_enabled} onchange={(e) => onUpdateSetting('sound_enabled', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-volume">{t(uiLang, 'settings.sound_volume')}</label>
        <input id="setting-volume" type="range" min="0" max="1" step="0.1" value={settings.sound_volume} onchange={(e) => onUpdateSetting('sound_volume', parseFloat(e.currentTarget.value))} />
      </div>
      <div class="setting-row">
        <label for="setting-zen">{t(uiLang, 'settings.zen_mode')}</label>
        <input id="setting-zen" type="checkbox" checked={settings.zen_mode_enabled} onchange={(e) => onUpdateSetting('zen_mode_enabled', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-vim">{t(uiLang, 'settings.vim_mode')}</label>
        <input id="setting-vim" type="checkbox" checked={settings.vim_mode} onchange={(e) => onUpdateSetting('vim_mode', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label for="setting-goal-type">{t(uiLang, 'settings.daily_goal_type')}</label>
        <select id="setting-goal-type" value={settings.daily_goal_type || 'time'} onchange={(e) => onUpdateSetting('daily_goal_type', e.currentTarget.value)}>
          <option value="time">{t(uiLang, 'settings.goal_time')}</option>
          <option value="wpm">{t(uiLang, 'settings.goal_wpm')}</option>
          <option value="accuracy">{t(uiLang, 'settings.goal_accuracy')}</option>
        </select>
      </div>
      {#if settings.daily_goal_type === 'wpm'}
        <div class="setting-row">
          <label for="setting-goal-wpm">{t(uiLang, 'settings.daily_goal_wpm')}</label>
          <input id="setting-goal-wpm" type="number" min="0" max="300" value={settings.daily_goal_wpm || 0} onchange={(e) => onUpdateSetting('daily_goal_wpm', parseFloat(e.currentTarget.value))} />
        </div>
      {/if}
      {#if settings.daily_goal_type === 'accuracy'}
        <div class="setting-row">
          <label for="setting-goal-acc">{t(uiLang, 'settings.daily_goal_accuracy')}</label>
          <input id="setting-goal-acc" type="number" min="0" max="100" step="0.1" value={settings.daily_goal_accuracy || 0} onchange={(e) => onUpdateSetting('daily_goal_accuracy', parseFloat(e.currentTarget.value))} />
        </div>
      {/if}
    </div>
    <ProfileTransferPanel {uiLang} />
    <h3>{t(uiLang, 'settings.theme_preview')}</h3>
    <div class="theme-toolbar">
      <input
        class="theme-search"
        type="search"
        placeholder={t(uiLang, 'settings.theme_search')}
        value={themeSearch}
        oninput={(event) => { themeSearch = event.currentTarget.value; }}
      />
      <span>{filteredThemes.length}/{themes.length}</span>
    </div>
    <div class="theme-cards">
      {#each filteredThemes as t2}
        <button
          type="button"
          class="theme-card {t2.name === activeTheme ? 'active' : ''}"
          style="background: {t2.preview_colors.bg}; border-color: {t2.preview_colors.main};"
          onclick={() => onSelectTheme(t2.name)}
        >
          <span style="color: {t2.preview_colors.main}">{t2.display_name}</span>
          <span style="color: {t2.preview_colors.text}">Sample text</span>
          <span style="color: {t2.preview_colors.error}">error</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .list-view { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 1rem 0 0.5rem; }
  .settings-form { display: flex; flex-direction: column; gap: 1rem; }
  .setting-row { display: flex; align-items: center; gap: 1rem; }
  .setting-row label { min-width: 180px; color: var(--sub); font-size: 0.875rem; }
  .setting-row input, .setting-row select {
    background-color: var(--bg-sub) !important; border: 1px solid var(--sub); color: var(--text) !important;
    padding: 0.5rem; font-family: inherit; border-radius: 4px; font-size: 0.875rem;
  }
  .setting-row select { appearance: none; min-width: 155px; padding-right: 2rem; background-image: linear-gradient(45deg, transparent 50%, var(--main) 50%), linear-gradient(135deg, var(--main) 50%, transparent 50%); background-position: calc(100% - 14px) 52%, calc(100% - 9px) 52%; background-size: 5px 5px, 5px 5px; background-repeat: no-repeat; }
  .setting-row select option { background-color: var(--bg-sub); color: var(--text); }
  .setting-row input[type='checkbox'] { accent-color: var(--main); }
  .setting-row input[type='range'] { accent-color: var(--main); background: transparent !important; }
  .setting-row input:focus, .setting-row select:focus, .theme-search:focus { outline: 2px solid color-mix(in srgb, var(--main) 55%, transparent); outline-offset: 1px; border-color: var(--main); }
  .theme-toolbar { display: flex; align-items: center; gap: 1rem; margin-bottom: 0.75rem; }
  .theme-toolbar span { color: var(--sub); font-size: 0.7rem; }
  .theme-search { min-width: 260px; padding: 0.5rem 0.75rem; border: 1px solid var(--sub); border-radius: 6px; background: var(--bg-sub); color: var(--text); font: inherit; font-size: 0.75rem; }
  .theme-search:focus { outline: none; border-color: var(--main); }
  .theme-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(145px, 1fr)); gap: 0.65rem; max-height: 460px; overflow-y: auto; padding: 0.25rem; scrollbar-color: var(--sub) var(--bg-sub); }
  .theme-card {
    padding: 0.75rem; border-radius: 8px; border: 2px solid transparent; cursor: pointer;
    display: flex; flex-direction: column; gap: 0.25rem; min-width: 0;
    font-family: inherit; text-align: left;
  }
  .theme-card.active { border-color: var(--main); }
</style>
