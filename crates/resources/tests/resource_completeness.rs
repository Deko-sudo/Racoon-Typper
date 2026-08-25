// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Resource-completeness audit (Phase 8 item 5 close-out).
//!
//! Every language advertised by `SUPPORTED_COURSE_LANGUAGES` must ship a
//! complete vertical slice: a course with enough modules and lessons, a quote
//! pack with usable variety, and a word list of practical size. The bar is
//! deliberately one notch above the current minimum so adding a new language
//! with stub content fails CI instead of shipping half-supported.

use std::collections::BTreeSet;

use racoon_resources::{CourseLoader, QuoteLoader, WordPackLoader, SUPPORTED_COURSE_LANGUAGES};

const MIN_MODULES: usize = 3;
const MIN_LESSONS: usize = 12;
const MIN_QUOTES: usize = 10;
const MIN_WORDS: usize = 300;

fn all_languages() -> Vec<&'static str> {
    let mut languages: Vec<&'static str> = SUPPORTED_COURSE_LANGUAGES.to_vec();
    languages.sort_unstable();
    languages
}

#[test]
fn course_loader_registry_matches_the_declared_language_set() {
    let loader = CourseLoader::new();
    let registered: BTreeSet<&str> = loader.languages().into_iter().collect();
    let declared: BTreeSet<&str> = all_languages().into_iter().collect();
    assert_eq!(
        registered, declared,
        "CourseLoader registry and SUPPORTED_COURSE_LANGUAGES drifted apart"
    );
}

#[test]
fn every_language_ships_a_complete_course() {
    let loader = CourseLoader::new();
    for language in all_languages() {
        let course = loader
            .load_course(language)
            .unwrap_or_else(|| panic!("course missing for {language}"));
        assert!(
            course.module_count() >= MIN_MODULES,
            "{language}: only {} modules (< {MIN_MODULES})",
            course.module_count()
        );
        assert!(
            course.lesson_count() >= MIN_LESSONS,
            "{language}: only {} lessons (< {MIN_LESSONS})",
            course.lesson_count()
        );

        let mut lesson_ids = BTreeSet::new();
        for module in &course.modules {
            assert!(
                !module.lessons.is_empty(),
                "{language}: empty module {}",
                module.id
            );
            for lesson in &module.lessons {
                assert!(
                    !lesson.text.trim().is_empty(),
                    "{language}: lesson {} has empty text",
                    lesson.id
                );
                assert!(
                    lesson_ids.insert(lesson.id.as_str()),
                    "{language}: duplicate lesson id {}",
                    lesson.id
                );
            }
        }
    }
}

#[test]
fn every_language_ships_a_usable_quote_pack() {
    let loader = QuoteLoader::new();
    for language in all_languages() {
        let pack = loader
            .get_pack(language)
            .unwrap_or_else(|| panic!("quote pack missing for {language}"));
        assert!(
            pack.len() >= MIN_QUOTES,
            "{language}: only {} quotes (< {MIN_QUOTES})",
            pack.len()
        );
        for quote in &pack.quotes {
            assert!(
                !quote.text.trim().is_empty(),
                "{language}: empty quote text"
            );
            assert!(
                !quote.author.trim().is_empty(),
                "{language}: quote without author"
            );
        }
    }
}

#[test]
fn every_language_ships_a_practical_word_pack() {
    let loader = WordPackLoader::new();
    for language in all_languages() {
        let pack = loader
            .get_pack(language)
            .unwrap_or_else(|| panic!("word pack missing for {language}"));
        assert!(
            pack.words.len() >= MIN_WORDS,
            "{language}: only {} words (< {MIN_WORDS})",
            pack.words.len()
        );
        for word in &pack.words {
            assert!(!word.trim().is_empty(), "{language}: empty word entry");
        }
        let unique: BTreeSet<&String> = pack.words.iter().collect();
        assert!(
            unique.len() >= pack.words.len() * 9 / 10,
            "{language}: more than 10% duplicated words"
        );
    }
}

#[test]
fn quote_and_word_loaders_cover_exactly_the_same_languages() {
    let quotes = QuoteLoader::new();
    let words = WordPackLoader::new();
    for language in all_languages() {
        assert!(
            quotes.get_pack(language).is_some(),
            "quotes missing for {language}"
        );
        assert!(
            words.get_pack(language).is_some(),
            "words missing for {language}"
        );
    }
}
