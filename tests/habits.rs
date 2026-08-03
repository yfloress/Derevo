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

mod common;

use common::{TestDb, days_ago, habit_input};
use derevo::error::AppError;
use derevo::svc::HabitService;
use derevo::svc::habits::{HabitInput, ScheduleInput};

fn create(db: &TestDb, name: &str, category: &str) -> String {
    HabitService::create_habit(db, habit_input(name, category)).expect("habit created")
}

#[test]
fn rejects_a_blank_name() {
    let db = TestDb::new();
    let err = HabitService::create_habit(&db, habit_input("   ", "health")).unwrap_err();
    assert_eq!(err.kind(), "validation");
}

#[test]
fn rejects_a_malformed_colour() {
    let db = TestDb::new();
    for colour in ["8b5cf6", "#8b5cf", "#zzzzzz", ""] {
        let err = HabitService::create_habit(
            &db,
            HabitInput {
                color: colour.to_string(),
                ..habit_input("Read", "mind")
            },
        )
        .unwrap_err();
        assert_eq!(
            err.kind(),
            "validation",
            "colour {colour} should be refused"
        );
    }
}

#[test]
fn normalises_the_reminder_time() {
    let db = TestDb::new();
    let id = HabitService::create_habit(
        &db,
        HabitInput {
            reminder_time: Some("7:5".to_string()),
            ..habit_input("Stretch", "body")
        },
    )
    .expect("habit created");
    let habit = db.get_habit(&id).unwrap().unwrap();
    assert_eq!(habit.reminder_time.as_deref(), Some("07:05"));
}

#[test]
fn an_empty_reminder_means_no_reminder() {
    let db = TestDb::new();
    let id = HabitService::create_habit(
        &db,
        HabitInput {
            reminder_time: Some("  ".to_string()),
            ..habit_input("Stretch", "body")
        },
    )
    .expect("habit created");
    assert_eq!(db.get_habit(&id).unwrap().unwrap().reminder_time, None);
}

#[test]
fn rejects_an_impossible_reminder_time() {
    let db = TestDb::new();
    for time in ["24:00", "12:60", "noon", "12"] {
        let err = HabitService::create_habit(
            &db,
            HabitInput {
                reminder_time: Some(time.to_string()),
                ..habit_input("Stretch", "body")
            },
        )
        .unwrap_err();
        assert_eq!(err.kind(), "validation", "time {time} should be refused");
    }
}

#[test]
fn a_category_that_differs_only_in_case_reuses_the_stored_spelling() {
    let db = TestDb::new();
    create(&db, "Walk", "salud");
    create(&db, "Water", "Salud");
    create(&db, "Sleep", "  SALUD  ");

    let categories = db.get_categories().unwrap();
    assert_eq!(categories, vec!["salud".to_string()]);
}

#[test]
fn collapses_whitespace_inside_a_new_category() {
    let db = TestDb::new();
    let id = create(&db, "Practise", "  deep   work  ");
    assert_eq!(db.get_habit(&id).unwrap().unwrap().category, "deep work");
}

#[test]
fn an_empty_category_falls_back_to_the_default() {
    let db = TestDb::new();
    let id = create(&db, "Journal", "   ");
    assert_eq!(
        db.get_habit(&id).unwrap().unwrap().category,
        derevo::svc::habits::DEFAULT_CATEGORY
    );
}

#[test]
fn editing_a_habit_also_canonicalises_its_category() {
    let db = TestDb::new();
    create(&db, "Walk", "salud");
    let id = create(&db, "Water", "otro");

    HabitService::update_habit(&db, id.clone(), habit_input("Water", "SALUD"))
        .expect("habit updated");

    assert_eq!(db.get_habit(&id).unwrap().unwrap().category, "salud");
}

#[test]
fn updating_a_habit_that_is_gone_reports_habit_not_found() {
    let db = TestDb::new();
    let err = HabitService::update_habit(
        &db,
        "does-not-exist".to_string(),
        habit_input("Water", "health"),
    )
    .unwrap_err();
    assert_eq!(err.kind(), "habit-not-found");
    assert!(matches!(err, AppError::Database(_)));
}

#[test]
fn a_weekday_schedule_is_stored_sorted_and_deduplicated() {
    let db = TestDb::new();
    let id = HabitService::create_habit(
        &db,
        HabitInput {
            schedule: ScheduleInput {
                kind: "weekdays".to_string(),
                days: vec![5, 1, 3, 1],
                target_per_period: None,
            },
            ..habit_input("Gym", "body")
        },
    )
    .expect("habit created");

    let habit = db.get_habit(&id).unwrap().unwrap();
    assert_eq!(habit.schedule_days.as_deref(), Some("1,3,5"));
    assert_eq!(habit.target_per_period, None);
}

