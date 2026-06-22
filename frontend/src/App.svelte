<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  const TEST_TEXT = 'The quick brown fox jumps over the lazy dog';

  // Navigation
  let view = $state<'test' | 'history' | 'bests' | 'custom' | 'settings'>('test');

  // Test state
  let text = $state('');
  let caretPos = $state(0);
  let charStatuses = $state<CharStatus[]>([]);
  let isRunning = $state(false);
  let isComplete = $state(false);
  let errorMsg = $state('');
  let liveWpm = $state(0);
  let liveAccuracy = $state(100);
  let elapsedMs = $state(0);
  let finalStats = $state<FinalStats | null>(null);

  // History
  let history = $state<TestSummary[]>([]);
  let historyTotal = $state(0);

  // Bests
  let bests = $state<PersonalBest[]>([]);

  // Custom texts
  let customTexts = $state<CustomText[]>([]);
  let editingText = $state<CustomText | null>(null);
  let newName = $state('');
  let newTextContent = $state('');
  let showEditor = $state(false);
  let searchText = $state('');

  // Settings
  let settings = $state<AppSettings | null>(null);

  // Themes
  let themes = $state<ThemeInfo[]>([]);
  let activeTheme = $state('serika_dark');
  let appliedTheme = $state('');

  interface CharStatus { expected: string; typed: string | null; status: string; }
  interface TestSessionResponse { session_id: string; text: string; text_length: number; }
  interface EngineOutput {
    key_result: string; caret_pos: number;
    live_stats: { wpm: number; accuracy: number; elapsed_ms: number } | null;
    test_complete: FinalStats | null;
  }
  interface FinalStats {
    wpm: number; raw_wpm: number; accuracy: number; raw_accuracy: number;
    correct_chars: number; incorrect_chars: number; backspaces: number; duration_ms: number;
  }
  interface TestSummary { id: number; created_at: string; mode_type: string; wpm: number; accuracy: number; duration_ms: number; is_pb: boolean; }
  interface StatsHistoryResponse { tests: TestSummary[]; total: number; }
  interface PersonalBest { mode_type: string; best_wpm: number; best_accuracy: number; updated_at: string; }
  interface CustomText { id: number; name: string; text: string; created_at: string; use_count: number; }
  interface AppSettings { theme: string; font_size: number; caret_style: string; show_live_wpm: boolean; show_accuracy: boolean; }
  interface ThemeInfo { name: string; display_name: string; is_dark: boolean; preview_colors: { bg: string; main: string; text: string; error: string; }; }

  async function startTest(testText: string = TEST_TEXT) {
    errorMsg = ''; finalStats = null;
    const resp = await invoke<TestSessionResponse>('start_test', { text: testText || TEST_TEXT });
    text = resp.text; caretPos = 0; isComplete = false; isRunning = true;
    liveWpm = 0; liveAccuracy = 100; elapsedMs = 0;
    charStatuses = text.split('').map((ch) => ({ expected: ch, typed: null, status: 'pending' }));
  }

  async function handleKeydown(e: KeyboardEvent) {
    if (!isRunning || isComplete) return;
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    if (e.key === 'Backspace' || e.key === 'Tab' || e.key === ' ' || e.key.length === 1) e.preventDefault();
    try {
      const output = await invoke<EngineOutput>('process_key', { key: e.key, code: e.code, timestamp: performance.now() });
      caretPos = output.caret_pos;
      if (output.live_stats) { liveWpm = output.live_stats.wpm; liveAccuracy = output.live_stats.accuracy; elapsedMs = output.live_stats.elapsed_ms; }
      if (output.key_result === 'correct' && caretPos > 0) charStatuses[caretPos - 1] = { ...charStatuses[caretPos - 1], typed: charStatuses[caretPos - 1].expected, status: 'correct' };
      else if (output.key_result === 'incorrect' && caretPos < charStatuses.length) charStatuses[caretPos] = { ...charStatuses[caretPos], typed: e.key, status: 'incorrect' };
      else if (output.key_result === 'undone_correct' && caretPos < charStatuses.length) charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'backspaced' };
      else if (output.key_result === 'undone_incorrect' && caretPos < charStatuses.length) charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'pending' };
      if (output.test_complete) { finalStats = output.test_complete; isComplete = true; isRunning = false; }
    } catch (e) { errorMsg = `Error: ${e}`; }
  }

  function abortTest() { if (isRunning) invoke('abort_session').catch(() => {}); isRunning = false; isComplete = false; caretPos = 0; charStatuses = []; }

  async function loadHistory() { const r = await invoke<StatsHistoryResponse>('get_stats_history', { limit: 20 }); history = r.tests; historyTotal = r.total; }
  async function loadBests() { bests = await invoke<PersonalBest[]>('get_personal_bests', {}); }
  async function loadCustomTexts() { customTexts = await invoke<CustomText[]>('get_custom_texts', { limit: 50 }); }
  async function loadSettings() { settings = await invoke<AppSettings>('get_settings'); activeTheme = settings.theme; await applyTheme(activeTheme); }
  async function loadThemes() { themes = await invoke<ThemeInfo[]>('get_themes'); }

  async function applyTheme(name: string) {
    const css = await invoke<string>('get_theme_css', { name });
    // Parse CSS variables and apply
    const styleEl = document.getElementById('theme-style') || (() => { const el = document.createElement('style'); el.id = 'theme-style'; document.head.appendChild(el); return el; })();
    styleEl.textContent = css;
    appliedTheme = name;
  }

  async function selectTheme(name: string) {
    activeTheme = name;
    await invoke('set_setting', { key: 'theme', value: name });
    await applyTheme(name);
    settings = await invoke<AppSettings>('get_settings');
  }

  async function updateSetting(key: string, value: any) {
    await invoke('set_setting', { key, value });
    settings = await invoke<AppSettings>('get_settings');
  }

  function openEditor(ct: CustomText | null = null) {
    editingText = ct;
    newName = ct ? ct.name : '';
    newTextContent = ct ? ct.text : '';
    showEditor = true;
  }

  async function saveCustomText() {
    try {
      if (editingText) { await invoke('update_custom_text', { id: editingText.id, name: newName, text: newTextContent }); }
      else { await invoke('save_custom_text', { name: newName, text: newTextContent }); }
      showEditor = false; await loadCustomTexts();
    } catch (e) { errorMsg = `Save error: ${e}`; }
  }

  async function deleteCustomText(id: number) {
    await invoke('delete_custom_text', { id }); await loadCustomTexts();
  }

  async function startCustomTest(id: number) {
    const resp = await invoke<TestSessionResponse>('start_custom_text_test', { customTextId: id });
    text = resp.text; caretPos = 0; isComplete = false; isRunning = true; finalStats = null;
    charStatuses = text.split('').map((ch) => ({ expected: ch, typed: null, status: 'pending' }));
    view = 'test';
  }

  async function searchCustom() {
    if (searchText.trim()) { customTexts = await invoke<CustomText[]>('search_custom_texts', { query: searchText, limit: 20 }); }
    else { await loadCustomTexts(); }
  }

  function switchView(v: typeof view) {
    view = v;
    if (v === 'history') loadHistory();
    if (v === 'bests') loadBests();
    if (v === 'custom') loadCustomTexts();
  }

  function formatDate(iso: string): string { try { return new Date(iso).toLocaleString(); } catch { return iso; } }

  onMount(async () => { await loadThemes(); await loadSettings(); await startTest(); });
