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
use crate::models::{
    Achievement, BackupData, Checkpoint, Goal, Habit, HabitLog, Milestone, StreakReward,
};
use rusqlite::{Connection, params};

/// Order matters on restore: parents before the rows that reference them.
const TABLES_CHILD_FIRST: [&str; 8] = [
    "achievements",
    "checkpoints",
    "goals",
    "milestones",
    "streak_rewards",
    "habit_logs",
    "habits",
    "settings",
];

impl super::Database {
    /// Every row of every table, archived entries included — a backup that drops
    /// them would quietly lose data on the next restore.
    pub fn export_bundle(&self) -> Result<BackupData, DbError> {
        let conn = self.read()?;

        let habits = conn
            .prepare(
                "SELECT id, name, description, color, category, created_at, archived, reminder_time
                 FROM habits ORDER BY created_at ASC",
            )?
            .query_map([], |row| {
                Ok(Habit {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    archived: row.get::<_, i32>(6)? != 0,
                    reminder_time: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let habit_logs = conn
            .prepare(
                "SELECT id, habit_id, completed_date FROM habit_logs
                 ORDER BY completed_date ASC",
            )?
            .query_map([], |row| {
                Ok(HabitLog {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    completed_date: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let streak_rewards = conn
            .prepare(
                "SELECT id, habit_id, is_consecutive, target_days, target_total, created_at
                 FROM streak_rewards ORDER BY created_at ASC",
            )?
            .query_map([], |row| {
                Ok(StreakReward {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    is_consecutive: row.get::<_, i32>(2)? != 0,
                    target_days: row.get(3)?,
                    target_total: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let milestones = conn
            .prepare(
                "SELECT id, reward_id, target_days, reward_text, unlocked, unlocked_at
                 FROM milestones ORDER BY target_days ASC",
            )?
            .query_map([], |row| {
                Ok(Milestone {
                    id: row.get(0)?,
                    reward_id: row.get(1)?,
                    target_days: row.get(2)?,
                    reward_text: row.get(3)?,
                    unlocked: row.get::<_, i32>(4)? != 0,
                    unlocked_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let goals = conn
            .prepare(
                "SELECT id, name, description, reward_text, deadline, is_completed,
                        completed_at, created_at, archived
                 FROM goals ORDER BY created_at ASC",
            )?
            .query_map([], |row| {
                Ok(Goal {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    reward_text: row.get(3)?,
                    deadline: row.get(4)?,
                    is_completed: row.get::<_, i32>(5)? != 0,
                    completed_at: row.get(6)?,
                    created_at: row.get(7)?,
                    archived: row.get::<_, i32>(8)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let checkpoints = conn
            .prepare(
                "SELECT id, goal_id, description, completed, completed_at, sort_order
                 FROM checkpoints ORDER BY sort_order ASC",
            )?
            .query_map([], |row| {
                Ok(Checkpoint {
                    id: row.get(0)?,
                    goal_id: row.get(1)?,
                    description: row.get(2)?,
                    completed: row.get::<_, i32>(3)? != 0,
                    completed_at: row.get(4)?,
                    sort_order: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let achievements = conn
            .prepare(
                "SELECT id, title, description, icon_path, achievement_type, source_id, achieved_at
                 FROM achievements ORDER BY achieved_at ASC",
            )?
            .query_map([], |row| {
                Ok(Achievement {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    icon_path: row.get(3)?,
                    achievement_type: row.get(4)?,
                    source_id: row.get(5)?,
                    achieved_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let settings = conn
            .prepare("SELECT key, value FROM settings ORDER BY key ASC")?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BackupData {
            habits,
            habit_logs,
            streak_rewards,
            milestones,
            goals,
            checkpoints,
            achievements,
            settings,
        })
    }

    /// Replaces the whole database with `data`. All or nothing: the caller's
    /// data survives untouched if any row in the backup is rejected.
    pub fn replace_all(&self, data: &BackupData) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            for table in TABLES_CHILD_FIRST {
                conn.execute(&format!("DELETE FROM {table}"), [])?;
            }
            insert_all(conn, data)
        })
    }
}

fn insert_all(conn: &Connection, data: &BackupData) -> Result<(), DbError> {
    for habit in &data.habits {
        conn.execute(
            "INSERT INTO habits
                (id, name, description, color, category, created_at, archived, reminder_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &habit.id,
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.created_at,
                habit.archived as i32,
                &habit.reminder_time
            ],
        )?;
    }
    for log in &data.habit_logs {
        conn.execute(
            "INSERT INTO habit_logs (id, habit_id, completed_date) VALUES (?1, ?2, ?3)",
            params![&log.id, &log.habit_id, &log.completed_date],
        )?;
    }
    for reward in &data.streak_rewards {
        conn.execute(
            "INSERT INTO streak_rewards
                (id, habit_id, is_consecutive, target_days, target_total, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &reward.id,
                &reward.habit_id,
                reward.is_consecutive as i32,
                reward.target_days,
                reward.target_total,
                &reward.created_at
            ],
        )?;
    }
    for milestone in &data.milestones {
        conn.execute(
            "INSERT INTO milestones
                (id, reward_id, target_days, reward_text, unlocked, unlocked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &milestone.id,
                &milestone.reward_id,
                milestone.target_days,
                &milestone.reward_text,
                milestone.unlocked as i32,
                &milestone.unlocked_at
            ],
        )?;
    }
    for goal in &data.goals {
        conn.execute(
            "INSERT INTO goals
                (id, name, description, reward_text, deadline, is_completed,
                 completed_at, created_at, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &goal.id,
                &goal.name,
                &goal.description,
                &goal.reward_text,
                &goal.deadline,
                goal.is_completed as i32,
                &goal.completed_at,
                &goal.created_at,
                goal.archived as i32
            ],
        )?;
    }
    for checkpoint in &data.checkpoints {
        conn.execute(
            "INSERT INTO checkpoints
                (id, goal_id, description, completed, completed_at, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &checkpoint.id,
                &checkpoint.goal_id,
                &checkpoint.description,
                checkpoint.completed as i32,
                &checkpoint.completed_at,
                checkpoint.sort_order
            ],
        )?;
    }
    for achievement in &data.achievements {
        conn.execute(
            "INSERT INTO achievements
                (id, title, description, icon_path, achievement_type, source_id, achieved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &achievement.id,
                &achievement.title,
                &achievement.description,
                &achievement.icon_path,
                &achievement.achievement_type,
                &achievement.source_id,
                &achievement.achieved_at
            ],
        )?;
    }
    for (key, value) in &data.settings {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
    }
    Ok(())
}
