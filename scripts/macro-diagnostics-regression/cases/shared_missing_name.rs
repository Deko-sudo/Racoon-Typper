// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, glib::SharedBoxed)]
struct Wrong(std::sync::Arc<String>);
fn main() {}
