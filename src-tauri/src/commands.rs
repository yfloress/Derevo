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

use chrono::{Datelike, NaiveDate};
use derevo::analytics::compute_habit_analytics;
use derevo::db::Database;
use derevo::dto::{
    AchievementDto, ArchivedHabitDto, CheckpointDto, ErrorDto, GoalDto, HabitAnalyticsResponse,
    HabitDto, HabitSummary, HabitsResponse, HeatmapDay, HeatmapResponse, ImportSummary,
    MilestoneDto, RadarChartData, SettingsDto, StreakRewardDto, WeekdayChartData,
};
use derevo::error::{AppError, DbError};
use derevo::models::Schedule;
use derevo::streaks;
use derevo::svc::habits::{HabitInput, ScheduleInput};
use derevo::svc::{BackupService, HabitService, RewardsService, SettingsService};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

type CmdResult<T> = Result<T, ErrorDto>;

fn invalid(message: &str) -> ErrorDto {
    AppError::Validation(message.to_string()).into()
}

/// The schedule half of the habit form. Kept as one object so the three fields
/// travel together and cannot drift apart across invoke calls.
///
/// The rename matters: Tauri only camel-cases a command's own arguments, so
/// without it `targetPerPeriod` would land as an unknown field and quietly
/// default to None.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HabitScheduleInput {
    pub kind: String,
    #[serde(default)]
    pub days: Vec<u32>,
    #[serde(default)]
    pub target_per_period: Option<i32>,
}

impl From<HabitScheduleInput> for ScheduleInput {
    fn from(input: HabitScheduleInput) -> Self {
        ScheduleInput {
            kind: input.kind,
            days: input.days,
            target_per_period: input.target_per_period,
        }
    }
}

/// The whole habit form as one payload.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HabitFormInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    #[serde(default)]
    pub reminder_time: Option<String>,
    pub schedule: HabitScheduleInput,
}

impl From<HabitFormInput> for HabitInput {
    fn from(input: HabitFormInput) -> Self {
        HabitInput {
            name: input.name,
            description: input.description,
            color: input.color,
            category: input.category,
            reminder_time: input.reminder_time,
            schedule: input.schedule.into(),
        }
    }
}

// ── Habits CRUD ──

#[tauri::command]
pub fn fetch_habits(
    db: State<'_, Arc<Database>>,
    month: i32,
    year: i32,
) -> CmdResult<HabitsResponse> {
    let date = NaiveDate::from_ymd_opt(year, month as u32, 1)
        .ok_or_else(|| invalid("Invalid month/year"))?;
    let habits = HabitService::get_habits(&db)?;

    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1)
    };
    let days_in_month = next_month
        .map(|d| d.pred_opt().unwrap_or(d).day())
        .unwrap_or(30) as i32;

    let start_date = date.format("%Y-%m-%d").to_string();
    let end_date = NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { (month + 1) as u32 },
        1,
    )
    .and_then(|d| d.pred_opt())
    .map(|d| d.format("%Y-%m-%d").to_string())
    .unwrap_or_else(|| format!("{year}-{month:02}-{days_in_month:02}"));

    let logs = HabitService::get_habit_logs(&db, start_date, end_date).unwrap_or_default();

    let habit_dtos: Vec<HabitDto> = habits
        .into_iter()
        .map(|h| {
            let mut days = vec![false; (days_in_month + 1) as usize];
            for log in &logs {
                if log.habit_id == h.id
                    && let Ok(d) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d")
                {
                    let day = d.day() as usize;
                    if day < days.len() {
                        days[day] = true;
                    }
                }
            }
            let schedule_days = match h.schedule() {
                Schedule::Weekdays(days) => days,
                _ => Vec::new(),
            };
            HabitDto {
                id: h.id,
                name: h.name,
                description: h.description,
                color: h.color,
                category: h.category,
                reminder_time: h.reminder_time,
                schedule_kind: h.schedule_kind,
                schedule_days,
                target_per_period: h.target_per_period,
                days,
            }
        })
        .collect();

    Ok(HabitsResponse {
        habits: habit_dtos,
        month,
        year,
        days_in_month,
    })
}

#[tauri::command]
pub fn fetch_archived_habits(db: State<'_, Arc<Database>>) -> CmdResult<Vec<ArchivedHabitDto>> {
    let rows = HabitService::get_archived_habits(&db)?;
    Ok(rows
        .into_iter()
        .map(|(h, log_count)| ArchivedHabitDto {
            id: h.id,
            name: h.name,
            color: h.color,
            category: h.category,
            created_at: h.created_at,
            log_count,
        })
        .collect())
}

