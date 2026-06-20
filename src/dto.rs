// Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
// Copyright (C) 2026  Kyronix
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

#[derive(Debug, Clone, Serialize)]
pub struct HabitDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    pub days: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HabitsResponse {
    pub habits: Vec<HabitDto>,
    pub month: i32,
    pub year: i32,
    pub days_in_month: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HabitInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HabitToggleInput {
    pub habit_id: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HabitSummary {
    pub habit_id: String,
    pub current_streak: i32,
    pub best_streak: i32,
    pub completion_rate: f64,
    pub last_30_days: i32,
    pub best_day: Option<String>,
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
    pub weekly_summary: String,
    pub insight: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct StreakRewardInput {
    pub id: Option<String>,
    pub habit_id: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
    pub milestones: Vec<MilestoneInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MilestoneInput {
    pub id: Option<String>,
    pub target_days: i32,
    pub reward_text: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct GoalInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub checkpoints: Vec<CheckpointInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckpointInput {
    pub id: Option<String>,
    pub description: String,
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
