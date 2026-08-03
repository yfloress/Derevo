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

use crate::db::Database;
use crate::dto::SettingsDto;
use crate::error::{AppError, DbError};

pub const KEY_THEME: &str = "theme";
pub const KEY_LANGUAGE: &str = "language";

const THEMES: [&str; 2] = ["dark", "light"];
const LANGUAGES: [&str; 2] = ["en", "es"];

pub struct SettingsService;

impl SettingsService {
    /// Defaults are applied here rather than in the frontend so a fresh install
    /// and a restored backup start from the same place.
    pub fn get_all(db: &Database) -> Result<SettingsDto, DbError> {
        Ok(SettingsDto {
            theme: db
                .get_setting(KEY_THEME)?
                .filter(|v| THEMES.contains(&v.as_str()))
                .unwrap_or_else(|| "dark".to_string()),
            language: db
                .get_setting(KEY_LANGUAGE)?
                .filter(|v| LANGUAGES.contains(&v.as_str()))
                .unwrap_or_else(|| "en".to_string()),
        })
    }

    pub fn set_theme(db: &Database, theme: &str) -> Result<(), AppError> {
        if !THEMES.contains(&theme) {
            return Err(AppError::Validation(format!("Unknown theme: {theme}")));
        }
        db.set_setting(KEY_THEME, theme)?;
        Ok(())
    }

    pub fn set_language(db: &Database, language: &str) -> Result<(), AppError> {
        if !LANGUAGES.contains(&language) {
            return Err(AppError::Validation(format!(
                "Unknown language: {language}"
            )));
        }
        db.set_setting(KEY_LANGUAGE, language)?;
        Ok(())
    }
}
