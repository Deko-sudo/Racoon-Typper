//! Database Layer — SQLite, rusqlite, миграции, repository pattern.
//! Sprint 4: Database init, migrations, TestRepository, PersonalBestsRepository.
//! Sprint 5: CustomTextRepository (full CRUD).

pub mod db;
pub mod error;
pub mod models;
pub mod repository;

pub use db::Database;
pub use error::DbError;
pub use repository::{
    CustomTextRepository, LessonRepository, PersonalBestsRepository, TestRepository,
};

// Re-export domain types for convenience
pub use racoon_domain::{PersonalBest, TestDetail, TestSummary};

// Re-export CustomText from repository
pub use repository::custom_texts::CustomText;
