//! Resources — загрузка word packs, quotes, courses.
//! Sprint 6: WordPackLoader, QuoteLoader.
//! Sprint 9: CourseLoader.

pub mod courses;
pub mod quotes;
pub mod random;
pub mod words;

pub use courses::{
    course_loader, Course, CourseLoader, LessonContent, ModuleContent, SUPPORTED_COURSE_LANGUAGES,
};
pub use quotes::{quote_loader, Quote, QuoteLoader, QuotePack};
pub use random::SystemRandomSource;
pub use words::{word_pack_loader, WordPack, WordPackLoader};
