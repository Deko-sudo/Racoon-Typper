<script lang="ts">
  import type { CustomText, LanguageCode } from '../lib/types/index';
  import { t, UI_LANGUAGES } from '../lib/i18n';
  import { open } from '@tauri-apps/plugin-dialog';
  import { readTextFile } from '@tauri-apps/plugin-fs';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import * as ipc from '../lib/api/ipc';

  let {
    customTexts,
    searchText,
    showEditor,
    newName,
    newTextContent,
    newLanguage,
    onSave,
    onDelete,
    onStart,
    onSearch,
    onOpenEditor,
    onCloseEditor,
    onNameChange,
    onTextChange,
    onLanguageChange,
    uiLang = 'en',
  }: {
    customTexts: CustomText[];
    searchText: string;
    showEditor: boolean;
    newName: string;
    newTextContent: string;
    newLanguage: LanguageCode;
    onSave: () => void;
    onDelete: (id: number) => void;
    onStart: (id: number) => void;
    onSearch: (q: string) => void;
    onOpenEditor: (ct: CustomText | null) => void;
    onCloseEditor: () => void;
    onNameChange: (name: string) => void;
    onTextChange: (text: string) => void;
    onLanguageChange: (language: LanguageCode) => void;
    uiLang?: string;
  } = $props();

  let importError = $state('');

  async function importFromFile() {
    importError = '';
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Text', extensions: ['txt', 'md', 'text'] }],
      });
      if (typeof selected !== 'string' || !selected) return;
      const content = await readTextFile(selected);
      if (!content.trim()) {
        importError = t(uiLang, 'custom.import_error_empty');
        return;
      }
      onTextChange(content);
    } catch (e) {
      importError = `${t(uiLang, 'custom.import_error_failed')} ${e}`;
    }
  }

  async function importFromClipboard() {
    importError = '';
    try {
      const content = await readText();
      if (!content || !content.trim()) {
        importError = t(uiLang, 'custom.import_error_empty');
        return;
      }
      onTextChange(content);
    } catch (e) {
      importError = `${t(uiLang, 'custom.import_error_failed')} ${e}`;
    }
  }

  async function importFromUrl() {
    importError = '';
    const url = window.prompt(t(uiLang, 'custom.import_url_prompt'), 'https://');
    if (!url) return;
    try {
      const text = await ipc.importTextFromUrl(url);
      onTextChange(text);
    } catch (e) {
      importError = `${t(uiLang, 'custom.import_error_failed')} ${e}`;
    }
  }
</script>

<div class="list-view">
  <h2>{t(uiLang, 'custom.title')}</h2>
  <div class="custom-actions">
    <input type="text" placeholder={t(uiLang, 'custom.search')} value={searchText} oninput={(e) => onSearch(e.currentTarget.value)} />
    <button onclick={() => onOpenEditor(null)}>+ {t(uiLang, 'custom.create')}</button>
  </div>
  {#if showEditor}
    <div class="editor">
      <input type="text" placeholder={t(uiLang, 'custom.name')} value={newName} oninput={(e) => onNameChange(e.currentTarget.value)} />
      <select value={newLanguage} onchange={(e) => onLanguageChange(e.currentTarget.value as LanguageCode)} aria-label="Text language">
        {#each UI_LANGUAGES as [code, name]}
          <option value={code}>{name}</option>
        {/each}
      </select>
      <textarea placeholder={t(uiLang, 'custom.text')} value={newTextContent} oninput={(e) => onTextChange(e.currentTarget.value)} rows="5"></textarea>
      {#if importError}<p class="import-error">{importError}</p>{/if}
      <div class="import-row">
        <button class="import-btn" onclick={importFromFile}>{t(uiLang, 'custom.import_file')}</button>
        <button class="import-btn" onclick={importFromClipboard}>{t(uiLang, 'custom.import_clipboard')}</button>
        <button class="import-btn" onclick={importFromUrl}>{t(uiLang, 'custom.import_url')}</button>
      </div>
      <div class="editor-btns">
        <button onclick={onSave}>{t(uiLang, 'custom.save')}</button>
        <button class="abort-btn" onclick={onCloseEditor}>{t(uiLang, 'custom.cancel')}</button>
      </div>
    </div>
  {/if}
  {#if customTexts.length === 0}
    <p class="empty">{t(uiLang, 'custom.empty')}</p>
  {:else}
    <div class="text-cards">
      {#each customTexts as ct}
        <div class="text-card">
          <h3>{ct.name} <span class="language">{ct.language.toUpperCase()}</span></h3>
          <p class="text-preview">{ct.text.substring(0, 80)}{ct.text.length > 80 ? '...' : ''}</p>
          <div class="card-actions">
            <span class="use-count">{t(uiLang, 'custom.used')}: {ct.use_count}</span>
            <button onclick={() => onStart(ct.id)}>{t(uiLang, 'custom.start')}</button>
            <button onclick={() => onOpenEditor(ct)}>{t(uiLang, 'custom.edit')}</button>
            <button class="abort-btn" onclick={() => onDelete(ct.id)}>{t(uiLang, 'custom.delete')}</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .list-view { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  .custom-actions { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  .custom-actions input { flex: 1; background: var(--bg-sub); border: 1px solid var(--sub); color: var(--text); padding: 0.5rem; font-family: inherit; border-radius: 4px; }
  .editor { background: var(--bg-sub); padding: 1rem; border-radius: 8px; margin-bottom: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .editor input, .editor textarea, .editor select { background: var(--bg); border: 1px solid var(--sub); color: var(--text); padding: 0.5rem; font-family: inherit; border-radius: 4px; font-size: 0.875rem; }
  .editor-btns { display: flex; gap: 0.5rem; }
  .import-row { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .import-btn { font-size: 0.75rem; padding: 0.3rem 0.75rem; border-style: dashed; border-color: var(--sub); color: var(--sub); background: transparent; }
  .import-btn:hover { color: var(--main); border-color: var(--main); background: transparent; }
  .import-error { color: var(--error, #c64850); font-size: 0.75rem; margin: 0; }
  .text-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 1rem; }
  .text-card { background: var(--bg-sub); padding: 1rem; border-radius: 8px; }
  .text-card h3 { color: var(--main); font-size: 1rem; margin: 0 0 0.5rem; }
  .language { color: var(--sub); font-size: 0.6rem; }
  .text-preview { color: var(--sub); font-size: 0.75rem; margin-bottom: 0.5rem; }
  .card-actions { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .use-count { font-size: 0.75rem; color: var(--sub); margin-right: auto; }
  .card-actions button { font-size: 0.75rem; padding: 0.25rem 0.75rem; }
  .abort-btn { border-color: var(--sub); color: var(--sub); }
  button { background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main); padding: 0.5rem 1.5rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px; }
  button:hover { background-color: var(--main); color: var(--color-accent-text); }
</style>
