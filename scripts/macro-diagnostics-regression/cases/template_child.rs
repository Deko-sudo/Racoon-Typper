// SPDX-License-Identifier: Apache-2.0
#[derive(gtk::CompositeTemplate)]
#[template(file = "../src/template.ui")]
struct Wrong {
    #[template_child(id = 42)]
    child: gtk::TemplateChild<gtk::Label>,
}
fn main() {}
