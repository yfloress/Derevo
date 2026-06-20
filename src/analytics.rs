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

use crate::db::Database;
use crate::error::AppError;
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone)]
pub struct WeekdayEfficiency {
    pub day_name: String,
    pub day_short: String,
    pub completion_rate: f32,
    pub is_best: bool,
}

#[derive(Debug, Clone)]
pub struct CategoryDistributionPoint {
    pub category: String,
    pub count: i32,
}

#[derive(Debug, Clone)]
pub struct HabitAnalytics {
    pub weekday_data: Vec<WeekdayEfficiency>,
    pub category_data: Vec<CategoryDistributionPoint>,
}

pub fn compute_habit_analytics(db: &Database, days: i32) -> Result<HabitAnalytics, AppError> {
    let today = chrono::Local::now().date_naive();
    let start_date = today
        .checked_sub_signed(chrono::Duration::days(days as i64))
        .unwrap_or(today);

    let logs = db.get_habit_logs(
        &start_date.format("%Y-%m-%d").to_string(),
        &today.format("%Y-%m-%d").to_string(),
    )?;
    let habits = db.get_habits()?;

    let active_ids: std::collections::HashSet<&str> =
        habits.iter().map(|h| h.id.as_str()).collect();
    let habit_start_dates: Vec<NaiveDate> = habits
        .iter()
        .filter_map(|h| {
            NaiveDate::parse_from_str(h.created_at.get(..10).unwrap_or(""), "%Y-%m-%d").ok()
        })
        .collect();

    let mut weekday_completions: [i32; 7] = [0; 7];
    let mut weekday_available: [i32; 7] = [0; 7];

    let mut cursor = start_date;
    loop {
        let weekday_idx = cursor.weekday().num_days_from_monday() as usize;
        let available = habit_start_dates.iter().filter(|&&c| c <= cursor).count() as i32;
        weekday_available[weekday_idx] += available;
        cursor = match cursor.succ_opt() {
            Some(c) => c,
            None => break,
        };
        if cursor > today {
            break;
        }
    }

    for log in &logs {
        if !active_ids.contains(log.habit_id.as_str()) {
            continue;
        }
        if let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") {
            let weekday_idx = date.weekday().num_days_from_monday() as usize;
            weekday_completions[weekday_idx] += 1;
        }
    }

    let weekday_rates: Vec<(usize, f32)> = (0..7)
        .map(|i| {
            let rate = if weekday_available[i] > 0 {
                (weekday_completions[i] as f32 / weekday_available[i] as f32).min(1.0)
            } else {
                0.0
            };
            (i, rate)
        })
        .collect();

    let max_rate = weekday_rates
        .iter()
        .map(|(_, rate)| *rate)
        .fold(0.0_f32, f32::max);
    let day_names = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    let day_shorts = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let weekday_data: Vec<WeekdayEfficiency> = weekday_rates
        .iter()
        .map(|(i, rate)| WeekdayEfficiency {
            day_name: day_names[*i].to_string(),
            day_short: day_shorts[*i].to_string(),
            completion_rate: *rate,
            is_best: (*rate - max_rate).abs() < 0.001 && max_rate > 0.0,
        })
        .collect();

    let mut category_counts: std::collections::HashMap<String, i32> =
        std::collections::HashMap::new();
    let habit_categories: std::collections::HashMap<&str, &str> = habits
        .iter()
        .map(|h| (h.id.as_str(), h.category.as_str()))
        .collect();
    for log in &logs {
        if let Some(cat) = habit_categories.get(log.habit_id.as_str()) {
            *category_counts.entry((*cat).to_string()).or_insert(0) += 1;
        }
    }
    let mut category_data: Vec<CategoryDistributionPoint> = category_counts
        .into_iter()
        .map(|(category, count)| CategoryDistributionPoint { category, count })
        .collect();
    category_data.sort_by_key(|b| std::cmp::Reverse(b.count));

    Ok(HabitAnalytics {
        weekday_data,
        category_data,
    })
}
