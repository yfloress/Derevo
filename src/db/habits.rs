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

use crate::error::DbError;
use crate::models::{Habit, HabitLog};
use rusqlite::{Connection, params};

impl super::Database {
    // ── Habits CRUD ──

    pub fn create_habit(&self, habit: &Habit) -> Result<(), DbError> {
        Self::create_habit_on(&self.write(), habit)
    }

    pub(crate) fn create_habit_on(conn: &Connection, habit: &Habit) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO habits
                (id, name, description, color, category, created_at, archived, reminder_time,
                 schedule_kind, schedule_days, target_per_period)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &habit.id,
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.created_at,
                habit.archived as i32,
                &habit.reminder_time,
                &habit.schedule_kind,
                &habit.schedule_days,
                habit.target_per_period
            ],
        )?;
        Ok(())
    }

    /// Every habit query selects the same columns in this order.
    pub(crate) const HABIT_COLUMNS: &'static str =
        "id, name, description, color, category, created_at, archived, reminder_time,
         schedule_kind, schedule_days, target_per_period";

    fn row_to_habit(row: &rusqlite::Row<'_>) -> rusqlite::Result<Habit> {
        Ok(Habit {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            category: row.get(4)?,
            created_at: row.get(5)?,
            archived: row.get::<_, i32>(6)? != 0,
            reminder_time: row.get(7)?,
            schedule_kind: row.get(8)?,
            schedule_days: row.get(9)?,
            target_per_period: row.get(10)?,
        })
    }

    pub fn get_habits(&self) -> Result<Vec<Habit>, DbError> {
        let conn = self.read()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM habits WHERE archived = 0 ORDER BY created_at ASC",
            Self::HABIT_COLUMNS
        ))?;
        let habits = stmt
            .query_map([], Self::row_to_habit)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(habits)
    }

    /// Archived habits, newest first, each with how many logs it still holds so
    /// the user knows what a permanent delete would take with it.
    pub fn get_archived_habits(&self) -> Result<Vec<(Habit, i32)>, DbError> {
        let conn = self.read()?;
        let mut stmt = conn.prepare(
            "SELECT h.id, h.name, h.description, h.color, h.category, h.created_at,
                    h.archived, h.reminder_time, h.schedule_kind, h.schedule_days,
                    h.target_per_period, COUNT(l.id)
             FROM habits h
             LEFT JOIN habit_logs l ON l.habit_id = h.id
             WHERE h.archived = 1
             GROUP BY h.id
             ORDER BY h.created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok((Self::row_to_habit(row)?, row.get::<_, i32>(11)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Distinct categories in use, most used first. Feeds the form's suggestions
    /// so the user reaches for an existing category instead of inventing a
    /// near-duplicate.
    pub fn get_categories(&self) -> Result<Vec<String>, DbError> {
        let conn = self.read()?;
        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*) AS uses
             FROM habits WHERE archived = 0
             GROUP BY category ORDER BY uses DESC, category ASC",
        )?;
        let categories = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }

    /// The stored spelling of an existing category that matches `name` apart
    /// from case and surrounding space, if there is one.
    pub fn find_category_match(&self, name: &str) -> Result<Option<String>, DbError> {
        let conn = self.read()?;
        let result = conn.query_row(
            "SELECT category FROM habits
             WHERE category = ?1 COLLATE NOCASE
             ORDER BY archived ASC LIMIT 1",
            params![name],
            |row| row.get(0),
        );
        match result {
            Ok(category) => Ok(Some(category)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn get_habit(&self, id: &str) -> Result<Option<Habit>, DbError> {
        let conn = self.read()?;
        Self::get_habit_on(&conn, id)
    }

    pub(crate) fn get_habit_on(conn: &Connection, id: &str) -> Result<Option<Habit>, DbError> {
        let result = conn.query_row(
            &format!("SELECT {} FROM habits WHERE id = ?1", Self::HABIT_COLUMNS),
            params![id],
            Self::row_to_habit,
        );
        match result {
            Ok(habit) => Ok(Some(habit)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn update_habit(&self, habit: &Habit) -> Result<(), DbError> {
        let conn = self.write();
        conn.execute(
            "UPDATE habits
             SET name = ?1, description = ?2, color = ?3, category = ?4, reminder_time = ?5,
                 schedule_kind = ?6, schedule_days = ?7, target_per_period = ?8
             WHERE id = ?9",
            params![
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.reminder_time,
                &habit.schedule_kind,
                &habit.schedule_days,
                habit.target_per_period,
                &habit.id
            ],
        )?;
        Ok(())
    }

    pub fn archive_habit(&self, id: &str) -> Result<(), DbError> {
        let conn = self.write();
        conn.execute("UPDATE habits SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn restore_habit(&self, id: &str) -> Result<(), DbError> {
        let conn = self.write();
        conn.execute("UPDATE habits SET archived = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Permanent: the foreign key cascades to this habit's logs and rewards.
    pub fn delete_habit(&self, id: &str) -> Result<(), DbError> {
        let conn = self.write();
        conn.execute("DELETE FROM habits WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Habit Logs CRUD ──

    pub fn get_habit_logs(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<HabitLog>, DbError> {
        let conn = self.read()?;
        let mut stmt = conn.prepare(
            "SELECT id, habit_id, completed_date
             FROM habit_logs WHERE completed_date >= ?1 AND completed_date <= ?2
             ORDER BY completed_date ASC",
        )?;
        let logs = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok(HabitLog {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    completed_date: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(logs)
    }

    pub(crate) fn toggle_habit_log(
        &self,
        habit_id: &str,
        date: &str,
    ) -> Result<(bool, Option<String>), DbError> {
        let conn = self.write();
        if Self::habit_log_exists_on(&conn, habit_id, date)? {
            let _rows = conn.execute(
                "DELETE FROM habit_logs WHERE habit_id = ?1 AND completed_date = ?2",
                params![habit_id, date],
            )?;
            Ok((false, None))
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let log = HabitLog::new(id.clone(), habit_id.to_string(), date.to_string());
            conn.execute(
                "INSERT INTO habit_logs (id, habit_id, completed_date) VALUES (?1, ?2, ?3)",
                params![&log.id, &log.habit_id, &log.completed_date],
            )?;
            Ok((true, Some(id)))
        }
    }

    pub(crate) fn habit_log_exists_on(
        conn: &Connection,
        habit_id: &str,
        date: &str,
    ) -> Result<bool, DbError> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM habit_logs WHERE habit_id = ?1 AND completed_date = ?2",
            params![habit_id, date],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}
