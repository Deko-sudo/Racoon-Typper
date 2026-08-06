// Client-side profile-transfer guards and presentation helpers.

import type { CollectionImportPlan, ProfileImportPlan } from './types/index';

export const MAX_PROFILE_FILE_BYTES = 64 * 1024 * 1024;

type ProfileFileMetadata = {
  name: string;
  size: number;
};

export type ProfileFileValidationError = 'empty' | 'not_json' | 'too_large';

export type ProfileImportRow = {
  key: Exclude<keyof ProfileImportPlan, 'policy'>;
  incoming: number;
  existing: number;
  toInsert: number;
};

const COLLECTION_KEYS: ProfileImportRow['key'][] = [
  'tests',
  'personal_bests',
  'daily_stats',
  'streaks',
  'custom_texts',
  'lesson_progress',
];

export function validateProfileFileMetadata(file: ProfileFileMetadata): ProfileFileValidationError | null {
  if (file.size === 0) return 'empty';
  if (!file.name.toLowerCase().endsWith('.json')) return 'not_json';
  if (file.size > MAX_PROFILE_FILE_BYTES) return 'too_large';
  return null;
}

export function profileImportRows(plan: ProfileImportPlan): ProfileImportRow[] {
  return COLLECTION_KEYS.map((key) => {
    const collection: CollectionImportPlan = plan[key];
    return {
      key,
      incoming: collection.incoming,
      existing: collection.existing,
      toInsert: collection.to_insert,
    };
  });
}