#[test]
fn a_weekday_schedule_with_no_days_is_refused() {
    let db = TestDb::new();
    let err = HabitService::create_habit(
        &db,
        HabitInput {
            schedule: ScheduleInput {
                kind: "weekdays".to_string(),
                days: vec![],
                target_per_period: None,
            },
            ..habit_input("Gym", "body")
        },
    )
    .unwrap_err();
    assert_eq!(err.kind(), "validation");
}

#[test]
fn a_weekly_target_outside_one_to_seven_is_refused() {
    let db = TestDb::new();
    for target in [0, 8] {
        let err = HabitService::create_habit(
            &db,
            HabitInput {
                schedule: ScheduleInput {
                    kind: "times_per_week".to_string(),
                    days: vec![],
                    target_per_period: Some(target),
                },
                ..habit_input("Gym", "body")
            },
        )
        .unwrap_err();
        assert_eq!(
            err.kind(),
            "validation",
            "target {target} should be refused"
        );
    }
}

#[test]
fn an_unknown_schedule_kind_is_refused() {
    let db = TestDb::new();
    let err = HabitService::create_habit(
        &db,
        HabitInput {
            schedule: ScheduleInput {
                kind: "every-full-moon".to_string(),
                days: vec![],
                target_per_period: None,
            },
            ..habit_input("Gym", "body")
        },
    )
    .unwrap_err();
    assert_eq!(err.kind(), "validation");
}

#[test]
fn switching_a_habit_back_to_daily_clears_the_old_schedule() {
    let db = TestDb::new();
    let id = HabitService::create_habit(
        &db,
        HabitInput {
            schedule: ScheduleInput {
                kind: "times_per_week".to_string(),
                days: vec![],
                target_per_period: Some(3),
            },
            ..habit_input("Gym", "body")
        },
    )
    .expect("habit created");

    HabitService::update_habit(&db, id.clone(), habit_input("Gym", "body")).expect("habit updated");

    let habit = db.get_habit(&id).unwrap().unwrap();
    assert_eq!(habit.schedule_kind, "daily");
    assert_eq!(
        habit.target_per_period, None,
        "the old target must not linger"
    );
    assert_eq!(habit.schedule_days, None);
}

#[test]
fn toggling_adds_then_removes_the_log() {
    let db = TestDb::new();
    let id = create(&db, "Walk", "body");
    let today = days_ago(0);

    assert!(HabitService::toggle_habit_completion(&db, id.clone(), today.clone()).unwrap());
    assert_eq!(db.get_habit_logs(&today, &today).unwrap().len(), 1);

    assert!(!HabitService::toggle_habit_completion(&db, id, today.clone()).unwrap());
    assert!(db.get_habit_logs(&today, &today).unwrap().is_empty());
}

#[test]
fn archiving_hides_the_habit_but_keeps_its_logs() {
    let db = TestDb::new();
    let id = create(&db, "Walk", "body");
    HabitService::toggle_habit_completion(&db, id.clone(), days_ago(1)).unwrap();
    HabitService::toggle_habit_completion(&db, id.clone(), days_ago(2)).unwrap();

    HabitService::archive_habit(&db, id.clone()).unwrap();

    assert!(HabitService::get_habits(&db).unwrap().is_empty());
    let archived = HabitService::get_archived_habits(&db).unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].0.id, id);
    assert_eq!(archived[0].1, 2, "logs should survive archiving");
}

#[test]
fn restoring_brings_the_habit_back_to_tracking() {
    let db = TestDb::new();
    let id = create(&db, "Walk", "body");
    HabitService::archive_habit(&db, id.clone()).unwrap();
    HabitService::restore_habit(&db, id.clone()).unwrap();

    assert_eq!(HabitService::get_habits(&db).unwrap().len(), 1);
    assert!(HabitService::get_archived_habits(&db).unwrap().is_empty());
}

#[test]
fn archiving_something_that_does_not_exist_is_an_error() {
    let db = TestDb::new();
    let err = HabitService::archive_habit(&db, "nope".to_string()).unwrap_err();
    assert_eq!(err.kind(), "habit-not-found");
}

#[test]
fn deleting_a_habit_takes_its_logs_with_it() {
    let db = TestDb::new();
    let id = create(&db, "Walk", "body");
    let day = days_ago(0);
    HabitService::toggle_habit_completion(&db, id.clone(), day.clone()).unwrap();

    HabitService::delete_habit(&db, id).unwrap();

    assert!(HabitService::get_habits(&db).unwrap().is_empty());
    assert!(db.get_habit_logs(&day, &day).unwrap().is_empty());
}

#[test]
fn archived_habits_stay_out_of_the_category_suggestions() {
    let db = TestDb::new();
    let id = create(&db, "Walk", "body");
    create(&db, "Read", "mind");
    HabitService::archive_habit(&db, id).unwrap();

    assert_eq!(db.get_categories().unwrap(), vec!["mind".to_string()]);
}
