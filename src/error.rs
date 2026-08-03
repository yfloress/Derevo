// Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Goal not found")]
    GoalNotFound,

    #[error("Habit not found")]
    HabitNotFound,

    #[error("Could not access application data directory")]
    AppDataDir,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Database(#[from] DbError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File error: {0}")]
    Io(String),

    #[error("The file is not a valid Derevo backup: {0}")]
    InvalidBackup(String),

    #[error("This backup was written by a newer version of Derevo (format {0})")]
    UnsupportedBackup(i32),
}

impl AppError {
    /// Stable identifier the frontend switches on to pick a translated message.
    /// Never change one of these without updating the matching i18n key.
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Database(DbError::GoalNotFound) => "goal-not-found",
            AppError::Database(DbError::HabitNotFound) => "habit-not-found",
            AppError::Database(DbError::AppDataDir) => "app-data-dir",
            AppError::Database(_) => "database",
            AppError::Validation(_) => "validation",
            AppError::Io(_) => "io",
            AppError::InvalidBackup(_) => "invalid-backup",
            AppError::UnsupportedBackup(_) => "unsupported-backup",
        }
    }
}
