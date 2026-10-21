// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use glib::prelude::*;

fn fixture() -> glib::Variant {
    ["alpha", "кириллица", "", "Čas", "日本語"].to_variant()
}

#[test]
fn forward_iteration_preserves_every_utf8_value() {
    let variant = fixture();
    let got: Vec<_> = variant.array_iter_str().expect("string array").collect();
    assert_eq!(got, ["alpha", "кириллица", "", "Čas", "日本語"]);
}

#[test]
fn optimized_next_nth_last_and_reverse_obey_the_iterator_contract() {
    let variant = fixture();
    let mut iter = variant.array_iter_str().expect("string array");
    assert_eq!(iter.next(), Some("alpha"));
    assert_eq!(iter.next_back(), Some("日本語"));
    assert_eq!(iter.nth(1), Some(""));
    assert_eq!(iter.nth_back(0), Some("Čas"));
    assert_eq!(iter.next(), None);
    assert_eq!(
        variant.array_iter_str().expect("array").last(),
        Some("日本語")
    );
    assert_eq!(
        variant.array_iter_str().expect("array").nth(usize::MAX),
        None
    );
    assert_eq!(
        variant
            .array_iter_str()
            .expect("array")
            .nth_back(usize::MAX),
        None
    );
}

#[test]
fn empty_and_repeated_string_arrays_are_safe() {
    let empty = Vec::<String>::new().to_variant();
    assert_eq!(empty.array_iter_str().expect("empty array").next(), None);
    let strings = vec!["повтор"; 512];
    let variant = strings.to_variant();
    assert_eq!(
        variant
            .array_iter_str()
            .expect("array")
            .rev()
            .collect::<Vec<_>>(),
        strings
    );
}
