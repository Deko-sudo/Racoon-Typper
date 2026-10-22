// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
// Synthetic examples, shared by unit and actual WebKitGTK renderer contracts.
export const languageSamples = {
  en: 'quick fox', ru: 'фыва ёж', de: 'größer süß', uk: 'ґанок їжак є і',
  cs: 'příliš žluťoučký', pl: 'zażółć gęślą jaźń', ro: 'țară școală înger',
  it: 'perché città più', fr: 'élève cœur où', es: 'niño pingüino qué',
  pt: 'ação coração avó', ja: '日本語 かな カナ', 'zh-hk': '香港 廣東話 學習',
  'zh-tw': '臺灣 注音 學習', ko: '한글 타자 연습',
};

export function sampleHeatmap(text) {
  const heatmap = Object.create(null);
  for (const symbol of text) {
    const entry = heatmap[symbol] ??= { total_attempts: 0, correct: 0, incorrect: 0, avg_wpm_at_key: 0 };
    entry.total_attempts++;
    entry.correct++;
  }
  return heatmap;
}
