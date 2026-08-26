// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildOnboardingResult,
  clampGoalValue,
  nextOnboardingStep,
  ONBOARDING_STEPS,
  previousOnboardingStep,
} from '../frontend/src/lib/onboarding.ts';

test('onboarding steps are bounded on both edges', () => {
  assert.equal(nextOnboardingStep(0), 1);
  assert.equal(nextOnboardingStep(ONBOARDING_STEPS - 2), ONBOARDING_STEPS - 1);
  assert.equal(nextOnboardingStep(ONBOARDING_STEPS - 1), ONBOARDING_STEPS - 1);
  assert.equal(previousOnboardingStep(0), 0);
  assert.equal(previousOnboardingStep(ONBOARDING_STEPS - 1), ONBOARDING_STEPS - 2);
});

test('goal values clamp into backend bounds per goal type', () => {
  assert.equal(clampGoalValue('time', 30), 30);
  assert.equal(clampGoalValue('time', -5), 0);
  assert.equal(clampGoalValue('time', 5000), 1440);
  assert.equal(clampGoalValue('wpm', 120.4), 120);
  assert.equal(clampGoalValue('wpm', 999), 300);
  assert.equal(clampGoalValue('accuracy', 99.6), 100);
  assert.equal(clampGoalValue('accuracy', 250), 100);
});

test('non-numeric goal input falls back to the minimum', () => {
  assert.equal(clampGoalValue('wpm', Number.NaN), 0);
  assert.equal(clampGoalValue('time', Number.NaN), 0);
});

test('result builder clamps every metric independently', () => {
  const result = buildOnboardingResult('de', 'wpm', -10, 350, 55.5);
  assert.deepEqual(result, {
    practiceLanguage: 'de',
    goalType: 'wpm',
    goalMinutes: 0,
    goalWpm: 300,
    goalAccuracy: 56,
  });
});