#[tauri::command]
pub fn fetch_categories(db: State<'_, Arc<Database>>) -> CmdResult<Vec<String>> {
    Ok(HabitService::get_categories(&db)?)
}

#[tauri::command]
pub fn create_habit(db: State<'_, Arc<Database>>, habit: HabitFormInput) -> CmdResult<String> {
    Ok(HabitService::create_habit(&db, habit.into())?)
}

#[tauri::command]
pub fn update_habit(
    db: State<'_, Arc<Database>>,
    id: String,
    habit: HabitFormInput,
) -> CmdResult<()> {
    Ok(HabitService::update_habit(&db, id, habit.into())?)
}

#[tauri::command]
pub fn archive_habit(db: State<'_, Arc<Database>>, id: String) -> CmdResult<()> {
    Ok(HabitService::archive_habit(&db, id)?)
}

#[tauri::command]
pub fn restore_habit(db: State<'_, Arc<Database>>, id: String) -> CmdResult<()> {
    Ok(HabitService::restore_habit(&db, id)?)
}

/// Permanent, logs included. `archive_habit` is the reversible option.
#[tauri::command]
pub fn delete_habit(db: State<'_, Arc<Database>>, id: String) -> CmdResult<()> {
    Ok(HabitService::delete_habit(&db, id)?)
}

