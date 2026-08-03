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
use crate::models::{Habit, HabitLog, SCHEDULE_DAILY, SCHEDULE_TIMES_PER_WEEK, SCHEDULE_WEEKDAYS};
use uuid::Uuid;

/// Used when a habit is created without a category.
pub const DEFAULT_CATEGORY: &str = "general";

/// What the caller says about how often the habit is expected. Validated into
/// the three columns the table stores.
#[derive(Debug, Clone)]
pub struct ScheduleInput {
    pub kind: String,
    /// Weekday numbers, Sunday being 0. Only read for `weekdays`.
    pub days: Vec<u32>,
    /// Only read for `times_per_week`.
    pub target_per_period: Option<i32>,
}

impl Default for ScheduleInput {
    fn default() -> Self {
        ScheduleInput {
            kind: SCHEDULE_DAILY.to_string(),
            days: Vec::new(),
            target_per_period: None,
        }
    }
}

/// Everything the habit form sends. One struct rather than a row of positional
/// arguments: the two calls that take it were up to eight parameters, where a
/// swapped colour and category would have compiled fine.
#[derive(Debug, Clone, Default)]
pub struct HabitInput {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    pub reminder_time: Option<String>,
    pub schedule: ScheduleInput,
}

pub struct HabitService;

impl HabitService {
    pub fn create_habit(db: &Database, input: HabitInput) -> Result<String, AppError> {
        let name = validate_name(input.name)?;
        let color = validate_color(input.color)?;
        let reminder_time = validate_reminder(input.reminder_time)?;
        let (kind, days, target) = validate_schedule(input.schedule)?;
        let category = canonical_category(db, &input.category)?;

        let id = Uuid::new_v4().to_string();
        let now = chrono::Local::now().to_rfc3339();
        let mut habit = Habit::new(id.clone(), name, input.description, color, category, now);
        habit.reminder_time = reminder_time;
        habit.schedule_kind = kind;
        habit.schedule_days = days;
        habit.target_per_period = target;
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

    pub fn update_habit(db: &Database, id: String, input: HabitInput) -> Result<(), AppError> {
        let name = validate_name(input.name)?;
        let color = validate_color(input.color)?;
        let reminder_time = validate_reminder(input.reminder_time)?;
        let (kind, days, target) = validate_schedule(input.schedule)?;
        let category = canonical_category(db, &input.category)?;

        match db.get_habit(&id)? {
            Some(mut habit) => {
                habit.name = name;
                habit.description = input.description;
                habit.color = color;
                habit.category = category;
                habit.reminder_time = reminder_time;
                habit.schedule_kind = kind;
                habit.schedule_days = days;
                habit.target_per_period = target;
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

/// Turns a schedule request into the three stored columns. A `weekdays` habit
/// with no days would never come due again, and a weekly target below one would
/// be satisfied by doing nothing, so both are refused rather than corrected.
type StoredSchedule = (String, Option<String>, Option<i32>);

fn validate_schedule(schedule: ScheduleInput) -> Result<StoredSchedule, AppError> {
    match schedule.kind.as_str() {
        SCHEDULE_WEEKDAYS => {
            let mut days: Vec<u32> = schedule.days.into_iter().filter(|d| *d < 7).collect();
            days.sort_unstable();
            days.dedup();
            if days.is_empty() {
                return Err(AppError::Validation("Pick at least one weekday".into()));
            }
            let joined = days
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(",");
            Ok((SCHEDULE_WEEKDAYS.to_string(), Some(joined), None))
        }
        SCHEDULE_TIMES_PER_WEEK => {
            let target = schedule.target_per_period.unwrap_or(0);
            if !(1..=7).contains(&target) {
                return Err(AppError::Validation(
                    "A weekly target has to be between 1 and 7".into(),
                ));
            }
            Ok((SCHEDULE_TIMES_PER_WEEK.to_string(), None, Some(target)))
        }
        SCHEDULE_DAILY => Ok((SCHEDULE_DAILY.to_string(), None, None)),
        other => Err(AppError::Validation(format!("Unknown schedule: {other}"))),
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
