//! CourseLoader — загрузка курсов слепой печати из TOML.
//! 2 курса: EN (8 модулей), RU (8 модулей).

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// Один урок в курсе.
#[derive(Debug, Clone, Deserialize)]
pub struct LessonContent {
    pub id: String,
    pub name: String,
    pub text: String,
}

/// Один модуль курса.
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleContent {
    pub id: String,
    pub name: String,
    pub difficulty: String,
    pub order: u32,
    pub lessons: Vec<LessonContent>,
}

/// Полный курс для языка.
#[derive(Debug, Clone, Deserialize)]
pub struct Course {
    pub modules: Vec<ModuleContent>,
}

impl Course {
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn lesson_count(&self) -> usize {
        self.modules.iter().map(|m| m.lessons.len()).sum()
    }

    pub fn get_module(&self, module_id: &str) -> Option<&ModuleContent> {
        self.modules.iter().find(|m| m.id == module_id)
    }

    pub fn get_lesson(&self, lesson_id: &str) -> Option<&LessonContent> {
        for m in &self.modules {
            if let Some(l) = m.lessons.iter().find(|l| l.id == lesson_id) {
                return Some(l);
            }
        }
        None
    }
}

/// Загрузчик курсов. Кэширует загруженные курсы.
pub struct CourseLoader {
    courses: HashMap<String, Course>,
}

impl CourseLoader {
    pub fn new() -> Self {
        let mut courses = HashMap::new();

        macro_rules! load_lang {
            ($code:expr, $path:expr) => {
                let toml_str = include_str!($path);
                if let Ok(course) = toml::from_str::<Course>(toml_str) {
                    courses.insert($code.to_string(), course);
                }
            };
        }

        load_lang!("en", "../../../resources/courses/en.toml");
        load_lang!("ru", "../../../resources/courses/ru.toml");
        load_lang!("de", "../../../resources/courses/de.toml");
        load_lang!("uk", "../../../resources/courses/uk.toml");
        load_lang!("cs", "../../../resources/courses/cs.toml");
        load_lang!("pl", "../../../resources/courses/pl.toml");
        load_lang!("ro", "../../../resources/courses/ro.toml");
        load_lang!("it", "../../../resources/courses/it.toml");
        load_lang!("fr", "../../../resources/courses/fr.toml");
        load_lang!("es", "../../../resources/courses/es.toml");
        load_lang!("pt", "../../../resources/courses/pt.toml");
        load_lang!("ja", "../../../resources/courses/ja.toml");
        load_lang!("zh-hk", "../../../resources/courses/zh-hk.toml");
        load_lang!("zh-tw", "../../../resources/courses/zh-tw.toml");
        load_lang!("ko", "../../../resources/courses/ko.toml");

        Self { courses }
    }

    /// Возвращает курс для языка.
    pub fn load_course(&self, language: &str) -> Option<&Course> {
        self.courses.get(language)
    }

    /// Возвращает модуль по ID.
    pub fn load_module(&self, language: &str, module_id: &str) -> Option<&ModuleContent> {
        self.load_course(language)?.get_module(module_id)
    }

    /// Возвращает урок по ID.
    pub fn load_lesson(&self, language: &str, lesson_id: &str) -> Option<&LessonContent> {
        self.load_course(language)?.get_lesson(lesson_id)
    }

    /// Валидирует курс: проверяет структуру, уникальность ID, непустые тексты.
    pub fn validate_course(&self, language: &str) -> Result<(), String> {
        let course = self
            .load_course(language)
            .ok_or_else(|| format!("Course not found for language: {}", language))?;

        if course.modules.is_empty() {
            return Err("Course has no modules".to_string());
        }

        let mut lesson_ids = std::collections::HashSet::new();
        for m in &course.modules {
            if m.id.is_empty() {
                return Err(format!("Module has empty id: {}", m.name));
            }
            if m.lessons.is_empty() {
                return Err(format!("Module {} has no lessons", m.id));
            }
            for l in &m.lessons {
                if l.id.is_empty() {
                    return Err(format!("Lesson has empty id: {}", l.name));
                }
                if !lesson_ids.insert(&l.id) {
                    return Err(format!("Duplicate lesson id: {}", l.id));
                }
                if l.text.is_empty() {
                    return Err(format!("Lesson {} has empty text", l.id));
                }
            }
        }

        Ok(())
    }