#[tauri::command]
pub fn toggle_habit(db: State<'_, Arc<Database>>, habit_id: String, date: String) -> CmdResult<()> {
    HabitService::toggle_habit_completion(&db, habit_id.clone(), date)?;

    if let Ok(rewards) = RewardsService::get_streak_rewards_by_habit(&db, &habit_id) {
        for reward in rewards {
            let _ = RewardsService::check_and_unlock_milestones(&db, &reward.id);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn fetch_habit_summary(
    db: State<'_, Arc<Database>>,
    habit_id: String,
) -> CmdResult<HabitSummary> {
    let habit = db
        .get_habit(&habit_id)?
        .ok_or_else(|| ErrorDto::from(DbError::HabitNotFound))?;
    let schedule = habit.schedule();
    let logs = db.get_habit_logs("1970-01-01", "2099-12-31")?;
    let today = chrono::Local::now().date_naive();

    let dates: BTreeSet<NaiveDate> = logs
        .iter()
        .filter(|l| l.habit_id == habit_id)
        .filter_map(|l| NaiveDate::parse_from_str(&l.completed_date, "%Y-%m-%d").ok())
        .collect();

    let thirty_days_ago = today - chrono::Duration::days(29);
    let last_30 = dates.range(thirty_days_ago..=today).count() as i32;
    // Against the days the schedule actually asks for. Dividing by a flat 30
    // would cap a three-times-a-week habit at 43%.
    let expected = streaks::expected_between(&schedule, thirty_days_ago, today);
    let completion_rate = if expected > 0 {
        (last_30 as f64 / expected as f64).min(1.0)
    } else {
        0.0
    };

    Ok(HabitSummary {
        habit_id,
        current_streak: streaks::current_streak(&schedule, &dates, today),
        best_streak: streaks::best_streak(&schedule, &dates),
        completion_rate,
        last_30_days: last_30,
    })
}

// ── Heatmap ──

#[tauri::command]
pub fn fetch_heatmap(db: State<'_, Arc<Database>>, year: i32) -> CmdResult<HeatmapResponse> {
    let start = format!("{year}-01-01");
    let end = format!("{year}-12-31");
    let habits = HabitService::get_habits(&db)?;
    let schedules: Vec<_> = habits.iter().map(|h| h.schedule()).collect();
    let logs = db.get_habit_logs(&start, &end)?;

    let mut day_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for log in &logs {
        *day_counts.entry(log.completed_date.clone()).or_insert(0) += 1;
    }

    let mut days: Vec<HeatmapDay> = Vec::new();
    if let Some(mut cursor) = NaiveDate::from_ymd_opt(year, 1, 1) {
        let year_end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(cursor);
        while cursor <= year_end {
            let date_str = cursor.format("%Y-%m-%d").to_string();
            let count = day_counts.get(&date_str).copied().unwrap_or(0);
            // Only the habits due that day, or a day off would never light up.
            let due_count = schedules
                .iter()
                .filter(|schedule| streaks::is_due_on(schedule, cursor))
                .count() as f64;
            let intensity = if due_count > 0.0 {
                let ratio = count as f64 / due_count;
                if ratio == 0.0 {
                    0
                } else if ratio <= 0.25 {
                    1
                } else if ratio <= 0.5 {
                    2
                } else if ratio <= 0.75 {
                    3
                } else {
                    4
                }
            } else {
                0
            };
            days.push(HeatmapDay {
                date: date_str,
                intensity,
            });
            cursor = match cursor.succ_opt() {
                Some(d) => d,
                None => break,
            };
        }
    }
    Ok(HeatmapResponse { year, data: days })
}

// ── Analytics ──

#[tauri::command]
pub fn fetch_habit_analytics(
    db: State<'_, Arc<Database>>,
    days: Option<i32>,
) -> CmdResult<HabitAnalyticsResponse> {
    let analytics = compute_habit_analytics(&db, days.unwrap_or(90))?;

    let weekday_labels: Vec<String> = analytics
        .weekday_data
        .iter()
        .map(|w| w.day_short.clone())
        .collect();
    let weekday_values: Vec<f64> = analytics
        .weekday_data
        .iter()
        .map(|w| w.completion_rate as f64)
        .collect();

    let radar_categories: Vec<String> = analytics
        .category_data
        .iter()
        .map(|c| c.category.clone())
        .collect();
    let radar_values: Vec<f64> = analytics
        .category_data
        .iter()
        .map(|c| c.count as f64)
        .collect();
    let radar_max = radar_values.iter().cloned().fold(0.0_f64, f64::max);

    // The frontend translates this; sending a rendered English sentence would
    // leave a stray language in a Spanish interface.
    let best_weekday = analytics
        .weekday_data
        .iter()
        .position(|w| w.is_best)
        .map(|i| i as i32);

    Ok(HabitAnalyticsResponse {
        radar: RadarChartData {
            categories: radar_categories,
            values: radar_values,
            max_value: radar_max,
        },
        weekday_efficiency: WeekdayChartData {
            labels: weekday_labels,
            values: weekday_values,
        },
        best_weekday,
    })
}

// ── Settings ──

#[tauri::command]
pub fn fetch_settings(db: State<'_, Arc<Database>>) -> CmdResult<SettingsDto> {
    Ok(SettingsService::get_all(&db)?)
}

#[tauri::command]
pub fn set_theme(db: State<'_, Arc<Database>>, theme: String) -> CmdResult<()> {
    Ok(SettingsService::set_theme(&db, &theme)?)
}

#[tauri::command]
pub fn set_language(db: State<'_, Arc<Database>>, language: String) -> CmdResult<()> {
    Ok(SettingsService::set_language(&db, &language)?)
}

// ── Backup ──

#[tauri::command]
pub fn export_backup(db: State<'_, Arc<Database>>, path: String) -> CmdResult<ImportSummary> {
    Ok(BackupService::export_to_file(&db, &PathBuf::from(path))?)
}

#[tauri::command]
pub fn import_backup(db: State<'_, Arc<Database>>, path: String) -> CmdResult<ImportSummary> {
    Ok(BackupService::import_from_file(&db, &PathBuf::from(path))?)
}

// ── Streak Rewards ──

#[tauri::command]
pub fn fetch_rewards(db: State<'_, Arc<Database>>) -> CmdResult<Vec<StreakRewardDto>> {
    let rewards = RewardsService::get_streak_rewards(&db)?;
    let habits = HabitService::get_habits(&db)?;
    let habit_names: std::collections::HashMap<String, String> =
        habits.into_iter().map(|h| (h.id.clone(), h.name)).collect();

    let dtos: Vec<StreakRewardDto> = rewards
        .into_iter()
        .map(|r| {
            let milestones = RewardsService::get_milestones(&db, &r.id)
                .unwrap_or_default()
                .into_iter()
                .map(|m| MilestoneDto {
                    id: m.id,
                    target_days: m.target_days,
                    reward_text: m.reward_text,
                    unlocked: m.unlocked,
                    unlocked_at: m.unlocked_at,
                })
                .collect();
            let progress = RewardsService::get_streak_progress(&db, &r).unwrap_or(0);
            StreakRewardDto {
                id: r.id,
                habit_id: r.habit_id.clone(),
                habit_name: habit_names.get(&r.habit_id).cloned().unwrap_or_default(),
                is_consecutive: r.is_consecutive,
                target_days: r.target_days,
                target_total: r.target_total,
                current_progress: progress,
                milestones,
            }
        })
        .collect();
    Ok(dtos)
}

#[tauri::command]
pub fn create_streak_reward(
    db: State<'_, Arc<Database>>,
    habit_id: String,
    is_consecutive: bool,
    target_days: i32,
    target_total: i32,
) -> CmdResult<String> {
    Ok(RewardsService::create_streak_reward(
        &db,
        habit_id,
        is_consecutive,
        target_days,
        target_total,
    )?)
}

#[tauri::command]
pub fn update_streak_reward(
    db: State<'_, Arc<Database>>,
    id: String,
    habit_id: String,
    is_consecutive: bool,
    target_days: i32,
    target_total: i32,
    milestones: Vec<(i32, String)>,
) -> CmdResult<()> {
    Ok(RewardsService::update_streak_reward_with_milestones(
        &db,
        id,
        habit_id,
        is_consecutive,
        target_days,
        target_total,
        milestones,
    )?)
}

#[tauri::command]
pub fn delete_streak_reward(db: State<'_, Arc<Database>>, id: String) -> CmdResult<()> {
    Ok(RewardsService::delete_streak_reward(&db, id)?)
}

// ── Goals ──

#[tauri::command]
pub fn fetch_goals(db: State<'_, Arc<Database>>) -> CmdResult<Vec<GoalDto>> {
    let goals = RewardsService::get_goals(&db)?;
    let dtos: Vec<GoalDto> = goals
        .into_iter()
        .map(|g| {
            let checkpoints = RewardsService::get_checkpoints(&db, &g.id)
                .unwrap_or_default()
                .into_iter()
                .map(|cp| CheckpointDto {
                    id: cp.id,
                    description: cp.description,
                    completed: cp.completed,
                    completed_at: cp.completed_at,
                })
                .collect();
            GoalDto {
                id: g.id,
                name: g.name,
                description: g.description,
                reward_text: g.reward_text,
                deadline: g.deadline,
                is_completed: g.is_completed,
                completed_at: g.completed_at,
                checkpoints,
            }
        })
        .collect();
    Ok(dtos)
}

#[tauri::command]
pub fn create_goal(
    db: State<'_, Arc<Database>>,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
) -> CmdResult<String> {
    let desc_opt = if description.trim().is_empty() {
        None
    } else {
        Some(description)
    };
    let deadline_opt = if deadline.trim().is_empty() {
        None
    } else {
        Some(deadline)
    };
    Ok(RewardsService::create_goal(
        &db,
        name,
        desc_opt,
        reward_text,
        deadline_opt,
    )?)
}

#[tauri::command]
pub fn update_goal(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
) -> CmdResult<()> {
    Ok(RewardsService::update_goal(
        &db,
        id,
        name,
        description,
        reward_text,
        deadline,
    )?)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GoalCheckpointInput {
    pub id: String,
    pub text: String,
}

#[tauri::command]
pub fn update_goal_with_checkpoints(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
    checkpoints: Vec<GoalCheckpointInput>,
) -> CmdResult<()> {
    let mut cps: Vec<(Option<String>, String, i32)> = Vec::with_capacity(checkpoints.len());
    for (idx, checkpoint) in checkpoints.into_iter().take(4).enumerate() {
        if checkpoint.text.trim().is_empty() {
            return Err(invalid("Checkpoint description cannot be empty"));
        }
        let id_opt = if checkpoint.id.trim().is_empty() {
            None
        } else {
            Some(checkpoint.id)
        };
        cps.push((id_opt, checkpoint.text, (idx + 1) as i32));
    }
    Ok(RewardsService::update_goal_with_checkpoints(
        &db,
        id,
        name,
        description,
        reward_text,
        deadline,
        cps,
    )?)
}

#[tauri::command]
pub fn delete_goal(db: State<'_, Arc<Database>>, id: String) -> CmdResult<()> {
    Ok(RewardsService::delete_goal(&db, id)?)
}

#[tauri::command]
pub fn complete_goal(db: State<'_, Arc<Database>>, id: String) -> CmdResult<Option<String>> {
    Ok(RewardsService::complete_goal(&db, id)?)
}

#[tauri::command]
pub fn archive_goal(db: State<'_, Arc<Database>>, id: String) -> CmdResult<()> {
    Ok(RewardsService::archive_goal(&db, id)?)
}

// ── Checkpoints ──

#[tauri::command]
pub fn toggle_checkpoint(
    db: State<'_, Arc<Database>>,
    goal_id: String,
    checkpoint_id: String,
) -> CmdResult<()> {
    RewardsService::toggle_checkpoint(&db, goal_id, checkpoint_id)?;
    Ok(())
}

// ── Achievements ──

#[tauri::command]
pub fn fetch_achievements(db: State<'_, Arc<Database>>) -> CmdResult<Vec<AchievementDto>> {
    let achievements = RewardsService::get_achievements(&db)?;
    Ok(achievements
        .into_iter()
        .map(|a| AchievementDto {
            id: a.id,
            title: a.title,
            description: a.description,
            icon_path: a.icon_path,
            achievement_type: a.achievement_type,
            achieved_at: a.achieved_at,
        })
        .collect())
}
