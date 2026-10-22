// SPDX-License-Identifier: Apache-2.0
fn wrong() { let value = 1; let _ = glib::closure!(@bad value => move || value); }
fn main() {}
