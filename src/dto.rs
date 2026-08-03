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

use crate::error::{AppError, DbError};
use serde::Serialize;

/// Every command fails with this shape. `kind` is a stable identifier the
/// frontend translates; `message` is only a fallback for unexpected kinds.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorDto {
    pub kind: String,
    pub message: String,
}

impl From<AppError> for ErrorDto {
    fn from(err: AppError) -> Self {
        ErrorDto {
            kind: err.kind().to_string(),
            message: err.to_string(),
        }
    }
}

impl From<DbError> for ErrorDto {
    fn from(err: DbError) -> Self {
        AppError::Database(err).into()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HabitDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    pub reminder_time: Option<String>,
    pub days: Vec<bool>,
}

/// Archived habits never reach the tracking grid, so they carry no day cells.
#[derive(Debug, Clone, Serialize)]
pub struct ArchivedHabitDto {
    pub id: String,
    pub name: String,
    pub color: String,
    pub category: String,
    pub created_at: String,
    pub log_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingsDto {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportSummary {
    pub habits: i32,
    pub logs: i32,
    pub rewards: i32,
    pub goals: i32,
    pub achievements: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HabitsResponse {
    pub habits: Vec<HabitDto>,
    pub month: i32,
    pub year: i32,
    pub days_in_month: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HabitSummary {
    pub habit_id: String,
    pub current_streak: i32,
    pub best_streak: i32,
    pub completion_rate: f64,
    pub last_30_days: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapResponse {
    pub year: i32,
    pub data: Vec<HeatmapDay>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapDay {
    pub date: String,
    pub intensity: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HabitAnalyticsResponse {
    pub radar: RadarChartData,
    pub weekday_efficiency: WeekdayChartData,
    /// Index into `weekday_efficiency.labels` of the strongest day, or None
    /// when there is nothing logged yet. The frontend writes the sentence.
    pub best_weekday: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RadarChartData {
    pub categories: Vec<String>,
    pub values: Vec<f64>,
    pub max_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeekdayChartData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreakRewardDto {
    pub id: String,
    pub habit_id: String,
    pub habit_name: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
    pub current_progress: i32,
    pub milestones: Vec<MilestoneDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MilestoneDto {
    pub id: String,
    pub target_days: i32,
    pub reward_text: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub checkpoints: Vec<CheckpointDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckpointDto {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AchievementDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_path: String,
    pub achievement_type: String,
    pub achieved_at: String,
}
