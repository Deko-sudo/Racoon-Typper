// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import type { Achievement } from './types/index';
import { t, UI_LANGUAGES } from './i18n';

// These values describe the existing backend catalog; they do not unlock achievements.
const goals: Record<string, { template: string; values: Record<string, number> }> = {
  "first_test": {"template": "first_test", "values": {}},
  "5_tests": {"template": "tests", "values": {"count": 5}},
  "10_tests": {"template": "tests", "values": {"count": 10}},
  "25_tests": {"template": "tests", "values": {"count": 25}},
  "50_tests": {"template": "tests", "values": {"count": 50}},
  "100_tests": {"template": "tests", "values": {"count": 100}},
  "250_tests": {"template": "tests", "values": {"count": 250}},
  "500_tests": {"template": "tests", "values": {"count": 500}},
  "1000_tests": {"template": "tests", "values": {"count": 1000}},
  "wpm_20": {"template": "wpm", "values": {"wpm": 20}},
  "wpm_30": {"template": "wpm", "values": {"wpm": 30}},
  "wpm_50": {"template": "wpm", "values": {"wpm": 50}},
  "wpm_60": {"template": "wpm", "values": {"wpm": 60}},
  "wpm_80": {"template": "wpm", "values": {"wpm": 80}},
  "wpm_100": {"template": "wpm", "values": {"wpm": 100}},
  "wpm_120": {"template": "wpm", "values": {"wpm": 120}},
  "wpm_150": {"template": "wpm", "values": {"wpm": 150}},
  "wpm_200": {"template": "wpm", "values": {"wpm": 200}},
  "acc_85": {"template": "accuracy", "values": {"accuracy": 85}},
  "acc_90": {"template": "accuracy", "values": {"accuracy": 90}},
  "acc_95": {"template": "accuracy", "values": {"accuracy": 95}},
  "acc_97": {"template": "accuracy", "values": {"accuracy": 97}},
  "acc_99": {"template": "accuracy", "values": {"accuracy": 99}},
  "acc_100": {"template": "accuracy", "values": {"accuracy": 100}},
  "streak_1": {"template": "streak", "values": {"days": 1}},
  "streak_3": {"template": "streak", "values": {"days": 3}},
  "streak_7": {"template": "streak", "values": {"days": 7}},
  "streak_14": {"template": "streak", "values": {"days": 14}},
  "streak_30": {"template": "streak", "values": {"days": 30}},
  "streak_60": {"template": "streak", "values": {"days": 60}},
  "streak_100": {"template": "streak", "values": {"days": 100}},
  "lessons_1": {"template": "first_lesson", "values": {}},
  "lessons_2": {"template": "lessons", "values": {"count": 2}},
  "lessons_5": {"template": "lessons", "values": {"count": 5}},
  "lessons_20": {"template": "lessons", "values": {"count": 20}},
  "lessons_50": {"template": "lessons", "values": {"count": 50}},
  "lessons_100": {"template": "lessons", "values": {"count": 100}},
  "lessons_200": {"template": "lessons", "values": {"count": 200}},
  "balanced_50_95": {"template": "speed_accuracy", "values": {"wpm": 50, "accuracy": 95}},
  "elite_80_95": {"template": "speed_accuracy", "values": {"wpm": 80, "accuracy": 95}},
  "marathon_100_80": {"template": "tests_wpm", "values": {"count": 100, "wpm": 80}},
  "legend_500_100": {"template": "tests_wpm", "values": {"count": 500, "wpm": 100}},
  "consistent_7_20": {"template": "streak_lessons", "values": {"days": 7, "count": 20}},
  "flawless_99_50": {"template": "tests_accuracy", "values": {"accuracy": 99, "count": 50}},
  "wpm_40": {"template": "wpm", "values": {"wpm": 40}},
  "wpm_70": {"template": "wpm", "values": {"wpm": 70}},
  "acc_92": {"template": "accuracy", "values": {"accuracy": 92}},
  "streak_21": {"template": "streak", "values": {"days": 21}},
  "lessons_10": {"template": "lessons", "values": {"count": 10}},
  "tests_75": {"template": "tests", "values": {"count": 75}},
  "wpm_90": {"template": "wpm", "values": {"wpm": 90}},
  "acc_98": {"template": "accuracy", "values": {"accuracy": 98}},
};

export function localizeAchievement(lang: string, achievement: Pick<Achievement, 'id' | 'name' | 'description'>): { name: string; description: string } {
  const goal = Object.hasOwn(goals, achievement.id) ? goals[achievement.id] : undefined;
  // A newer backend may return IDs this frontend does not know yet.
  if (!goal) return { name: achievement.name, description: achievement.description };
  const locale = UI_LANGUAGES.some(([code]) => code === lang) ? lang : 'en';
  const numbers = new Intl.NumberFormat(locale);
  const key = `achievement.goal.${goal.template}`;
  const quantity = goal.values.count ?? goal.values.days;
  const pluralKey = quantity === undefined ? key : `${key}.${new Intl.PluralRules(locale).select(quantity)}`;
  const pluralCopy = t(lang, pluralKey);
  const description = (pluralCopy === pluralKey ? t(lang, key) : pluralCopy)
    .replace(/\{(\w+)\}/g, (placeholder, key: string) =>
      Object.hasOwn(goal.values, key) ? numbers.format(goal.values[key]) : placeholder);
  return { name: t(lang, `achievement.${achievement.id}.name`), description };
}
