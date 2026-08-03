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

//! Streak arithmetic, kept away from the database so the rules can be read and
//! tested on their own. Everything here takes the dates it needs as input.

use crate::models::Schedule;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::BTreeSet;

/// Days that count towards a weekly target, Monday through Sunday.
const DAYS_PER_WEEK: i64 = 7;

/// How far back a streak walk will go before giving up. A streak longer than
/// this is not worth the loop, and it stops a corrupt date from spinning.
const MAX_LOOKBACK_DAYS: i64 = 366 * 20;

/// Days on which the habit is expected. `times_per_week` has no fixed days, so
/// every day is a chance to make progress.
pub fn is_due_on(schedule: &Schedule, date: NaiveDate) -> bool {
    match schedule {
        Schedule::Daily | Schedule::TimesPerWeek(_) => true,
        Schedule::Weekdays(days) => days.contains(&date.weekday().num_days_from_sunday()),
    }
}

/// The streak as of `today`, in the unit the schedule counts in: days for
/// `daily` and `weekdays`, satisfied weeks for `times_per_week`.
///
/// The current period never breaks a streak — the day (or the week) is not over
/// yet, so an unticked today leaves yesterday's streak standing.
pub fn current_streak(schedule: &Schedule, dates: &BTreeSet<NaiveDate>, today: NaiveDate) -> i32 {
    match schedule {
        Schedule::TimesPerWeek(target) => current_week_streak(*target, dates, today),
        _ => current_day_streak(schedule, dates, today),
    }
}

fn current_day_streak(schedule: &Schedule, dates: &BTreeSet<NaiveDate>, today: NaiveDate) -> i32 {
    let mut streak = 0;
    let mut cursor = today;
    for _ in 0..MAX_LOOKBACK_DAYS {
        if !is_due_on(schedule, cursor) {
            // Not scheduled: neither a success nor a miss, so step over it.
            cursor = match cursor.pred_opt() {
                Some(day) => day,
                None => break,
            };
            continue;
        }
        if dates.contains(&cursor) {
            streak += 1;
        } else if cursor != today {
            break;
        }
        cursor = match cursor.pred_opt() {
            Some(day) => day,
            None => break,
        };
    }
    streak
}

fn current_week_streak(target: i32, dates: &BTreeSet<NaiveDate>, today: NaiveDate) -> i32 {
    let target = target.max(1);
    let mut streak = 0;
    let mut week_start = monday_of(today);
    for _ in 0..(MAX_LOOKBACK_DAYS / DAYS_PER_WEEK) {
        let week_end = week_start + Duration::days(DAYS_PER_WEEK - 1);
        let done = dates.range(week_start..=week_end).count() as i32;
        if done >= target {
            streak += 1;
        } else if week_start != monday_of(today) {
            break;
        }
        week_start = match week_start.checked_sub_signed(Duration::days(DAYS_PER_WEEK)) {
            Some(day) => day,
            None => break,
        };
    }
    streak
}

/// The longest run ever reached, in the same unit as [`current_streak`].
pub fn best_streak(schedule: &Schedule, dates: &BTreeSet<NaiveDate>) -> i32 {
    let (Some(first), Some(last)) = (dates.iter().next(), dates.iter().next_back()) else {
        return 0;
    };

    match schedule {
        Schedule::TimesPerWeek(target) => {
            let target = (*target).max(1);
            let mut best = 0;
            let mut run = 0;
            let mut week_start = monday_of(*first);
            while week_start <= *last {
                let week_end = week_start + Duration::days(DAYS_PER_WEEK - 1);
                if dates.range(week_start..=week_end).count() as i32 >= target {
                    run += 1;
                    best = best.max(run);
                } else {
                    run = 0;
                }
                week_start += Duration::days(DAYS_PER_WEEK);
            }
            best
        }
        _ => {
            let mut best = 0;
            let mut run = 0;
            let mut cursor = *first;
            while cursor <= *last {
                if is_due_on(schedule, cursor) {
                    if dates.contains(&cursor) {
                        run += 1;
                        best = best.max(run);
                    } else {
                        run = 0;
                    }
                }
                cursor = match cursor.succ_opt() {
                    Some(day) => day,
                    None => break,
                };
            }
            best
        }
    }
}

/// How many completions the schedule asks for between the two dates. This is
/// what a completion rate divides by: measuring a three-times-a-week habit
/// against every day of the month would cap it at 43%.
pub fn expected_between(schedule: &Schedule, start: NaiveDate, end: NaiveDate) -> i32 {
    if end < start {
        return 0;
    }
    let span = (end - start).num_days() + 1;
    match schedule {
        Schedule::Daily => span as i32,
        Schedule::Weekdays(days) => {
            if days.is_empty() {
                return 0;
            }
            let mut count = 0;
            let mut cursor = start;
            while cursor <= end {
                if days.contains(&cursor.weekday().num_days_from_sunday()) {
                    count += 1;
                }
                cursor = match cursor.succ_opt() {
                    Some(day) => day,
                    None => break,
                };
            }
            count
        }
        // Prorated rather than rounded up to whole weeks, so a 30-day window
        // does not demand a fifth week that is only partly inside it.
        Schedule::TimesPerWeek(target) => {
            ((*target).max(0) as i64 * span / DAYS_PER_WEEK).max(0) as i32
        }
    }
}

fn monday_of(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

/// Sunday is 0, matching both `chrono`'s `num_days_from_sunday` and the
/// frontend's `Date.getDay()`.
pub fn weekday_index(date: NaiveDate) -> u32 {
    date.weekday().num_days_from_sunday()
}

/// Used when validating a stored schedule: 0..=6 only.
pub fn weekday_from_index(index: u32) -> Option<Weekday> {
    match index {
        0 => Some(Weekday::Sun),
        1 => Some(Weekday::Mon),
        2 => Some(Weekday::Tue),
        3 => Some(Weekday::Wed),
        4 => Some(Weekday::Thu),
        5 => Some(Weekday::Fri),
        6 => Some(Weekday::Sat),
        _ => None,
    }
}
