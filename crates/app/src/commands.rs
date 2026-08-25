//! Tauri IPC command adapters, grouped by responsibility.
//!
//! Command modules validate transport inputs and delegate lifecycle/persistence
//! work to the private application service or data repositories. They retain
//! the existing command names and wire shapes.

use racoon_data::DbError;
use rusqlite::Connection;

use crate::error::AppError;
use crate::state::AppState;

pub(crate) mod content;
pub(crate) mod contracts;
pub(crate) mod preferences;
pub(crate) mod profile_transfer;
pub(crate) mod reporting;
pub(crate) mod session;
pub(crate) mod text_pack;

pub(crate) fn with_db<T>(
    state: &AppState,
    operation: impl FnOnce(&Connection) -> Result<T, DbError>,
) -> Result<T, AppError> {
    state.db.with_connection(operation).map_err(AppError::from)
}
