<script lang="ts">
  import type { AppSettings, ThemeInfo } from '../lib/types/index';
  import { t, UI_LANGUAGES } from '../lib/i18n';
  import ProfileTransferPanel from './ProfileTransferPanel.svelte';
  import { checkForUpdate, installUpdate } from '../lib/updater';

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

  let updateStatus = $state<string>('');
  let updateVersion = $state<string | null>(null);
  let checkingUpdate = $state(false);

  async function handleCheckUpdate() {
    checkingUpdate = true;
    updateStatus = '';
    updateVersion = null;
    const result = await checkForUpdate();
    checkingUpdate = false;
    if (result.error) {
      updateStatus = `Update check failed: ${result.error}`;
    } else if (result.available) {
      updateVersion = result.version ?? null;
      updateStatus = `Update available: v${result.version}`;
    } else {
      updateStatus = 'You are up to date.';
    }
  }

  async function handleInstallUpdate() {
    const ok = await installUpdate();
    if (!ok) updateStatus = 'Update install failed.';
  }

  const themeDescriptions: Record<string, string> = {
    racoon_graphite: 'Calm graphite surfaces with soft silver contrast.',
    racoon_silver: 'Neutral daylight theme with warm silver surfaces.',
    racoon_warm: 'Warm charcoal surfaces with a restrained copper accent.',
    racoon_high_contrast: 'Maximum contrast and clearly separated typing states.',
    midnight_ink: 'Deep navy-black surfaces for long nighttime sessions.',
    arctic_slate: 'Cold neutral gray with quiet icy-blue accents.',
    racoon_forest: 'Deep forest surfaces with muted natural highlights.',
    moss: 'Relaxed olive and moss tones for focused practice.',
    coffee: 'Dark coffee, walnut, and cream with a soft copper accent.',
    paper: 'Soft reading-paper daylight without a pure-white canvas.',
    sandstone: 'Subtle sandy daylight with warm, readable contrast.',
    mist: 'Cool light gray with a restrained neutral-blue accent.',
    lavender_dusk: 'Elegant desaturated purple for quiet evening sessions.',
    plum: 'Warm dark plum with a restrained rose accent.',
    ocean: 'Dark muted ocean and teal without a cyberpunk glow.',
    deep_sea: 'Very dark blue-green low-light palette.',
    ember: 'Charcoal with restrained ember-red and copper accents.',
    burgundy: 'Sophisticated subdued wine red with ivory text.',
    amber_terminal: 'Modern low-glare amber terminal reinterpretation.',
    green_terminal: 'Modern low-glare green terminal palette.',
    steel_blue: 'Industrial steel and muted blue for metallic focus.',
    carbon: 'Near-monochrome dark theme, flatter than Graphite.',
    moonlight: 'Soft blue-gray nighttime palette for low eye strain.',
    dawn: 'Soft warm daylight with a muted sunrise accent.',
    sage: 'Quiet desaturated green daylight for professional practice.',
  };

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
        <label class="toggle"><input id="setting-live-wpm" type="checkbox" checked={settings.show_live_wpm} onchange={(e) => onUpdateSetting('show_live_wpm', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-accuracy">{t(uiLang, 'settings.show_accuracy')}</label>
        <label class="toggle"><input id="setting-accuracy" type="checkbox" checked={settings.show_accuracy} onchange={(e) => onUpdateSetting('show_accuracy', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-hand-guide">{t(uiLang, 'settings.hand_guide')}</label>
        <label class="toggle"><input id="setting-hand-guide" type="checkbox" checked={settings.show_hand_guide} onchange={(e) => onUpdateSetting('show_hand_guide', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-capslock">{t(uiLang, 'settings.capslock_warnings')}</label>
        <label class="toggle"><input id="setting-capslock" type="checkbox" checked={settings.show_capslock_warnings} onchange={(e) => onUpdateSetting('show_capslock_warnings', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-sound">{t(uiLang, 'settings.sound_enabled')}</label>
        <label class="toggle"><input id="setting-sound" type="checkbox" checked={settings.sound_enabled} onchange={(e) => onUpdateSetting('sound_enabled', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-volume">{t(uiLang, 'settings.sound_volume')}</label>
        <input id="setting-volume" type="range" min="0" max="1" step="0.1" value={settings.sound_volume} onchange={(e) => onUpdateSetting('sound_volume', parseFloat(e.currentTarget.value))} />
      </div>
      <div class="setting-row">
        <label for="setting-zen">{t(uiLang, 'settings.zen_mode')}</label>
        <label class="toggle"><input id="setting-zen" type="checkbox" checked={settings.zen_mode_enabled} onchange={(e) => onUpdateSetting('zen_mode_enabled', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-blind">{t(uiLang, 'settings.blind_mode')}</label>
        <label class="toggle"><input id="setting-blind" type="checkbox" checked={settings.blind_mode_enabled} onchange={(e) => onUpdateSetting('blind_mode_enabled', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      <div class="setting-row">
        <label for="setting-vim">{t(uiLang, 'settings.vim_mode')}</label>
        <label class="toggle"><input id="setting-vim" type="checkbox" checked={settings.vim_mode} onchange={(e) => onUpdateSetting('vim_mode', e.currentTarget.checked)} /><span class="toggle-slider"></span></label>
      </div>
      {#if settings.vim_mode}
        <div class="vim-hint">
          <span class="vim-key">h</span> <span class="vim-desc">{t(uiLang, 'vim.hint_prev')}</span>
          <span class="vim-key">l</span> <span class="vim-desc">{t(uiLang, 'vim.hint_next')}</span>
          <span class="vim-key">k</span> <span class="vim-desc">{t(uiLang, 'vim.hint_up')}</span>
          <span class="vim-key">j</span> <span class="vim-desc">{t(uiLang, 'vim.hint_down')}</span>
          <span class="vim-key">gg</span> <span class="vim-desc">{t(uiLang, 'vim.hint_top')}</span>
          <span class="vim-key">G</span> <span class="vim-desc">{t(uiLang, 'vim.hint_bottom')}</span>
          <span class="vim-key">r</span> <span class="vim-desc">{t(uiLang, 'vim.hint_restart')}</span>
        </div>
      {/if}
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
    <h3>Updates</h3>
    <div class="update-panel">
      <button class="update-btn" onclick={handleCheckUpdate} disabled={checkingUpdate}>
        {checkingUpdate ? 'Checking...' : 'Check for updates'}
      </button>
      {#if updateVersion}
        <button class="update-btn primary" onclick={handleInstallUpdate}>Install v{updateVersion}</button>
      {/if}
      {#if updateStatus}
        <span class="update-status">{updateStatus}</span>
      {/if}
    </div>
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
          aria-pressed={t2.name === activeTheme}
          style="background: {t2.preview_colors.bg}; border-color: {t2.preview_colors.main};"
          onclick={() => onSelectTheme(t2.name)}
        >
          <span style="color: {t2.preview_colors.main}">{t2.display_name}</span>
          <span class="theme-description" style="color: {t2.preview_colors.text}">{themeDescriptions[t2.name]}</span>
          <span class="theme-state-preview">
            <span style="color: {t2.preview_colors.text}">Aa</span>
            <span style="color: {t2.preview_colors.main}; border-left-color: {t2.preview_colors.main}">caret</span>
            <span style="color: {t2.preview_colors.error}; text-decoration-color: {t2.preview_colors.error}">error</span>
          </span>
          {#if t2.name === activeTheme}<span class="selected-label" style="color: {t2.preview_colors.main}">✓ Selected</span>{/if}
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
  .setting-row select { min-width: 155px; padding-right: 0.5rem; }
  .setting-row select option { background-color: var(--bg-sub); color: var(--text); }
  /* Custom toggle switch (iOS/macOS style). The real checkbox is visually
     hidden but remains focusable for keyboard access. */
  .toggle { position: relative; display: inline-block; width: 36px; height: 20px; flex-shrink: 0; cursor: pointer; }
  .toggle input { opacity: 0; width: 0; height: 0; position: absolute; }
  .toggle-slider {
    position: absolute; inset: 0; background: var(--sub); border-radius: 20px;
    transition: background 0.2s ease;
  }
  .toggle-slider::before {
    content: ''; position: absolute; width: 14px; height: 14px; left: 3px; top: 3px;
    background: #fff; border-radius: 50%; transition: transform 0.2s ease;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
  }
  .toggle input:checked + .toggle-slider { background: var(--main); }
  .toggle input:checked + .toggle-slider::before { transform: translateX(16px); }
  .toggle input:focus-visible + .toggle-slider { outline: 2px solid var(--color-focus-ring); outline-offset: 2px; }
  .toggle input:disabled + .toggle-slider { opacity: 0.5; cursor: default; }
  .vim-hint { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-left: 180px; padding: 0.5rem 0.75rem; background: var(--bg-sub); border: 1px solid var(--sub); border-radius: 6px; font-size: 0.8rem; }
  .vim-key { display: inline-block; min-width: 1.4em; text-align: center; padding: 0.1rem 0.35rem; background: var(--main); color: var(--bg); border-radius: 4px; font-family: monospace; font-weight: 700; }
  .vim-desc { color: var(--sub); }
  .setting-row input[type='range'] { accent-color: var(--main); background: transparent !important; }
  .setting-row input:focus-visible, .setting-row select:focus-visible, .theme-search:focus-visible { outline: 2px solid var(--color-focus-ring); outline-offset: 2px; border-color: var(--color-focus-ring); }
  .theme-toolbar { display: flex; align-items: center; gap: 1rem; margin-bottom: 0.75rem; }
  .theme-toolbar span { color: var(--sub); font-size: 0.7rem; }
  .theme-search { min-width: 260px; padding: 0.5rem 0.75rem; border: 1px solid var(--sub); border-radius: 6px; background: var(--bg-sub); color: var(--text); font: inherit; font-size: 0.75rem; }
  .theme-search:focus-visible { border-color: var(--color-focus-ring); }
  .theme-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(145px, 1fr)); gap: 0.65rem; max-height: 460px; overflow-y: auto; padding: 0.25rem; scrollbar-color: var(--sub) var(--bg-sub); }
  .theme-card {
    padding: 0.75rem; border-radius: 8px; border: 2px solid transparent; cursor: pointer;
    display: flex; flex-direction: column; gap: 0.25rem; min-width: 0;
    font-family: inherit; text-align: left;
  }
  .theme-card.active { border-color: var(--main); }
  .theme-card:focus-visible { outline: 2px solid var(--color-focus-ring); outline-offset: 2px; }
  .theme-description { min-height: 2.7em; font-size: 0.62rem; line-height: 1.35; opacity: 0.85; }
  .theme-state-preview { display: flex; align-items: center; gap: 0.5rem; font-size: 0.65rem; }
  .theme-state-preview span:nth-child(2) { border-left: 2px solid; padding-left: 0.25rem; }
  .theme-state-preview span:last-child { text-decoration: underline 2px; text-underline-offset: 0.16em; }
  .selected-label { font-size: 0.62rem; font-weight: 700; }
  .update-panel { display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem; margin-bottom: 1rem; }
  .update-btn {
    background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main);
    padding: 0.5rem 1.25rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px;
  }
  .update-btn.primary { background-color: var(--main); color: var(--bg); }
  .update-btn:hover:not(:disabled) { opacity: 0.85; }
  .update-btn:disabled { opacity: 0.5; cursor: default; }
  .update-status { color: var(--sub); font-size: 0.8rem; }
</style>
