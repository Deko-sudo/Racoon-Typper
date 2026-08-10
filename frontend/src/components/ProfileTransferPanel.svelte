<script lang="ts">
  import * as ipc from '../lib/api/ipc';
  import { t } from '../lib/i18n';
  import { profileImportRows, validateProfileFileMetadata } from '../lib/profileTransfer';
  import type { ProfileImportPlan, ProfileImportPolicy } from '../lib/types/index';

  let { uiLang = 'en' }: { uiLang?: string } = $props();

  let policy = $state<ProfileImportPolicy>('merge');
  let fileName = $state('');
  let profileDocument = $state('');
  let previewedDocument = $state('');
  let previewedPolicy = $state<ProfileImportPolicy | null>(null);
  let plan = $state<ProfileImportPlan | null>(null);
  let replaceAcknowledged = $state(false);
  let isExporting = $state(false);
  let isReadingFile = $state(false);
  let isPreviewing = $state(false);
  let isApplying = $state(false);
  let errorMessage = $state('');
  let successMessage = $state('');
  let fileRevision = 0;
  let previewRevision = 0;

  let rows = $derived(plan ? profileImportRows(plan) : []);
  let hasCurrentPreview = $derived(
    plan !== null
      && previewedDocument === profileDocument
      && previewedPolicy === policy,
  );
  let canApply = $derived(
    hasCurrentPreview
      && !isApplying
      && (policy !== 'replace' || replaceAcknowledged),
  );

  function clearMessages() {
    errorMessage = '';
    successMessage = '';
  }

  function invalidatePreview() {
    previewRevision += 1;
    plan = null;
    previewedDocument = '';
    previewedPolicy = null;
    replaceAcknowledged = false;
  }

  async function exportCurrentProfile() {
    clearMessages();
    isExporting = true;
    try {
      const exportedProfile = await ipc.exportProfile();
      const blob = new Blob([exportedProfile], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `racoon-typper-profile-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
      successMessage = t(uiLang, 'profile.exported');
    } catch (error) {
      errorMessage = ipc.ipcErrorMessage(error);
    } finally {
      isExporting = false;
    }
  }

  async function selectProfileFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    const revision = ++fileRevision;
    clearMessages();
    invalidatePreview();
    isReadingFile = false;
    fileName = '';
    profileDocument = '';

    if (!file) return;
    fileName = file.name;
    const metadataError = validateProfileFileMetadata(file);
    if (metadataError) {
      errorMessage = t(uiLang, `profile.file_error_${metadataError}`);
      input.value = '';
      return;
    }

    isReadingFile = true;
    try {
      const contents = await file.text();
      if (revision === fileRevision) profileDocument = contents;
    } catch (error) {
      if (revision === fileRevision) errorMessage = ipc.ipcErrorMessage(error);
    } finally {
      if (revision === fileRevision) isReadingFile = false;
      input.value = '';
    }
  }

  function selectPolicy(event: Event) {
    policy = (event.currentTarget as HTMLSelectElement).value as ProfileImportPolicy;
    clearMessages();
    invalidatePreview();
  }

  async function previewRestore() {
    clearMessages();
    invalidatePreview();
    if (!profileDocument) {
      errorMessage = t(uiLang, 'profile.choose_file');
      return;
    }

    const documentToPreview = profileDocument;
    const policyToPreview = policy;
    const requestRevision = previewRevision;
    isPreviewing = true;
    try {
      const result = await ipc.previewProfileImport(documentToPreview, policyToPreview);
      if (
        previewRevision === requestRevision
        && profileDocument === documentToPreview
        && policy === policyToPreview
      ) {
        plan = result;
        previewedDocument = documentToPreview;
        previewedPolicy = policyToPreview;
      }
    } catch (error) {
      if (previewRevision === requestRevision) {
        errorMessage = ipc.ipcErrorMessage(error);
      }
    } finally {
      isPreviewing = false;
    }
  }

  async function applyRestore() {
    clearMessages();
    if (!hasCurrentPreview || !canApply) {
      errorMessage = t(uiLang, 'profile.preview_required');
      return;
    }

    const documentToImport = previewedDocument;
    const policyToImport = previewedPolicy;
    if (!policyToImport) return;

    isApplying = true;
    try {
      await ipc.importProfile(documentToImport, policyToImport);
      successMessage = t(uiLang, 'profile.restored');
      window.setTimeout(() => {
        isApplying = false;
        window.location.reload();
      }, 700);
    } catch (error) {
      errorMessage = ipc.ipcErrorMessage(error);
      isApplying = false;
    }
  }
</script>

<section class="profile-transfer" aria-labelledby="profile-transfer-title">
  <header>
    <h3 id="profile-transfer-title">{t(uiLang, 'profile.title')}</h3>
    <p>{t(uiLang, 'profile.description')}</p>
  </header>

  <button type="button" class="secondary" onclick={exportCurrentProfile} disabled={isExporting || isApplying}>
    {t(uiLang, 'profile.export')}
  </button>

  <div class="restore-controls">
    <label class="field">
      <span>{t(uiLang, 'profile.file')}</span>
      <input
        type="file"
        accept="application/json,.json"
        onchange={selectProfileFile}
        disabled={isApplying}
      />
    </label>

    <label class="field">
      <span>{t(uiLang, 'profile.policy')}</span>
      <select value={policy} onchange={selectPolicy} disabled={isApplying}>
        <option value="merge">{t(uiLang, 'profile.merge')}</option>
        <option value="replace">{t(uiLang, 'profile.replace')}</option>
      </select>
    </label>

    <button
      type="button"
      class="secondary preview-button"
      onclick={previewRestore}
      disabled={!profileDocument || isReadingFile || isPreviewing || isApplying}
    >
      {isPreviewing ? t(uiLang, 'profile.previewing') : t(uiLang, 'profile.preview')}
    </button>
  </div>

  {#if fileName}
    <p class="selected-file">{fileName}</p>
  {/if}

  {#if errorMessage}
    <p class="status error" role="alert">{errorMessage}</p>
  {/if}
  {#if successMessage}
    <p class="status success" role="status">{successMessage}</p>
  {/if}

  {#if plan && hasCurrentPreview}
    <div class="preview" aria-live="polite">
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>{t(uiLang, 'profile.collection')}</th>
              <th>{t(uiLang, 'profile.incoming')}</th>
              <th>{t(uiLang, 'profile.existing')}</th>
              <th>{t(uiLang, 'profile.to_insert')}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as row}
              <tr>
                <td>{t(uiLang, `profile.${row.key}`)}</td>
                <td>{row.incoming}</td>
                <td>{row.existing}</td>
                <td>{row.toInsert}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      {#if policy === 'replace'}
        <div class="replace-warning">
          <p>{t(uiLang, 'profile.replace_warning')}</p>
          <label>
            <input type="checkbox" bind:checked={replaceAcknowledged} disabled={isApplying} />
            <span>{t(uiLang, 'profile.replace_confirm')}</span>
          </label>
        </div>
      {/if}

      <button type="button" class="primary" onclick={applyRestore} disabled={!canApply}>
        {isApplying ? t(uiLang, 'profile.applying') : t(uiLang, 'profile.apply')}
      </button>
    </div>
  {/if}
</section>

<style>
  .profile-transfer {
    margin-top: 1.5rem;
    padding: 1rem;
    border: 1px solid var(--sub);
    border-radius: 8px;
    background: var(--bg-sub);
  }

  header { margin-bottom: 1rem; }
  h3 { margin: 0 0 0.35rem; color: var(--main); font-size: 1.1rem; }
  header p, .selected-file { margin: 0; color: var(--sub); font-size: 0.78rem; line-height: 1.5; }

  button, input, select { font: inherit; }
  button {
    min-height: 2.35rem;
    padding: 0.55rem 0.85rem;
    border-radius: 5px;
    cursor: pointer;
  }
  button:disabled { cursor: not-allowed; color: var(--color-text-disabled); opacity: 0.72; border-style: dashed; }
  button:focus-visible, input:focus-visible, select:focus-visible {
    outline: 2px solid var(--main);
    outline-offset: 2px;
  }
  .primary { border: 1px solid var(--main); background: var(--main); color: var(--color-accent-text); }
  .secondary { border: 1px solid var(--sub); background: var(--bg); color: var(--text); }
  .secondary:hover:not(:disabled) { border-color: var(--main); color: var(--main); }

  .restore-controls {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) minmax(220px, 1fr) auto;
    align-items: end;
    gap: 0.75rem;
    margin-top: 1rem;
  }
  .field { display: flex; flex-direction: column; gap: 0.35rem; min-width: 0; }
  .field > span { color: var(--sub); font-size: 0.72rem; }
  .field input, .field select {
    min-height: 2.35rem;
    max-width: 100%;
    border: 1px solid var(--sub);
    border-radius: 5px;
    background: var(--bg);
    color: var(--text);
    padding: 0.45rem 0.6rem;
  }
  .preview-button { white-space: nowrap; }
  .selected-file { margin-top: 0.6rem; overflow-wrap: anywhere; }

  .status { margin: 0.75rem 0 0; padding: 0.65rem 0.75rem; border: 1px solid currentColor; border-radius: 5px; font-size: 0.75rem; }
  .status.error { color: var(--error); }
  .status.success { color: var(--main); }

  .preview { margin-top: 1rem; }
  .table-wrap { max-width: 100%; overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.72rem; }
  th, td { padding: 0.55rem 0.65rem; border-bottom: 1px solid var(--sub); text-align: right; white-space: nowrap; }
  th:first-child, td:first-child { text-align: left; }
  th { color: var(--sub); font-weight: 500; }
  td { color: var(--text); }

  .replace-warning {
    margin: 1rem 0;
    padding: 0.75rem;
    border: 1px solid var(--error);
    border-radius: 5px;
    color: var(--error);
    font-size: 0.75rem;
  }
  .replace-warning p { margin: 0 0 0.65rem; line-height: 1.5; }
  .replace-warning label { display: flex; align-items: flex-start; gap: 0.55rem; color: var(--text); }
  .replace-warning input { margin-top: 0.15rem; accent-color: var(--error); }

  @media (max-width: 760px) {
    .profile-transfer { padding: 0.85rem; }
    .restore-controls { grid-template-columns: 1fr; }
    .restore-controls button, .primary, .secondary { width: 100%; }
    th, td { padding: 0.5rem; }
  }
</style>
