<script lang="ts">
  import type { CustomText, LanguageCode } from '../lib/types/index';
  import { t, UI_LANGUAGES } from '../lib/i18n';
  import { open } from '@tauri-apps/plugin-dialog';
  import { readTextFile } from '@tauri-apps/plugin-fs';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import * as ipc from '../lib/api/ipc';
  import { formatForFile, summarizePlan } from '../lib/textPack';
  import type { TextPackImportPlan, TextPackPolicy } from '../lib/types/index';

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
  let packBusy = $state(false);
  let packMessage = $state('');
  let packError = $state('');
  let packPolicy = $state<TextPackPolicy>('merge');
  let packReplaceAck = $state(false);
  let packPreview = $state<TextPackImportPlan | null>(null);
  let packDocument = $state('');
  let packSourceFormat = $state<string | null>(null);

  const canApplyPack = $derived(
    packPreview !== null && !packBusy && (packPolicy !== 'replace' || packReplaceAck),
  );

  function resetPackState() {
    packMessage = '';
    packError = '';
    packPreview = null;
    packDocument = '';
    packSourceFormat = null;
    packReplaceAck = false;
  }

  async function exportCurrentPack() {
    packMessage = '';
    packError = '';
    try {
      const language = customTexts[0]?.language ?? newLanguage;
      const exported = await ipc.exportTextPack(language);
      const blob = new Blob([exported], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `racoon-typper-texts-${language}-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
      packMessage = t(uiLang, 'textpack.exported');
    } catch (e) {
      packError = ipc.ipcErrorMessage(e);
    }
  }

  async function choosePackFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    packMessage = '';
    packError = '';
    packPreview = null;
    try {
      packDocument = await file.text();
      packSourceFormat = formatForFile(file.name);
    } catch (e) {
      packError = String(e);
    } finally {
      input.value = '';
    }
  }

  async function previewPack() {
    packMessage = '';
    packError = '';
    if (!packDocument) {
      packError = t(uiLang, 'textpack.choose_file');
      return;
    }
    try {
      packPreview = await ipc.previewTextPackImport(packDocument, packSourceFormat, packPolicy);
    } catch (e) {
      packPreview = null;
      packError = ipc.ipcErrorMessage(e);
    }
  }

  async function applyPack() {
    if (!canApplyPack || !packDocument) return;
    packBusy = true;
    packMessage = '';
    packError = '';
    try {
      const applied = await ipc.importTextPack(packDocument, packSourceFormat, packPolicy);
      packMessage = `${t(uiLang, 'textpack.applied')} ${summarizePlan(applied)}`;
      packPreview = null;
      packDocument = '';
      packReplaceAck = false;
      onSearch(searchText);
    } catch (e) {
      packError = ipc.ipcErrorMessage(e);
    } finally {
      packBusy = false;
    }
  }

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

  <section class="pack-panel">
    <h3>{t(uiLang, 'textpack.title')}</h3>
    <p class="pack-hint">{t(uiLang, 'textpack.hint')}</p>
    <div class="pack-controls">
      <button class="pack-btn" onclick={exportCurrentPack} disabled={packBusy}>
        {t(uiLang, 'textpack.export')}
      </button>
      <label class="pack-file">
        <input type="file" accept=".json,.csv,.tsv,.txt,.md" onchange={choosePackFile} />
      </label>
      <select
        class="pack-select"
        value={packPolicy}
        onchange={(e) => { packPolicy = e.currentTarget.value as TextPackPolicy; packPreview = null; }}
      >
        <option value="merge">{t(uiLang, 'textpack.merge')}</option>
        <option value="replace">{t(uiLang, 'textpack.replace')}</option>
      </select>
      <button class="pack-btn" onclick={previewPack} disabled={packBusy || !packDocument}>
        {t(uiLang, 'textpack.preview')}
      </button>
      <button class="pack-btn primary" onclick={applyPack} disabled={!canApplyPack}>
        {t(uiLang, 'textpack.apply')}
      </button>
    </div>
    {#if packPolicy === 'replace'}
      <label class="pack-ack">
        <input type="checkbox" bind:checked={packReplaceAck} />
        {t(uiLang, 'textpack.replace_ack')}
      </label>
    {/if}
    {#if packPreview}
      <p class="pack-plan">
        {t(uiLang, 'textpack.plan_language')}: {packPreview.language}
        · {t(uiLang, 'textpack.plan_incoming')}: {packPreview.incoming}
        · +{packPreview.to_insert}
        · ~{packPreview.to_skip}
        · −{packPreview.removed_by_replace}
      </p>
    {/if}
    {#if packMessage}<p class="pack-message">{packMessage}</p>{/if}
    {#if packError}<p class="pack-error">{packError}</p>{/if}
  </section>
</div>

<style>
  .list-view { max-width: 900px; width: 100%; }
  .pack-panel {
    margin-top: 2rem; padding: 1rem; background: var(--bg-sub);
    border: 1px solid var(--sub); border-radius: 8px;
    display: flex; flex-direction: column; gap: 0.5rem;
  }
  .pack-panel h3 { margin: 0; font-size: 1rem; color: var(--main); }
  .pack-hint { margin: 0; font-size: 0.8rem; color: var(--sub); }
  .pack-controls { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; }
  .pack-btn {
    background: var(--bg); color: var(--main); border: 1px solid var(--sub);
    border-radius: 4px; padding: 0.45rem 0.9rem; cursor: pointer; font-family: inherit;
  }
  .pack-btn:hover:not(:disabled) { border-color: var(--main); }
  .pack-btn.primary { background: var(--main); color: var(--color-accent-text); border-color: var(--main); }
  .pack-btn:disabled { opacity: 0.4; cursor: default; }
  .pack-file input { color: var(--text); font-family: inherit; max-width: 15rem; }
  .pack-select {
    background: var(--bg); color: var(--text); border: 1px solid var(--sub);
    border-radius: 4px; padding: 0.45rem; font-family: inherit;
  }
  .pack-ack { display: flex; align-items: center; gap: 0.4rem; font-size: 0.8rem; color: var(--text); }
  .pack-plan { margin: 0; font-size: 0.85rem; color: var(--text); }
  .pack-message { margin: 0; font-size: 0.85rem; color: #6c8; }
  .pack-error { margin: 0; font-size: 0.85rem; color: var(--error); }
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
