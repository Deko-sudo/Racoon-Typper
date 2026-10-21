import type { ModuleResponse } from './types/index';

export type LessonResultNavigation = {
  lessonId: string;
  nextLessonId: string | null;
};

export function lessonResultNavigation(
  modules: ModuleResponse[],
  lessonId: string | null,
): LessonResultNavigation | null {
  if (!lessonId) return null;

  const lessonIds = [...modules]
    .sort((left, right) => left.order - right.order)
    .flatMap((module) => module.lessons.map((lesson) => lesson.id));
  const currentIndex = lessonIds.indexOf(lessonId);
  if (currentIndex < 0) return null;

  return {
    lessonId,
    nextLessonId: lessonIds[currentIndex + 1] ?? null,
  };
}
