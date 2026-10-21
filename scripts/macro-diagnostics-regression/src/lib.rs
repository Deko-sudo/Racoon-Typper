// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

#[cfg(test)]
mod tests {
    use glib::{error::ErrorDomain, prelude::*, variant::FromVariant};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum)]
    #[enum_type(name = "RacoonMacroRegressionEnum")]
    enum TestEnum {
        One = 1,
        Two = 2,
    }

    #[glib::flags(name = "RacoonMacroRegressionFlags")]
    enum TestFlags {
        One = 1,
        Two = 2,
    }

    #[derive(Debug, Copy, Clone, glib::ErrorDomain)]
    #[error_domain(name = "racoon-macro-regression-error")]
    enum TestError {
        Failed = 1,
    }

    #[derive(Debug, Clone, PartialEq, Eq, glib::Boxed)]
    #[boxed_type(name = "RacoonMacroRegressionBoxed")]
    struct TestBoxed(String);

    #[derive(Debug, Clone, PartialEq, Eq, glib::SharedBoxed)]
    #[shared_boxed_type(name = "RacoonMacroRegressionShared")]
    struct TestShared(std::sync::Arc<String>);

    #[derive(Debug, PartialEq, glib::Variant)]
    enum TestVariant {
        Text(String),
        Number(i32),
    }

    mod imp {
        use glib::{prelude::*, subclass::prelude::*};
        use std::cell::Cell;

        #[derive(Default, glib::Properties)]
        #[properties(wrapper_type = super::TestObject)]
        pub struct TestObject {
            #[property(get, set)]
            count: Cell<i32>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for TestObject {
            const NAME: &'static str = "RacoonMacroRegressionObject";
            type Type = super::TestObject;
        }

        #[glib::derived_properties]
        impl ObjectImpl for TestObject {}

        #[derive(Clone, Copy)]
        #[repr(C)]
        pub struct TestInterface {
            parent: glib::gobject_ffi::GTypeInterface,
        }

        #[glib::object_interface]
        unsafe impl ObjectInterface for TestInterface {
            const NAME: &'static str = "RacoonMacroRegressionInterface";
        }
    }

    glib::wrapper! {
        pub struct TestObject(ObjectSubclass<imp::TestObject>);
    }

    glib::wrapper! {
        pub struct TestInterface(ObjectInterface<imp::TestInterface>);
    }

    // Compile CompositeTemplate expansion without a display or GTK initialization.
    mod widget {
        use gtk::subclass::prelude::*;

        mod imp {
            use super::*;

            #[derive(Default, gtk::CompositeTemplate)]
            #[template(file = "template.ui")]
            pub struct Widget;

            #[glib::object_subclass]
            impl ObjectSubclass for Widget {
                const NAME: &'static str = "RacoonMacroRegressionWidget";
                type Type = super::Widget;
                type ParentType = gtk::Box;
            }

            impl ObjectImpl for Widget {}
            impl WidgetImpl for Widget {}
            impl ContainerImpl for Widget {}
            impl BoxImpl for Widget {}
        }

        glib::wrapper! {
            pub struct Widget(ObjectSubclass<imp::Widget>)
                @extends gtk::Widget, gtk::Container, gtk::Box;
        }
    }

    #[test]
    fn enum_flags_error_and_boxed_roundtrip() {
        assert_eq!(
            TestEnum::Two.to_value().get::<TestEnum>().unwrap(),
            TestEnum::Two
        );
        let flags = TestFlags::One | TestFlags::Two;
        assert_eq!(flags.to_value().get::<TestFlags>().unwrap().bits(), 3);
        assert_eq!(<TestError as ErrorDomain>::from(1).unwrap().code(), 1);
        assert!(!TestError::domain().as_str().is_empty());
        let boxed = TestBoxed("plain".into());
        assert_eq!(boxed.to_value().get::<TestBoxed>().unwrap(), boxed);
        let shared = TestShared(std::sync::Arc::new("shared".into()));
        assert_eq!(shared.to_value().get::<TestShared>().unwrap(), shared);
    }

    #[test]
    fn variant_clone_closure_and_properties() {
        let original = TestVariant::Text("фыва 日本語".into());
        assert_eq!(
            TestVariant::from_variant(&original.to_variant()).unwrap(),
            original
        );
        let count = std::rc::Rc::new(3);
        let cloned = glib::clone!(@strong count => move || *count);
        assert_eq!(cloned(), 3);
        let closure = glib::closure_local!(@strong count => move |n: i32| n + *count);
        assert_eq!(closure.invoke::<i32>(&[&4i32]), 7);
        let object: TestObject = glib::Object::new();
        object.set_property("count", 42i32);
        assert_eq!(object.property::<i32>("count"), 42);
        assert!(TestInterface::static_type().is_a(glib::Type::INTERFACE));
    }
}
