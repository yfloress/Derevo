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

use super::DbError;
use rusqlite::Connection;

pub fn up(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS habits (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            color TEXT NOT NULL DEFAULT '#8b5cf6',
            category TEXT NOT NULL DEFAULT 'mind',
            created_at TEXT NOT NULL,
            archived INTEGER NOT NULL DEFAULT 0,
            -- Local time of day as HH:MM. NULL means no reminder.
            reminder_time TEXT,
            schedule_kind TEXT NOT NULL DEFAULT 'daily'
                CHECK(schedule_kind IN ('daily', 'weekdays', 'times_per_week')),
            -- Weekday numbers for the weekdays kind, Sunday is 0: '1,3,5'.
            schedule_days TEXT,
            -- Completions a week must reach, for the times_per_week kind.
            target_per_period INTEGER
        );

        CREATE TABLE IF NOT EXISTS habit_logs (
            id TEXT PRIMARY KEY NOT NULL,
            habit_id TEXT NOT NULL,
            completed_date TEXT NOT NULL,
            FOREIGN KEY (habit_id) REFERENCES habits(id) ON DELETE CASCADE,
            UNIQUE(habit_id, completed_date)
        );

        CREATE INDEX IF NOT EXISTS idx_habits_archived ON habits(archived);
        CREATE INDEX IF NOT EXISTS idx_habit_logs_habit_id ON habit_logs(habit_id);
        CREATE INDEX IF NOT EXISTS idx_habit_logs_date ON habit_logs(completed_date);
        CREATE INDEX IF NOT EXISTS idx_habit_logs_habit_date ON habit_logs(habit_id, completed_date);

        CREATE TABLE IF NOT EXISTS streak_rewards (
            id TEXT PRIMARY KEY NOT NULL,
            habit_id TEXT NOT NULL,
            is_consecutive INTEGER NOT NULL DEFAULT 1,
            target_days INTEGER,
            target_total INTEGER,
            created_at TEXT NOT NULL,
            FOREIGN KEY (habit_id) REFERENCES habits(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_streak_rewards_habit ON streak_rewards(habit_id);

        CREATE TABLE IF NOT EXISTS milestones (
            id TEXT PRIMARY KEY NOT NULL,
            reward_id TEXT NOT NULL,
            target_days INTEGER NOT NULL,
            reward_text TEXT NOT NULL,
            unlocked INTEGER NOT NULL DEFAULT 0,
            unlocked_at TEXT,
            FOREIGN KEY (reward_id) REFERENCES streak_rewards(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_milestones_reward ON milestones(reward_id);

        CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            reward_text TEXT NOT NULL,
            deadline TEXT,
            is_completed INTEGER NOT NULL DEFAULT 0,
            completed_at TEXT,
            created_at TEXT NOT NULL,
            archived INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_goals_completed ON goals(is_completed);

        CREATE TABLE IF NOT EXISTS checkpoints (
            id TEXT PRIMARY KEY NOT NULL,
            goal_id TEXT NOT NULL,
            description TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            completed_at TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_checkpoints_goal ON checkpoints(goal_id);

        CREATE TABLE IF NOT EXISTS achievements (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            icon_path TEXT NOT NULL,
            achievement_type TEXT NOT NULL CHECK(achievement_type IN ('streak', 'goal')),
            source_id TEXT NOT NULL,
            achieved_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_achievements_type ON achievements(achievement_type);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}
