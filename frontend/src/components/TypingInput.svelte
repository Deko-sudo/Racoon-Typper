<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- Copyright 2026 Racoon Typper Contributors -->
<script lang="ts">
  import { typingCharacters } from '../lib/typingInput';
  import { t } from '../lib/i18n';

  let { onKey, onCapsLockChange = () => {}, uiLang = 'en' }: {
    onKey: (key: string, code: string) => void;
    onCapsLockChange?: (enabled: boolean) => void;
    uiLang?: string;
  } = $props();

  let input: HTMLTextAreaElement | null = $state(null);
  let lastCode = 'Unidentified';

  $effect(() => { input?.focus({ preventScroll: true }); });

  function consume(value: string, inputType: string) {
    const chars = typingCharacters(value, inputType);
    const code = chars.length === 1 ? lastCode : 'Unidentified';
    lastCode = 'Unidentified';
    for (const char of chars) onKey(char, code);
  }

  function handleKeydown(event: KeyboardEvent) {
    onCapsLockChange(event.getModifierState('CapsLock'));
    if (event.isComposing || event.keyCode === 229) return;
    lastCode = event.key.length === 1 ? event.code || 'Unidentified' : 'Unidentified';
    if (event.key === 'Backspace' && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      onKey('Backspace', event.code || 'Backspace');
      lastCode = 'Unidentified';
    } else if (event.key === 'Enter') {
      event.preventDefault();
    }
    // Printable keys keep their native default action; input supplies the text.
  }

  function handleInput(event: Event) {
    const nativeEvent = event as InputEvent;
    if (nativeEvent.isComposing) return;
    const target = event.currentTarget as HTMLTextAreaElement;
    const value = target.value;
    target.value = '';
    consume(value, nativeEvent.inputType);
  }

  function finishComposition(event: CompositionEvent) {
    if (!event.data) { if (input) input.value = ''; return; }
    // Engines differ on whether final input precedes compositionend. If input
    // already consumed the committed value this is empty, avoiding duplicates.
    queueMicrotask(() => {
      if (!input?.value) return;
      const value = input.value;
      input.value = '';
      lastCode = 'Unidentified';
      consume(value, 'insertFromComposition');
    });
  }
</script>

<textarea
  class="typing-input"
  aria-label={t(uiLang, 'test.input_label')}
  autocomplete="off"
  autocapitalize="off"
  spellcheck={false}
  rows="1"
  bind:this={input}
  onkeydown={handleKeydown}
  onbeforeinput={(event) => {
    if (event.inputType.startsWith('insertFromPaste') || event.inputType === 'insertFromDrop') event.preventDefault();
  }}
  oninput={handleInput}
  oncompositionstart={() => { lastCode = 'Unidentified'; }}
  oncompositionend={finishComposition}
  onpaste={(event) => event.preventDefault()}
  ondrop={(event) => event.preventDefault()}
></textarea>

<style>
  /* A real editable target keeps the WebView input context/layout in sync.
     It covers the text card, so clicking the text restores native focus. */
  .typing-input { position:absolute; inset:0; z-index:2; width:100%; height:100%; padding:0; border:0; resize:none; opacity:0; color:transparent; caret-color:transparent; cursor:text; }
</style>
