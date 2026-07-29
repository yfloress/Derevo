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
use crate::error::DbError;
use crate::models::{Habit, HabitLog};
use uuid::Uuid;

pub struct HabitService;

impl HabitService {
    pub fn create_habit(
        db: &Database,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Local::now().to_rfc3339();
        let habit = Habit::new(id.clone(), name, description, color, category, now);
        db.create_habit(&habit)?;
        Ok(id)
    }

    pub fn get_habits(db: &Database) -> Result<Vec<Habit>, DbError> {
        db.get_habits()
    }

    pub fn update_habit(
        db: &Database,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        is_archived: bool,
    ) -> Result<(), DbError> {
        match db.get_habit(&id)? {
            Some(mut habit) => {
                habit.name = name;
                habit.description = description;
                habit.color = color;
                habit.category = category;
                db.update_habit(&habit)?;
                if is_archived {
                    db.archive_habit(&id)?;
                }
                Ok(())
            }
            None => Err(DbError::GoalNotFound),
        }
    }

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
