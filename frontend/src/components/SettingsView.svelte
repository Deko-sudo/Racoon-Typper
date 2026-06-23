<script lang="ts">
  import type { AppSettings, ThemeInfo } from '../types/index';

  let {
    settings,
    themes,
    activeTheme,
    onSelectTheme,
    onUpdateSetting,
  }: {
    settings: AppSettings | null;
    themes: ThemeInfo[];
    activeTheme: string;
    onSelectTheme: (name: string) => void;
    onUpdateSetting: (key: string, value: unknown) => void;
  } = $props();
</script>

<div class="list-view">
  <h2>Settings</h2>
  {#if settings}
    <div class="settings-form">
      <div class="setting-row">
        <label>Theme</label>
        <select value={settings.theme} onchange={(e) => onSelectTheme(e.currentTarget.value)}>
          {#each themes as t}
            <option value={t.name} selected={t.name === settings.theme}>{t.display_name}</option>
          {/each}
        </select>
      </div>
      <div class="setting-row">
        <label>Font Size</label>
        <input type="number" value={settings.font_size} onchange={(e) => onUpdateSetting('font_size', parseInt(e.currentTarget.value))} />
      </div>
      <div class="setting-row">
        <label>Caret Style</label>
        <select value={settings.caret_style} onchange={(e) => onUpdateSetting('caret_style', e.currentTarget.value)}>
          <option value="underline">Underline</option>
          <option value="block">Block</option>
          <option value="solid">Solid</option>
          <option value="off">Off</option>
        </select>
      </div>
      <div class="setting-row">
        <label>Show Live WPM</label>
        <input type="checkbox" checked={settings.show_live_wpm} onchange={(e) => onUpdateSetting('show_live_wpm', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label>Show Accuracy</label>
        <input type="checkbox" checked={settings.show_accuracy} onchange={(e) => onUpdateSetting('show_accuracy', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label>Keyboard Trainer</label>
        <input type="checkbox" checked={settings.show_keyboard_trainer} onchange={(e) => onUpdateSetting('show_keyboard_trainer', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label>Hand Guide</label>
        <input type="checkbox" checked={settings.show_hand_guide} onchange={(e) => onUpdateSetting('show_hand_guide', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label>Layout Warnings</label>
        <input type="checkbox" checked={settings.show_layout_warnings} onchange={(e) => onUpdateSetting('show_layout_warnings', e.currentTarget.checked)} />
      </div>
      <div class="setting-row">
        <label>Caps Lock Warnings</label>
        <input type="checkbox" checked={settings.show_capslock_warnings} onchange={(e) => onUpdateSetting('show_capslock_warnings', e.currentTarget.checked)} />
      </div>
    </div>
    <h3>Theme Preview</h3>
    <div class="theme-cards">
      {#each themes as t}
        <div
          class="theme-card {t.name === activeTheme ? 'active' : ''}"
          style="background: {t.preview_colors.bg}; border-color: {t.preview_colors.main};"
          onclick={() => onSelectTheme(t.name)}
        >
          <span style="color: {t.preview_colors.main}">{t.display_name}</span>
          <span style="color: {t.preview_colors.text}">Sample text</span>
          <span style="color: {t.preview_colors.error}">error</span>
        </div>
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
  .setting-row label { min-width: 150px; color: var(--sub); font-size: 0.875rem; }
  .setting-row input, .setting-row select {
    background: var(--bg-sub); border: 1px solid var(--sub); color: var(--text);
    padding: 0.5rem; font-family: inherit; border-radius: 4px; font-size: 0.875rem;
  }
  .theme-cards { display: flex; gap: 1rem; flex-wrap: wrap; }
  .theme-card {
    padding: 1rem; border-radius: 8px; border: 2px solid transparent; cursor: pointer;
    display: flex; flex-direction: column; gap: 0.25rem; min-width: 120px;
  }
  .theme-card.active { border-color: var(--main); }
</style>