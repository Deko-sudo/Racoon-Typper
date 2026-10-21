// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Native input data is the text after layout/IME processing. KeyboardEvent.key
// is deliberately not used as the character source. Paste/drop are not typing.
export function typingCharacters(value: string, inputType: string): string[] {
  if (!['insertText', 'insertCompositionText', 'insertFromComposition'].includes(inputType)) return [];
  return Array.from(value).filter((char) => !/[\u0000-\u001f\u007f]/u.test(char));
}
