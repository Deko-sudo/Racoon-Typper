// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Client-side text-pack helpers: source-format selection from a file name and
// the wire contract shared with racoon-data's TextPackImportPlan (pinned by
// scripts/text-pack-contract.test.mjs).

import type { TextPackImportPlan } from './types/index';

export type TextPackSourceFormatId = 'json' | 'tsv' | 'csv' | 'blocks';

export function formatForFile(fileName: string): TextPackSourceFormatId {
  const extension = fileName.toLowerCase().split('.').at(-1) ?? '';
  if (extension === 'json') return 'json';
  if (extension === 'csv') return 'csv';
  if (extension === 'tsv') return 'tsv';
  return 'blocks';
}

// Mirrors serde field names of racoon_data::text_pack::TextPackImportPlan.
export const TEXT_PACK_PLAN_FIELDS = [
  'policy',
  'source_format',
  'language',
  'incoming',
  'duplicates_in_pack',
  'existing_in_language',
  'to_insert',
  'to_skip',
  'removed_by_replace',
] as const satisfies ReadonlyArray<keyof TextPackImportPlan>;

export function summarizePlan(plan: TextPackImportPlan): string {
  return `+${plan.to_insert} / ~${plan.to_skip} / −${plan.removed_by_replace}`;
}
