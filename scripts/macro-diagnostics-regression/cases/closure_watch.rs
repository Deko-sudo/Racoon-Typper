// SPDX-License-Identifier: Apache-2.0
fn wrong() { let a = 1; let b = 2; let _ = glib::closure!(@watch a, @watch b => move || ()); }
fn main() {}
