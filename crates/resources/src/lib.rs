//! Resources — загрузка word packs, quotes, courses.
//! Sprint 6: WordPackLoader, QuoteLoader.
//! Sprint 9: CourseLoader.

pub mod courses;
pub mod quotes;
pub mod words;

pub use courses::{course_loader, Course, CourseLoader, LessonContent, ModuleContent};
pub use quotes::{quote_loader, Quote, QuoteLoader, QuotePack};
pub use words::{word_pack_loader, WordPack, WordPackLoader};