</script>

<svelte:window on:keydown={handleKeydown} />

<main>
  <nav>
    <button class:active={view === 'test'} onclick={() => switchView('test')}>Test</button>
    <button class:active={view === 'history'} onclick={() => switchView('history')}>History ({historyTotal})</button>
    <button class:active={view === 'bests'} onclick={() => switchView('bests')}>Bests</button>
    <button class:active={view === 'custom'} onclick={() => switchView('custom')}>Texts</button>
    <button class:active={view === 'settings'} onclick={() => switchView('settings')}>Settings</button>
  </nav>

  {#if errorMsg}<p class="error">{errorMsg}</p>{/if}

  {#if view === 'test'}
    {#if isComplete && finalStats}
      <div class="result-overlay">
        <h2>Test Complete</h2>
        <div class="stats-grid">
          <div class="stat-box"><span class="stat-value">{finalStats.wpm.toFixed(1)}</span><span class="stat-label">WPM</span></div>
          <div class="stat-box"><span class="stat-value">{finalStats.raw_wpm.toFixed(1)}</span><span class="stat-label">Raw WPM</span></div>
          <div class="stat-box"><span class="stat-value">{finalStats.accuracy.toFixed(1)}%</span><span class="stat-label">Accuracy</span></div>
          <div class="stat-box"><span class="stat-value">{finalStats.raw_accuracy.toFixed(1)}%</span><span class="stat-label">Raw Acc</span></div>
        </div>
        <div class="stats-details"><span>Correct: {finalStats.correct_chars}</span><span>Incorrect: {finalStats.incorrect_chars}</span><span>Backspaces: {finalStats.backspaces}</span><span>Duration: {(finalStats.duration_ms / 1000).toFixed(1)}s</span></div>
        <button onclick={() => startTest()}>Restart</button>
      </div>
    {:else if text}
      <div class="live-stats">
        {#if settings?.show_live_wpm}<span class="stat">WPM: {liveWpm.toFixed(0)}</span>{/if}
        {#if settings?.show_accuracy}<span class="stat">Acc: {liveAccuracy.toFixed(1)}%</span>{/if}
        <span class="stat">Time: {(elapsedMs / 1000).toFixed(1)}s</span>
      </div>
      <div class="text-display" tabindex="0">
        {#each charStatuses as char, i}<span class="char {char.status}" class:caret={i === caretPos}>{char.expected === ' ' ? '\u00A0' : char.expected}</span>{/each}
      </div>
      <div class="info"><span>Position: {caretPos}/{text.length}</span><button class="abort-btn" onclick={abortTest}>Abort</button></div>
    {/if}
  {:else if view === 'history'}
    <div class="list-view"><h2>Test History ({historyTotal})</h2>
      {#if history.length === 0}<p class="empty">No tests yet.</p>{:else}
        <table><thead><tr><th>Date</th><th>Mode</th><th>WPM</th><th>Accuracy</th><th>Duration</th><th>PB</th></tr></thead>
        <tbody>{#each history as t}<tr><td>{formatDate(t.created_at)}</td><td>{t.mode_type}</td><td>{t.wpm.toFixed(1)}</td><td>{t.accuracy.toFixed(1)}%</td><td>{(t.duration_ms / 1000).toFixed(1)}s</td><td>{t.is_pb ? '★' : ''}</td></tr>{/each}</tbody></table>
      {/if}
    </div>
  {:else if view === 'bests'}
    <div class="list-view"><h2>Personal Bests</h2>
      {#if bests.length === 0}<p class="empty">No records yet.</p>{:else}
        <table><thead><tr><th>Mode</th><th>Best WPM</th><th>Best Accuracy</th><th>Updated</th></tr></thead>
        <tbody>{#each bests as b}<tr><td>{b.mode_type}</td><td>{b.best_wpm.toFixed(1)}</td><td>{b.best_accuracy.toFixed(1)}%</td><td>{formatDate(b.updated_at)}</td></tr>{/each}</tbody></table>
      {/if}
    </div>
  {:else if view === 'custom'}
    <div class="list-view">
      <h2>Custom Texts</h2>
      <div class="custom-actions"><input type="text" placeholder="Search..." bind:value={searchText} oninput={searchCustom} /><button onclick={() => openEditor()}>+ New</button></div>
      {#if showEditor}
        <div class="editor"><input type="text" placeholder="Name" bind:value={newName} /><textarea placeholder="Text content" bind:value={newTextContent} rows="5"></textarea><div class="editor-btns"><button onclick={saveCustomText}>Save</button><button class="abort-btn" onclick={() => showEditor = false}>Cancel</button></div></div>
      {/if}
      {#if customTexts.length === 0}<p class="empty">No custom texts. Create one!</p>{:else}
        <div class="text-cards">{#each customTexts as ct}<div class="text-card"><h3>{ct.name}</h3><p class="text-preview">{ct.text.substring(0, 80)}{ct.text.length > 80 ? '...' : ''}</p><div class="card-actions"><span class="use-count">Used: {ct.use_count}</span><button onclick={() => startCustomTest(ct.id)}>Start</button><button onclick={() => openEditor(ct)}>Edit</button><button class="abort-btn" onclick={() => deleteCustomText(ct.id)}>Delete</button></div></div>{/each}</div>
      {/if}
    </div>
  {:else if view === 'settings'}
    <div class="list-view"><h2>Settings</h2>
      {#if settings}
        <div class="settings-form">
          <div class="setting-row"><label>Theme</label><select value={settings.theme} onchange={(e) => selectTheme(e.currentTarget.value)}>{#each themes as t}<option value={t.name} selected={t.name === settings.theme}>{t.display_name}</option>{/each}</select></div>
          <div class="setting-row"><label>Font Size</label><input type="number" value={settings.font_size} onchange={(e) => updateSetting('font_size', parseInt(e.currentTarget.value))} /></div>
          <div class="setting-row"><label>Caret Style</label><select value={settings.caret_style} onchange={(e) => updateSetting('caret_style', e.currentTarget.value)}><option value="underline">Underline</option><option value="block">Block</option><option value="solid">Solid</option><option value="off">Off</option></select></div>
          <div class="setting-row"><label>Show Live WPM</label><input type="checkbox" checked={settings.show_live_wpm} onchange={(e) => updateSetting('show_live_wpm', e.currentTarget.checked)} /></div>
          <div class="setting-row"><label>Show Accuracy</label><input type="checkbox" checked={settings.show_accuracy} onchange={(e) => updateSetting('show_accuracy', e.currentTarget.checked)} /></div>
        </div>
        <h3>Theme Preview</h3>
        <div class="theme-cards">{#each themes as t}<div class="theme-card {t.name === activeTheme ? 'active' : ''}" style="background: {t.preview_colors.bg}; border-color: {t.preview_colors.main};" onclick={() => selectTheme(t.name)}><span style="color: {t.preview_colors.main}">{t.display_name}</span><span style="color: {t.preview_colors.text}">Sample text</span><span style="color: {t.preview_colors.error}">error</span></div>{/each}</div>
      {/if}
    </div>
  {/if}
</main>

<style>
  :root { --bg: #323437; --bg-sub: #2c2e31; --main: #e2b714; --sub: #555555; --text: #999999; --error: #ca4754; --caret: #e2b714; }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  main { display: flex; flex-direction: column; align-items: center; min-height: 100vh; gap: 1.5rem; padding: 1rem; background-color: var(--bg); color: var(--text); font-family: 'JetBrains Mono', monospace; font-size: 24px; }
  nav { display: flex; gap: 0.5rem; }
  nav button { background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub); padding: 0.5rem 1rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px; }
  nav button.active { color: var(--main); border-color: var(--main); }
  h1 { color: var(--main); } h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; } h3 { color: var(--main); font-size: 1.1rem; margin: 1rem 0 0.5rem; }
  .live-stats { display: flex; gap: 2rem; font-size: 1.25rem; } .stat { color: var(--sub); }
  .text-display { font-size: 2rem; line-height: 1.8; max-width: 900px; text-align: center; padding: 2rem; background-color: var(--bg-sub); border-radius: 8px; user-select: none; }
  .char { transition: color 0.05s; position: relative; } .char.pending { color: var(--sub); } .char.correct { color: var(--text); } .char.incorrect { color: var(--error); } .char.backspaced { color: var(--sub); }
  .char.caret::before { content: '|'; position: absolute; left: -0.5ch; color: var(--caret); animation: blink 1s infinite; }
  @keyframes blink { 0%,50%{opacity:1} 51%,100%{opacity:0} }
  .info { display: flex; align-items: center; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  .result-overlay { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1.5rem; }
  .stat-box { display: flex; flex-direction: column; gap: 0.25rem; padding: 1.5rem 2rem; background-color: var(--bg-sub); border-radius: 8px; }
  .stat-value { font-size: 2rem; color: var(--main); } .stat-label { font-size: 0.75rem; color: var(--sub); text-transform: uppercase; }
  .stats-details { display: flex; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  button { background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main); padding: 0.5rem 1.5rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px; }
  button:hover { background-color: var(--main); color: var(--bg); }
  .abort-btn { border-color: var(--sub); color: var(--sub); } .abort-btn:hover { background: var(--sub); color: var(--bg); }
  .error { color: var(--error); font-size: 0.875rem; } .empty { color: var(--sub); text-align: center; padding: 2rem; }
  .list-view { max-width: 900px; width: 100%; }
  table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
  th { text-align: left; color: var(--main); padding: 0.5rem; border-bottom: 1px solid var(--bg-sub); }
  td { padding: 0.5rem; color: var(--text); border-bottom: 1px solid var(--bg-sub); }
  .custom-actions { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  .custom-actions input { flex: 1; background: var(--bg-sub); border: 1px solid var(--sub); color: var(--text); padding: 0.5rem; font-family: inherit; border-radius: 4px; }
  .editor { background: var(--bg-sub); padding: 1rem; border-radius: 8px; margin-bottom: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .editor input, .editor textarea { background: var(--bg); border: 1px solid var(--sub); color: var(--text); padding: 0.5rem; font-family: inherit; border-radius: 4px; font-size: 0.875rem; }
  .editor-btns { display: flex; gap: 0.5rem; }
  .text-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 1rem; }
  .text-card { background: var(--bg-sub); padding: 1rem; border-radius: 8px; }
  .text-card h3 { color: var(--main); font-size: 1rem; margin: 0 0 0.5rem; }
  .text-preview { color: var(--sub); font-size: 0.75rem; margin-bottom: 0.5rem; }
  .card-actions { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .use-count { font-size: 0.75rem; color: var(--sub); margin-right: auto; }
  .card-actions button { font-size: 0.75rem; padding: 0.25rem 0.75rem; }
  .settings-form { display: flex; flex-direction: column; gap: 1rem; }
  .setting-row { display: flex; align-items: center; gap: 1rem; }
  .setting-row label { min-width: 150px; color: var(--sub); font-size: 0.875rem; }
  .setting-row input, .setting-row select { background: var(--bg-sub); border: 1px solid var(--sub); color: var(--text); padding: 0.5rem; font-family: inherit; border-radius: 4px; font-size: 0.875rem; }
  .theme-cards { display: flex; gap: 1rem; flex-wrap: wrap; }
  .theme-card { padding: 1rem; border-radius: 8px; border: 2px solid transparent; cursor: pointer; display: flex; flex-direction: column; gap: 0.25rem; min-width: 120px; }
  .theme-card.active { border-color: var(--main); }
</style>