// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Pure onboarding flow logic shared by OnboardingView and its UI contract
// tests: step navigation bounds and goal value clamping.

export const ONBOARDING_STEPS = 3;

export function nextOnboardingStep(step: number): number {
  return Math.min(step + 1, ONBOARDING_STEPS - 1);
}

export function previousOnboardingStep(step: number): number {
  return Math.max(step - 1, 0);
}

export type OnboardingGoalType = 'time' | 'wpm' | 'accuracy';

import type { LanguageCode } from './types/index';

export interface OnboardingResult {
  practiceLanguages: LanguageCode[];
  goalType: OnboardingGoalType;
  goalMinutes: number;
  goalWpm: number;
  goalAccuracy: number;
}

const GOAL_LIMITS: Record<OnboardingGoalType, { min: number; max: number }> = {
  time: { min: 0, max: 1440 },
  wpm: { min: 0, max: 300 },
  accuracy: { min: 0, max: 100 },
};

export function clampGoalValue(
  goalType: OnboardingGoalType,
  rawValue: number,
): number {
  const limits = GOAL_LIMITS[goalType];
  const value = Number.isFinite(rawValue) ? rawValue : limits.min;
  return Math.min(limits.max, Math.max(limits.min, Math.round(value)));
}

export function buildOnboardingResult(
  practiceLanguages: LanguageCode[],
  goalType: OnboardingGoalType,
  rawMinutes: number,
  rawWpm: number,
  rawAccuracy: number,
): OnboardingResult {
  if (practiceLanguages.length === 0) {
    throw new Error('Select at least one practice language');
  }
  return {
    practiceLanguages: [...new Set(practiceLanguages)],
    goalType,
    goalMinutes: clampGoalValue('time', rawMinutes),
    goalWpm: clampGoalValue('wpm', rawWpm),
    goalAccuracy: clampGoalValue('accuracy', rawAccuracy),
  };
}

export function togglePracticeLanguage(
  languages: LanguageCode[],
  language: LanguageCode,
): LanguageCode[] {
  return languages.includes(language)
    ? languages.filter((selected) => selected !== language)
    : [...languages, language];
}
