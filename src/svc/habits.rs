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
use crate::error::{AppError, DbError};
use crate::models::{Habit, HabitLog};
use uuid::Uuid;

/// Used when a habit is created without a category.
pub const DEFAULT_CATEGORY: &str = "general";

pub struct HabitService;

impl HabitService {
    pub fn create_habit(
        db: &Database,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        reminder_time: Option<String>,
    ) -> Result<String, AppError> {
        let name = validate_name(name)?;
        let color = validate_color(color)?;
        let reminder_time = validate_reminder(reminder_time)?;
        let category = canonical_category(db, &category)?;

        let id = Uuid::new_v4().to_string();
        let now = chrono::Local::now().to_rfc3339();
        let mut habit = Habit::new(id.clone(), name, description, color, category, now);
        habit.reminder_time = reminder_time;
        db.create_habit(&habit)?;
        Ok(id)
    }

    pub fn get_habits(db: &Database) -> Result<Vec<Habit>, DbError> {
        db.get_habits()
    }

    pub fn get_archived_habits(db: &Database) -> Result<Vec<(Habit, i32)>, DbError> {
        db.get_archived_habits()
    }

    pub fn get_categories(db: &Database) -> Result<Vec<String>, DbError> {
        db.get_categories()
    }

    pub fn update_habit(
        db: &Database,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        reminder_time: Option<String>,
    ) -> Result<(), AppError> {
        let name = validate_name(name)?;
        let color = validate_color(color)?;
        let reminder_time = validate_reminder(reminder_time)?;
        let category = canonical_category(db, &category)?;

        match db.get_habit(&id)? {
            Some(mut habit) => {
                habit.name = name;
                habit.description = description;
                habit.color = color;
                habit.category = category;
                habit.reminder_time = reminder_time;
                db.update_habit(&habit)?;
                Ok(())
            }
            None => Err(AppError::Database(DbError::HabitNotFound)),
        }
    }

    /// Hides the habit from tracking but keeps every log it has.
    pub fn archive_habit(db: &Database, id: String) -> Result<(), AppError> {
        if db.get_habit(&id)?.is_none() {
            return Err(AppError::Database(DbError::HabitNotFound));
        }
        db.archive_habit(&id)?;
        Ok(())
    }

    pub fn restore_habit(db: &Database, id: String) -> Result<(), AppError> {
        if db.get_habit(&id)?.is_none() {
            return Err(AppError::Database(DbError::HabitNotFound));
        }
        db.restore_habit(&id)?;
        Ok(())
    }

    /// Permanent, and takes the habit's logs with it.
    pub fn delete_habit(db: &Database, id: String) -> Result<(), DbError> {
        db.delete_habit(&id)
    }

    pub fn toggle_habit_completion(
        db: &Database,
        habit_id: String,
        date: String,
    ) -> Result<bool, DbError> {
        let (active, _id) = db.toggle_habit_log(&habit_id, &date)?;
        Ok(active)
    }

    pub fn get_habit_logs(
        db: &Database,
        start_date: String,
        end_date: String,
    ) -> Result<Vec<HabitLog>, DbError> {
        db.get_habit_logs(&start_date, &end_date)
    }
}

fn validate_name(name: String) -> Result<String, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("Habit name is required".into()));
    }
    Ok(name)
}

fn validate_color(color: String) -> Result<String, AppError> {
    let color = color.trim().to_string();
    let valid = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if !valid {
        return Err(AppError::Validation(format!("Invalid colour: {color}")));
    }
    Ok(color)
}

/// Accepts "HH:MM" in 24-hour form. An empty string means "no reminder", which
/// is what the form sends when the field is cleared.
fn validate_reminder(reminder: Option<String>) -> Result<Option<String>, AppError> {
    let Some(raw) = reminder else { return Ok(None) };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let parsed = raw
        .split_once(':')
        .and_then(|(h, m)| Some((h.parse::<u32>().ok()?, m.parse::<u32>().ok()?)))
        .filter(|(h, m)| *h < 24 && *m < 60);
    match parsed {
        Some((h, m)) => Ok(Some(format!("{h:02}:{m:02}"))),
        None => Err(AppError::Validation(format!(
            "Invalid reminder time: {raw}"
        ))),
    }
}

/// Keeps categories from splitting into near-duplicates. Whitespace is
/// collapsed, and a category that already exists under a different casing wins:
/// typing "Salud" when "salud" is on file stores "salud", so both habits land in
/// the same slice of the radar chart.
fn canonical_category(db: &Database, category: &str) -> Result<String, AppError> {
    let cleaned = category.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        return Ok(DEFAULT_CATEGORY.to_string());
    }
    match db.find_category_match(&cleaned)? {
        Some(existing) => Ok(existing),
        None => Ok(cleaned),
    }
}
