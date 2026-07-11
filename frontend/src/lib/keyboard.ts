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
  ё: 'LP', ф: 'LP', я: 'LP',
  ц: 'LR', ы: 'LR', ч: 'LR',
  у: 'LM', в: 'LM', с: 'LM',
  а: 'LI', п: 'LI', к: 'LI', м: 'LI',
  о: 'RI', л: 'RI', д: 'RI', р: 'RI', т: 'RI',
  е: 'RM', г: 'RM', ш: 'RM',
  н: 'RR', щ: 'RR', з: 'RR',
  ь: 'RP', б: 'RP', ю: 'RP', ъ: 'RP',
};

export const HOME_ROW_EN = new Set(ROWS[1]);
export const HOME_ROW_RU = new Set(RU_ROWS[1]);

export const VIEWPORT_CHARS = 120;
export const VIEWPORT_PADDING = 30;
