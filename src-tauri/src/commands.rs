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

use chrono::{Datelike, NaiveDate};
use derevo::analytics::compute_habit_analytics;
use derevo::db::Database;
use derevo::dto::{
    AchievementDto, CheckpointDto, GoalDto, HabitAnalyticsResponse, HabitDto, HabitSummary,
    HabitsResponse, HeatmapDay, HeatmapResponse, MilestoneDto, RadarChartData, StreakRewardDto,
    WeekdayChartData,
};
use derevo::svc::{HabitService, RewardsService};
use std::sync::Arc;
use tauri::State;

fn to_string<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ── Habits CRUD ──

#[tauri::command]
pub fn fetch_habits(
    db: State<'_, Arc<Database>>,
    month: i32,
    year: i32,
) -> Result<HabitsResponse, String> {
    let date = NaiveDate::from_ymd_opt(year, month as u32, 1)
        .ok_or_else(|| "Invalid month/year".to_string())?;
    let habits = HabitService::get_habits(&db).map_err(to_string)?;

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
        .filter(|h| !h.archived)
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
            HabitDto {
                id: h.id,
                name: h.name,
                description: h.description,
                color: h.color,
                category: h.category,
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
pub fn create_habit(
    db: State<'_, Arc<Database>>,
    name: String,
    description: Option<String>,
    color: String,
    category: String,
) -> Result<String, String> {
    HabitService::create_habit(&db, name, description, color, category).map_err(to_string)
}

#[tauri::command]
pub fn update_habit(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    description: Option<String>,
    color: String,
    category: String,
) -> Result<(), String> {
    HabitService::update_habit(&db, id, name, description, color, category, false)
        .map_err(to_string)
}

#[tauri::command]
pub fn delete_habit(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    HabitService::delete_habit(&db, id).map_err(to_string)
}

#[tauri::command]
pub fn toggle_habit(
    db: State<'_, Arc<Database>>,
    habit_id: String,
    date: String,
) -> Result<(), String> {
    HabitService::toggle_habit_completion(&db, habit_id.clone(), date).map_err(to_string)?;

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
) -> Result<HabitSummary, String> {
    let logs = db
        .get_habit_logs("1970-01-01", "2099-12-31")
        .map_err(to_string)?;
    let today = chrono::Local::now().date_naive();

    let mut dates: Vec<NaiveDate> = logs
        .iter()
        .filter(|l| l.habit_id == habit_id)
        .filter_map(|l| NaiveDate::parse_from_str(&l.completed_date, "%Y-%m-%d").ok())
        .collect();
    dates.sort();
    dates.dedup();

    let mut current_streak = 0i32;
    let mut cursor = today;
    loop {
        if dates.binary_search(&cursor).is_ok() {
            current_streak += 1;
            cursor = match cursor.pred_opt() {
                Some(d) => d,
                None => break,
            };
        } else {
            break;
        }
    }

    let mut best_streak = 0i32;
    let mut streak = 0i32;
    let mut prev: Option<NaiveDate> = None;
    for d in &dates {
        if let Some(p) = prev {
            if *d == p.succ_opt().unwrap_or(p) {
                streak += 1;
            } else {
                streak = 1;
            }
        } else {
            streak = 1;
        }
        if streak > best_streak {
            best_streak = streak;
        }
        prev = Some(*d);
    }

    let thirty_days_ago = today - chrono::Duration::days(30);
    let last_30 = dates.iter().filter(|d| **d >= thirty_days_ago).count() as i32;
    let completion_rate = last_30 as f64 / 30.0;

    Ok(HabitSummary {
        habit_id,
        current_streak,
        best_streak,
        completion_rate,
        last_30_days: last_30,
        best_day: None,
    })
}

// ── Heatmap ──

#[tauri::command]
pub fn fetch_heatmap(db: State<'_, Arc<Database>>, year: i32) -> Result<HeatmapResponse, String> {
    let start = format!("{year}-01-01");
    let end = format!("{year}-12-31");
    let habits = HabitService::get_habits(&db).map_err(to_string)?;
    let habit_count = habits.iter().filter(|h| !h.archived).count() as f64;
    let logs = db.get_habit_logs(&start, &end).map_err(to_string)?;

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
            let intensity = if habit_count > 0.0 {
                let ratio = count as f64 / habit_count;
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
) -> Result<HabitAnalyticsResponse, String> {
    let analytics = compute_habit_analytics(&db, days.unwrap_or(90)).map_err(to_string)?;

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

    let best_day = analytics
        .weekday_data
        .iter()
        .find(|w| w.is_best)
        .map(|w| w.day_name.clone());
    let weekly_summary = match &best_day {
        Some(day) => format!("Your best day is {day}"),
        None => "No data yet".to_string(),
    };

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
        weekly_summary,
        insight: String::new(),
    })
}

// ── Streak Rewards ──

#[tauri::command]
pub fn fetch_rewards(db: State<'_, Arc<Database>>) -> Result<Vec<StreakRewardDto>, String> {
    let rewards = RewardsService::get_streak_rewards(&db).map_err(to_string)?;
    let habits = HabitService::get_habits(&db).map_err(to_string)?;
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
) -> Result<String, String> {
    let (days_opt, total_opt) = if is_consecutive {
        (None, None)
    } else {
        (Some(target_days), Some(target_total))
    };
    RewardsService::create_streak_reward(&db, habit_id, is_consecutive, days_opt, total_opt)
        .map_err(to_string)
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
) -> Result<(), String> {
    let (days_opt, total_opt) = if is_consecutive {
        (None, None)
    } else {
        (Some(target_days), Some(target_total))
    };
    RewardsService::update_streak_reward_with_milestones(
        &db,
        id,
        habit_id,
        is_consecutive,
        days_opt,
        total_opt,
        milestones,
    )
    .map_err(to_string)
}

#[tauri::command]
pub fn delete_streak_reward(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    RewardsService::delete_streak_reward(&db, id).map_err(to_string)
}

#[tauri::command]
pub fn add_milestone(
    db: State<'_, Arc<Database>>,
    reward_id: String,
    target_days: i32,
    reward_text: String,
) -> Result<String, String> {
    RewardsService::add_milestone(&db, reward_id, target_days, reward_text).map_err(to_string)
}

// ── Goals ──

#[tauri::command]
pub fn fetch_goals(db: State<'_, Arc<Database>>) -> Result<Vec<GoalDto>, String> {
    let goals = RewardsService::get_goals(&db).map_err(to_string)?;
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
) -> Result<String, String> {
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
    RewardsService::create_goal(&db, name, desc_opt, reward_text, deadline_opt).map_err(to_string)
}

#[tauri::command]
pub fn update_goal(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
) -> Result<(), String> {
    RewardsService::update_goal(&db, id, name, description, reward_text, deadline)
        .map_err(to_string)
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
) -> Result<(), String> {
    let count = checkpoints.len().min(4) as i32;
    let get = |i: usize| -> (String, String) {
        checkpoints
            .get(i)
            .map(|c| (c.id.clone(), c.text.clone()))
            .unwrap_or_default()
    };
    let (cp1_id, cp1_text) = get(0);
    let (cp2_id, cp2_text) = get(1);
    let (cp3_id, cp3_text) = get(2);
    let (cp4_id, cp4_text) = get(3);

    let mut cps: Vec<(Option<String>, String, i32)> = Vec::with_capacity(count as usize);
    let entries = [
        (cp1_id, cp1_text),
        (cp2_id, cp2_text),
        (cp3_id, cp3_text),
        (cp4_id, cp4_text),
    ];
    for (idx, (cp_id, cp_text)) in entries.into_iter().enumerate().take(count as usize) {
        if cp_text.trim().is_empty() {
            return Err("Checkpoint description cannot be empty".into());
        }
        let id_opt = if cp_id.trim().is_empty() {
            None
        } else {
            Some(cp_id)
        };
        cps.push((id_opt, cp_text, (idx + 1) as i32));
    }
    RewardsService::update_goal_with_checkpoints(
        &db,
        id,
        name,
        description,
        reward_text,
        deadline,
        cps,
    )
    .map_err(to_string)
}

#[tauri::command]
pub fn delete_goal(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    RewardsService::delete_goal(&db, id).map_err(to_string)
}

#[tauri::command]
pub fn complete_goal(db: State<'_, Arc<Database>>, id: String) -> Result<Option<String>, String> {
    RewardsService::complete_goal(&db, id).map_err(to_string)
}

#[tauri::command]
pub fn archive_goal(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    RewardsService::archive_goal(&db, id).map_err(to_string)
}

// ── Checkpoints ──

#[tauri::command]
pub fn add_checkpoint(
    db: State<'_, Arc<Database>>,
    goal_id: String,
    description: String,
) -> Result<String, String> {
    RewardsService::add_checkpoint(&db, goal_id, description).map_err(to_string)
}

#[tauri::command]
pub fn update_checkpoint(
    db: State<'_, Arc<Database>>,
    checkpoint_id: String,
    description: String,
) -> Result<(), String> {
    RewardsService::update_checkpoint(&db, checkpoint_id, description).map_err(to_string)
}

#[tauri::command]
pub fn delete_checkpoint(
    db: State<'_, Arc<Database>>,
    checkpoint_id: String,
) -> Result<(), String> {
    RewardsService::delete_checkpoint(&db, checkpoint_id).map_err(to_string)
}

#[tauri::command]
pub fn toggle_checkpoint(
    db: State<'_, Arc<Database>>,
    goal_id: String,
    checkpoint_id: String,
) -> Result<(), String> {
    RewardsService::toggle_checkpoint(&db, goal_id, checkpoint_id)
        .map(|_| ())
        .map_err(|_| "Toggle failed".to_string())
}

// ── Achievements ──

#[tauri::command]
pub fn fetch_achievements(db: State<'_, Arc<Database>>) -> Result<Vec<AchievementDto>, String> {
    let achievements = RewardsService::get_achievements(&db).map_err(to_string)?;
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
