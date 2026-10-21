export const ROWS = [
  ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
  ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
  ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
];

export const RU_ROWS = [
  ['й', 'ц', 'у', 'к', 'е', 'н', 'г', 'ш', 'щ', 'з', 'х', 'ъ'],
  ['ф', 'ы', 'в', 'а', 'п', 'р', 'о', 'л', 'д', 'ж', 'э'],
  ['я', 'ч', 'с', 'м', 'и', 'т', 'ь', 'б', 'ю', '.'],
];
export const DVORAK_ROWS = [
  ["'", ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', 'l'],
  ['a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's'],
  [';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z'],
];


// Canonical touch-typing fingering by PHYSICAL column. All layout tables
// above are ordered by physical position, so one column map covers every
// layout and language: the finger depends on where the key sits, not on the
// character it produces.
//   cols 0-4  -> left pinky/ring/middle/index/index
//   cols 5-6  -> right index, col 7 -> right middle,
//   col 8     -> right ring, col >= 9 -> right pinky
const FINGER_BY_COLUMN = ['LP', 'LR', 'LM', 'LI', 'LI'] as const;
function fingerForColumn(column: number): string {
  if (column < 5) return FINGER_BY_COLUMN[column];
  if (column <= 6) return 'RI';
  if (column === 7) return 'RM';
  if (column === 8) return 'RR';
  return 'RP';
}

// Number row — identical physical positions on every layout.
const DIGIT_ROW = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='];

function fingersForRows(rows: string[][]): Record<string, string> {
  const map: Record<string, string> = {};
  for (const row of rows) {
    row.forEach((character, column) => {
      const key = character.toLowerCase();
      if (!(key in map)) map[key] = fingerForColumn(column);
    });
  }
  return map;
}

export const FINGERS: Record<string, string> = (() => {
  const map = fingersForRows([DIGIT_ROW, ...ROWS]);
  // QWERTY-only outer punctuation keeps its home-column finger.
  for (const key of ['[', ']', '\\', "'", '-', '=']) map[key] ??= 'RP';
  return map;
})();

export const RU_FINGERS: Record<string, string> = (() => {
  const map = fingersForRows([DIGIT_ROW, ...RU_ROWS]);
  map['ё'] ??= 'LP'; // backtick position — left pinky
  return map;
})();

export const DVORAK_FINGERS: Record<string, string> =
  fingersForRows([DIGIT_ROW, ...DVORAK_ROWS]);

export type KeyboardLayoutId = 'qwerty' | 'jcuken' | 'dvorak';

const LATIN_LAYOUT_TABLES: Record<
  Exclude<KeyboardLayoutId, 'jcuken'>,
  { rows: string[][]; fingers: Record<string, string> }
> = {
  qwerty: { rows: ROWS, fingers: FINGERS },
  dvorak: { rows: DVORAK_ROWS, fingers: DVORAK_FINGERS },
};

function normalizeLayout(layout: string | undefined | null): Exclude<KeyboardLayoutId, 'jcuken'> {
  return layout === 'dvorak' ? 'dvorak' : 'qwerty';
}

// Cyrillic characters physically live on JCUKEN regardless of the selected
// Latin layout (same rule as `finger_for_char_with_layout` in racoon-core).
export function layoutRows(
  layout: string | undefined | null,
  isCyrillic: boolean,
): string[][] {
  if (isCyrillic) return RU_ROWS;
  return LATIN_LAYOUT_TABLES[normalizeLayout(layout)].rows;
}

export function layoutFingers(
  layout: string | undefined | null,
  isCyrillic: boolean,
): Record<string, string> {
  if (isCyrillic) return RU_FINGERS;
  return LATIN_LAYOUT_TABLES[normalizeLayout(layout)].fingers;
}

/// Physical key -> finger for the next-character highlight. Resolves the
/// character against the active layout's position tables (digits included),
/// so a displayed character never picks a finger by its glyph: A/Ф share the
/// left-pinky position, S/Ы the left-ring position, and so on. Unknown keys
/// return '' and must leave the hand guide un-highlighted.
export function fingerForKey(
  char: string,
  layout: string | undefined | null,
  isCyrillic: boolean,
): string {
  if (!char) return '';
  if (char === ' ') return 'RT'; // right thumb on the space bar
  const key = char.toLowerCase();
  for (const row of [DIGIT_ROW, ...layoutRows(layout, isCyrillic)]) {
    const column = row.indexOf(key);
    if (column !== -1) return fingerForColumn(column);
  }
  return layoutFingers(layout, isCyrillic)[key] ?? '';
}

export const HOME_ROW_EN = new Set(ROWS[1]);
export const HOME_ROW_RU = new Set(RU_ROWS[1]);

// Dvorak letter keys on the same physical positions as QWERTY (mirrors
// `finger_for_key_dvorak` in crates/core/src/finger_map.rs).
export const VIEWPORT_CHARS = 120;
export const VIEWPORT_PADDING = 30;
