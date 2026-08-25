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

export const FINGERS: Record<string, string> = {
  q: 'LP', a: 'LP', z: 'LP',
  w: 'LR', s: 'LR', x: 'LR',
  e: 'LM', d: 'LM', c: 'LM',
  r: 'LI', f: 'LI', v: 'LI', t: 'LI', g: 'LI', b: 'LI',
  y: 'RI', h: 'RI', n: 'RI', u: 'RI', j: 'RI', m: 'RI',
  i: 'RM', k: 'RM', ',': 'RM',
  o: 'RR', l: 'RR', '.': 'RR',
  p: 'RP', ';': 'RP', '/': 'RP',
};

export const RU_FINGERS: Record<string, string> = {
  ё: 'LP', ф: 'LP', я: 'LP', й: 'LP',
  ц: 'LR', ы: 'LR', ч: 'LR',
  у: 'LM', в: 'LM', с: 'LM',
  а: 'LI', п: 'LI', к: 'LI', м: 'LI', и: 'LI',
  о: 'RI', л: 'RI', д: 'RI', р: 'RI', т: 'RI',
  е: 'RM', г: 'RM', ш: 'RM',
  н: 'RR', щ: 'RR', з: 'RR', х: 'RR',
  ь: 'RP', б: 'RP', ю: 'RP', ъ: 'RP', ж: 'RP', э: 'RP', '.': 'RP',
};

export const HOME_ROW_EN = new Set(ROWS[1]);
export const HOME_ROW_RU = new Set(RU_ROWS[1]);

// Dvorak letter keys on the same physical positions as QWERTY (mirrors
// `finger_for_key_dvorak` in crates/core/src/finger_map.rs).
export const DVORAK_ROWS = [
  ["'", ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', 'l'],
  ['a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's'],
  [';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z'],
];

export const DVORAK_FINGERS: Record<string, string> = {
  "'": 'LP', a: 'LP', ';': 'LP',
  ',': 'LR', o: 'LR', q: 'LR',
  '.': 'LM', e: 'LM', j: 'LM',
  p: 'LI', y: 'LI', u: 'LI', i: 'LI', k: 'LI', x: 'LI',
  f: 'RI', g: 'RI', d: 'RI', h: 'RI', b: 'RI', m: 'RI',
  c: 'RM', t: 'RM', w: 'RM',
  r: 'RR', n: 'RR', v: 'RR',
  l: 'RP', s: 'RP', z: 'RP', '/': 'RP',
};

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

export function isHomeRowKey(key: string): boolean {
  return (
    HOME_ROW_EN.has(key) || HOME_ROW_RU.has(key) || DVORAK_HOME_ROW.has(key)
  );
}

const DVORAK_HOME_ROW = new Set(DVORAK_ROWS[1]);

export const VIEWPORT_CHARS = 120;
export const VIEWPORT_PADDING = 30;
