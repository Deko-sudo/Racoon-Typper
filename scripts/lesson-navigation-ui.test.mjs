// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import { lessonResultNavigation } from '../frontend/src/lib/lessonNavigation.ts';

const modules = [
  {
    id: 'module-2',
    name: 'Second',
    difficulty: 'medium',
    order: 2,
    lessons: [
      { id: 'lesson-3', name: 'Third', text_length: 30 },
    ],
  },
  {
    id: 'module-1',
    name: 'First',
    difficulty: 'easy',
    order: 1,
    lessons: [
      { id: 'lesson-1', name: 'First', text_length: 10 },
      { id: 'lesson-2', name: 'Second', text_length: 20 },
    ],
  },
];

test('completed lesson exposes repeat, course, and the next lesson in course order', () => {
  assert.deepEqual(lessonResultNavigation(modules, 'lesson-1'), {
    lessonId: 'lesson-1',
    nextLessonId: 'lesson-2',
  });
  assert.deepEqual(lessonResultNavigation(modules, 'lesson-2'), {
    lessonId: 'lesson-2',
    nextLessonId: 'lesson-3',
  });
});

test('last completed lesson keeps lesson actions without inventing a next lesson', () => {
  assert.deepEqual(lessonResultNavigation(modules, 'lesson-3'), {
    lessonId: 'lesson-3',
    nextLessonId: null,
  });
});

test('regular tests and unknown lesson ids do not expose lesson actions', () => {
  assert.equal(lessonResultNavigation(modules, null), null);
  assert.equal(lessonResultNavigation(modules, 'missing'), null);
});