    /// Возвращает список доступных языков.
    pub fn available_languages(&self) -> Vec<String> {
        self.courses.keys().cloned().collect()
    }
}

impl Default for CourseLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный синглтон.
static LOADER: OnceLock<CourseLoader> = OnceLock::new();

pub fn course_loader() -> &'static CourseLoader {
    LOADER.get_or_init(CourseLoader::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_en_course() {
        let loader = CourseLoader::new();
        let course = loader.load_course("en").unwrap();
        assert_eq!(course.module_count(), 8);
    }

    #[test]
    fn load_ru_course() {
        let loader = CourseLoader::new();
        let course = loader.load_course("ru").unwrap();
        assert_eq!(course.module_count(), 8);
    }

    #[test]
    fn en_has_5_lessons_per_module() {
        let loader = CourseLoader::new();
        let course = loader.load_course("en").unwrap();
        for m in &course.modules {
            assert!(
                m.lessons.len() >= 5,
                "Module {} has {} lessons",
                m.id,
                m.lessons.len()
            );
        }
    }

    #[test]
    fn ru_has_5_lessons_per_module() {
        let loader = CourseLoader::new();
        let course = loader.load_course("ru").unwrap();
        for m in &course.modules {
            assert!(
                m.lessons.len() >= 5,
                "Module {} has {} lessons",
                m.id,
                m.lessons.len()
            );
        }
    }

    #[test]
    fn en_total_40_lessons() {
        let loader = CourseLoader::new();
        let course = loader.load_course("en").unwrap();
        assert_eq!(course.lesson_count(), 40);
    }

    #[test]
    fn ru_total_40_lessons() {
        let loader = CourseLoader::new();
        let course = loader.load_course("ru").unwrap();
        assert_eq!(course.lesson_count(), 40);
    }

    #[test]
    fn load_module_by_id() {
        let loader = CourseLoader::new();
        let m = loader.load_module("en", "en_m1").unwrap();
        assert_eq!(m.name, "Home Row");
    }

    #[test]
    fn load_lesson_by_id() {
        let loader = CourseLoader::new();
        let l = loader.load_lesson("en", "en_m1_l1").unwrap();
        assert_eq!(l.name, "Left Hand: f j");
        assert!(!l.text.is_empty());
    }

    #[test]
    fn load_lesson_nonexistent() {
        let loader = CourseLoader::new();
        assert!(loader.load_lesson("en", "nonexistent").is_none());
    }

    #[test]
    fn validate_en_course() {
        let loader = CourseLoader::new();
        assert!(loader.validate_course("en").is_ok());
    }

    #[test]
    fn validate_ru_course() {
        let loader = CourseLoader::new();
        assert!(loader.validate_course("ru").is_ok());
    }

    #[test]
    fn validate_nonexistent_course() {
        let loader = CourseLoader::new();
        assert!(loader.validate_course("fr").is_err());
    }

    #[test]
    fn available_languages() {
        let loader = CourseLoader::new();
        let langs = loader.available_languages();
        assert!(langs.contains(&"en".to_string()));
        assert!(langs.contains(&"ru".to_string()));
    }

    #[test]
    fn en_module_order() {
        let loader = CourseLoader::new();
        let course = loader.load_course("en").unwrap();
        for w in course.modules.windows(2) {
            assert!(
                w[0].order < w[1].order,
                "Modules not in order: {} > {}",
                w[0].order,
                w[1].order
            );
        }
    }

    #[test]
    fn ru_module_order() {
        let loader = CourseLoader::new();
        let course = loader.load_course("ru").unwrap();
        for w in course.modules.windows(2) {
            assert!(
                w[0].order < w[1].order,
                "Modules not in order: {} > {}",
                w[0].order,
                w[1].order
            );
        }
    }

    #[test]
    fn en_lesson_ids_unique() {
        let loader = CourseLoader::new();
        let course = loader.load_course("en").unwrap();
        let mut ids: Vec<&str> = Vec::new();
        for m in &course.modules {
            for l in &m.lessons {
                ids.push(&l.id);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "Duplicate lesson IDs found");
    }

    #[test]
    fn lesson_text_not_empty() {
        let loader = CourseLoader::new();
        for lang in ["en", "ru"] {
            let course = loader.load_course(lang).unwrap();
            for m in &course.modules {
                for l in &m.lessons {
                    assert!(!l.text.is_empty(), "Empty text in {} / {}", lang, l.id);
                }
            }
        }
    }
}
