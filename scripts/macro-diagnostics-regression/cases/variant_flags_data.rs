// SPDX-License-Identifier: Apache-2.0
#[derive(glib::Variant)]
#[variant_enum(flags)]
enum Wrong { One, Two(i32) }
fn main() {}
