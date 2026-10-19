//! Guards bundled lesson content against cross-script contamination.
//!
//! A Russian or English lesson must never show Chinese, Japanese or Korean
//! characters, and CJK lessons must not show Cyrillic. These tests walk every
//! embedded word pack, course and quote so a stray character fails CI instead
//! of reaching a typing session.

use racoon_resources::{course_loader, find_foreign_characters, quote_loader, word_pack_loader};

#[test]
fn every_course_passes_validation() {
    let loader = course_loader();
    let mut languages = loader.available_languages();
    languages.sort();
    assert!(!languages.is_empty());

    for language in languages {
        if let Err(error) = loader.validate_course(&language) {
            panic!("course {language} failed validation: {error}");
        }
    }
}

#[test]
fn word_packs_only_contain_their_own_script() {
    let loader = word_pack_loader();
    let mut languages = loader.available_languages();
    languages.sort();

    for language in languages {
        let pack = loader.get_pack(&language).expect("word pack is loaded");
        for word in &pack.words {
            let foreign = find_foreign_characters(&language, word);
            assert!(
                foreign.is_empty(),
                "word pack {language} contains {foreign:?} in {word:?}"
            );
        }
    }
}

#[test]
fn quote_texts_only_contain_their_own_script() {
    let loader = quote_loader();
    let mut languages = loader.available_languages();
    languages.sort();

    for language in languages {
        let pack = loader.get_pack(&language).expect("quote pack is loaded");
        for quote in &pack.quotes {
            let foreign = find_foreign_characters(&language, &quote.text);
            assert!(
                foreign.is_empty(),
                "quotes for {language} contain {foreign:?} in {:?}",
                quote.text
            );
        }
    }
}
