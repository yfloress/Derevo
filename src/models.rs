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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    pub created_at: String,
    pub archived: bool,
    /// Local time of day as "HH:MM", or None for no reminder.
    pub reminder_time: Option<String>,
}

impl Habit {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            color,
            category,
            created_at,
            archived: false,
            reminder_time: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitLog {
    pub id: String,
    pub habit_id: String,
    pub completed_date: String,
}

impl HabitLog {
    pub fn new(id: String, habit_id: String, completed_date: String) -> Self {
        Self {
            id,
            habit_id,
            completed_date,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakReward {
    pub id: String,
    pub habit_id: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
    pub created_at: String,
}

impl StreakReward {
    pub fn new(
        id: String,
        habit_id: String,
        is_consecutive: bool,
        target_days: Option<i32>,
        target_total: Option<i32>,
    ) -> Self {
        Self {
            id,
            habit_id,
            is_consecutive,
            target_days,
            target_total,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub reward_id: String,
    pub target_days: i32,
    pub reward_text: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

impl Milestone {
    pub fn new(id: String, reward_id: String, target_days: i32, reward_text: String) -> Self {
        Self {
            id,
            reward_id,
            target_days,
            reward_text,
            unlocked: false,
            unlocked_at: None,
        }
    }

    pub fn unlock(&mut self) {
        self.unlocked = true;
        self.unlocked_at = Some(chrono::Local::now().to_rfc3339());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub archived: bool,
}

impl Goal {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        reward_text: String,
        deadline: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            reward_text,
            deadline,
            is_completed: false,
            completed_at: None,
            created_at: chrono::Local::now().to_rfc3339(),
            archived: false,
        }
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
        self.completed_at = Some(chrono::Local::now().to_rfc3339());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub sort_order: i32,
}

impl Checkpoint {
    pub fn new(id: String, goal_id: String, description: String, sort_order: i32) -> Self {
        Self {
            id,
            goal_id,
            description,
            completed: false,
            completed_at: None,
            sort_order,
        }
    }

    pub fn toggle(&mut self) -> bool {
        self.completed = !self.completed;
        self.completed_at = if self.completed {
            Some(chrono::Local::now().to_rfc3339())
        } else {
            None
        };
        self.completed
    }
}

/// Everything the database holds, as written to and read back from a backup
/// file. Adding a field here changes the on-disk format — bump
/// `svc::backup::FORMAT_VERSION` when that happens.
/// Missing sections default to empty so a file written by an older version, or
/// one that simply had nothing to say about rewards, still restores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BackupData {
    pub habits: Vec<Habit>,
    pub habit_logs: Vec<HabitLog>,
    pub streak_rewards: Vec<StreakReward>,
    pub milestones: Vec<Milestone>,
    pub goals: Vec<Goal>,
    pub checkpoints: Vec<Checkpoint>,
    pub achievements: Vec<Achievement>,
    pub settings: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_path: String,
    pub achievement_type: String,
    pub source_id: String,
    pub achieved_at: String,
}

impl Achievement {
    pub fn new(
        id: String,
        title: String,
        description: String,
        icon_path: String,
        achievement_type: String,
        source_id: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            icon_path,
            achievement_type,
            source_id,
            achieved_at: chrono::Local::now().to_rfc3339(),
        }
    }
}
