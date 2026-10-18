// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { typingCharacters } from '../frontend/src/lib/typingInput.ts';

test('committed Cyrillic is preserved without substituting physical Latin keys', () => {
  assert.deepEqual(typingCharacters('фыва', 'insertText'), ['ф', 'ы', 'в', 'а']);
  assert.deepEqual(typingCharacters('hello', 'insertText'), ['h', 'e', 'l', 'l', 'o']);
});

test('composition commits retain Unicode scalar boundaries', () => {
  assert.deepEqual(typingCharacters('日本😀', 'insertFromComposition'), ['日', '本', '😀']);
  assert.deepEqual(typingCharacters('é', 'insertCompositionText'), ['é']);
});

test('clipboard, drop, history and deletion are not new typed characters', () => {
  for (const inputType of ['insertFromPaste', 'insertFromDrop', 'deleteContentBackward', 'historyUndo']) {
    assert.deepEqual(typingCharacters('ф', inputType), []);
  }
});

test('spaces remain input but newline, tab and control characters do not', () => {
  assert.deepEqual(typingCharacters(' \r\n\t\u0000a', 'insertText'), [' ', 'a']);
  assert.deepEqual(typingCharacters('', 'insertText'), []);
});
