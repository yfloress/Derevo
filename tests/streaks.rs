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

//! Pure streak arithmetic, on fixed dates rather than "today", so the results
//! do not shift depending on when the suite runs.

use chrono::NaiveDate;
use derevo::models::Schedule;
use derevo::streaks;
use std::collections::BTreeSet;

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("valid date")
}

fn dates(days: &[&str]) -> BTreeSet<NaiveDate> {
    days.iter().map(|d| date(d)).collect()
}

/// 2026-08-03 is a Monday, so the week runs to Sunday 2026-08-09.
const MONDAY: &str = "2026-08-03";
const TUESDAY: &str = "2026-08-04";
const WEDNESDAY: &str = "2026-08-05";
const THURSDAY: &str = "2026-08-06";
const FRIDAY: &str = "2026-08-07";
const SATURDAY: &str = "2026-08-08";
const SUNDAY: &str = "2026-08-09";

/// Sunday is 0, so Monday/Wednesday/Friday is 1, 3, 5.
fn mon_wed_fri() -> Schedule {
    Schedule::Weekdays(vec![1, 3, 5])
}

#[test]
fn a_daily_habit_counts_every_day_back() {
    let logs = dates(&[MONDAY, TUESDAY, WEDNESDAY]);
    assert_eq!(
        streaks::current_streak(&Schedule::Daily, &logs, date(WEDNESDAY)),
        3
    );
}

#[test]
fn a_daily_habit_breaks_on_a_gap() {
    let logs = dates(&[MONDAY, WEDNESDAY, THURSDAY]);
    assert_eq!(
        streaks::current_streak(&Schedule::Daily, &logs, date(THURSDAY)),
        2
    );
}

#[test]
fn an_unticked_today_does_not_break_a_daily_streak() {
    let logs = dates(&[MONDAY, TUESDAY]);
    assert_eq!(
        streaks::current_streak(&Schedule::Daily, &logs, date(WEDNESDAY)),
        2
    );
}

#[test]
fn a_weekday_habit_ignores_the_days_it_is_not_due() {
    // Tuesday and Thursday are blank, but the habit was never due on them.
    let logs = dates(&[MONDAY, WEDNESDAY, FRIDAY]);
    assert_eq!(
        streaks::current_streak(&mon_wed_fri(), &logs, date(FRIDAY)),
        3
    );
}

#[test]
fn a_weekday_habit_breaks_on_a_missed_scheduled_day() {
    // Wednesday was due and skipped, so only Friday counts.
    let logs = dates(&[MONDAY, FRIDAY]);
    assert_eq!(
        streaks::current_streak(&mon_wed_fri(), &logs, date(FRIDAY)),
        1
    );
}

#[test]
fn a_weekday_habit_survives_the_weekend_it_does_not_work_on() {
    let logs = dates(&[WEDNESDAY, FRIDAY]);
    // Saturday and Sunday are not scheduled, so Monday's check reaches Friday.
    assert_eq!(
        streaks::current_streak(&mon_wed_fri(), &logs, date("2026-08-10")),
        2
    );
}

#[test]
fn an_unticked_scheduled_today_does_not_break_the_streak() {
    let logs = dates(&[MONDAY, WEDNESDAY]);
    assert_eq!(
        streaks::current_streak(&mon_wed_fri(), &logs, date(FRIDAY)),
        2
    );
}

#[test]
fn a_weekly_target_counts_satisfied_weeks() {
    let previous = dates(&["2026-07-27", "2026-07-29", "2026-07-31"]);
    let mut logs = previous;
    logs.extend(dates(&[MONDAY, WEDNESDAY, FRIDAY]));
    assert_eq!(
        streaks::current_streak(&Schedule::TimesPerWeek(3), &logs, date(SUNDAY)),
        2
    );
}

#[test]
fn a_week_short_of_the_target_ends_the_run() {
    let mut logs = dates(&["2026-07-27", "2026-07-29"]);
    logs.extend(dates(&[MONDAY, WEDNESDAY, FRIDAY]));
    // The previous week only reached two of three.
    assert_eq!(
        streaks::current_streak(&Schedule::TimesPerWeek(3), &logs, date(SUNDAY)),
        1
    );
}

#[test]
fn an_unfinished_current_week_does_not_break_the_run() {
    let mut logs = dates(&["2026-07-27", "2026-07-29", "2026-07-31"]);
    logs.extend(dates(&[MONDAY]));
    // This week is at one of three, but it is not over yet.
    assert_eq!(
        streaks::current_streak(&Schedule::TimesPerWeek(3), &logs, date(TUESDAY)),
        1
    );
}

#[test]
fn the_best_daily_streak_is_the_longest_run_ever() {
    let logs = dates(&[
        "2026-07-01",
        "2026-07-02",
        "2026-07-03",
        "2026-07-10",
        "2026-07-11",
    ]);
    assert_eq!(streaks::best_streak(&Schedule::Daily, &logs), 3);
}

#[test]
fn the_best_weekday_streak_skips_the_unscheduled_days() {
    let logs = dates(&[MONDAY, WEDNESDAY, FRIDAY, "2026-08-12"]);
    // Mon, Wed, Fri, then Monday the 10th missed, then Wednesday the 12th.
    assert_eq!(streaks::best_streak(&mon_wed_fri(), &logs), 3);
}

#[test]
fn no_logs_means_no_best_streak() {
    assert_eq!(streaks::best_streak(&Schedule::Daily, &BTreeSet::new()), 0);
}

#[test]
fn a_daily_habit_expects_one_completion_per_day() {
    assert_eq!(
        streaks::expected_between(&Schedule::Daily, date(MONDAY), date(SUNDAY)),
        7
    );
}

#[test]
fn a_weekday_habit_only_expects_its_own_days() {
    assert_eq!(
        streaks::expected_between(&mon_wed_fri(), date(MONDAY), date(SUNDAY)),
        3
    );
}

#[test]
fn a_weekly_target_is_prorated_over_the_window() {
    // Three a week over 30 days is twelve, not fifteen.
    assert_eq!(
        streaks::expected_between(&Schedule::TimesPerWeek(3), date(MONDAY), date("2026-09-01")),
        12
    );
}

#[test]
fn a_habit_is_only_due_on_its_own_weekdays() {
    assert!(streaks::is_due_on(&mon_wed_fri(), date(MONDAY)));
    assert!(!streaks::is_due_on(&mon_wed_fri(), date(TUESDAY)));
    assert!(!streaks::is_due_on(&mon_wed_fri(), date(SATURDAY)));
}

#[test]
fn a_weekly_target_is_due_every_day_because_any_day_counts() {
    assert!(streaks::is_due_on(
        &Schedule::TimesPerWeek(3),
        date(SATURDAY)
    ));
}
