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

use common::{TestDb, days_ago};
use derevo::svc::{BackupService, HabitService, RewardsService, SettingsService};
use std::path::PathBuf;

/// Backups are written next to the database under test, so the TestDb guard
/// cleans them up too.
fn backup_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "derevo-backup-{}-{name}.json",
        uuid::Uuid::new_v4()
    ))
}

fn seed(db: &TestDb) -> String {
    let id = HabitService::create_habit(
        db,
        "Walk".to_string(),
        Some("Around the block".to_string()),
        "#8b5cf6".to_string(),
        "body".to_string(),
        Some("08:30".to_string()),
    )
    .expect("habit");
    HabitService::toggle_habit_completion(db, id.clone(), days_ago(0)).unwrap();
    HabitService::toggle_habit_completion(db, id.clone(), days_ago(1)).unwrap();

    let reward =
        RewardsService::create_streak_reward(db, id.clone(), true, None, None).expect("reward");
    RewardsService::add_milestone(db, reward, 2, "Coffee".to_string()).unwrap();

    let goal = RewardsService::create_goal(
        db,
        "Certification".to_string(),
        None,
        "Dinner out".to_string(),
        Some("2026-12-01".to_string()),
    )
    .expect("goal");
    RewardsService::add_checkpoint(db, goal.clone(), "Study".to_string()).unwrap();
    RewardsService::complete_goal(db, goal).unwrap();

    SettingsService::set_language(db, "es").unwrap();
    id
}

#[test]
fn a_backup_restores_into_an_empty_database() {
    let source = TestDb::new();
    let habit_id = seed(&source);
    let path = backup_path("roundtrip");

    let exported = BackupService::export_to_file(&source, &path).expect("exported");
    assert_eq!(exported.habits, 1);
    assert_eq!(exported.logs, 2);
    assert_eq!(exported.rewards, 1);
    assert_eq!(exported.goals, 1);
    assert_eq!(exported.achievements, 1);

    let target = TestDb::new();
    let imported = BackupService::import_from_file(&target, &path).expect("imported");
    assert_eq!(imported.habits, exported.habits);

    let habit = target
        .get_habit(&habit_id)
        .unwrap()
        .expect("habit restored");
    assert_eq!(habit.name, "Walk");
    assert_eq!(habit.reminder_time.as_deref(), Some("08:30"));
    assert_eq!(habit.description.as_deref(), Some("Around the block"));
    assert_eq!(
        target
            .get_habit_logs("1970-01-01", "2099-12-31")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(RewardsService::get_goals(&target).unwrap().len(), 1);
    assert_eq!(RewardsService::get_achievements(&target).unwrap().len(), 1);
    assert_eq!(SettingsService::get_all(&target).unwrap().language, "es");

    let rewards = RewardsService::get_streak_rewards(&target).unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(
        RewardsService::get_milestones(&target, &rewards[0].id)
            .unwrap()
            .len(),
        1
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn archived_habits_survive_the_round_trip() {
    let source = TestDb::new();
    let id = seed(&source);
    HabitService::archive_habit(&source, id.clone()).unwrap();
    let path = backup_path("archived");
    BackupService::export_to_file(&source, &path).expect("exported");

    let target = TestDb::new();
    BackupService::import_from_file(&target, &path).expect("imported");

    assert!(HabitService::get_habits(&target).unwrap().is_empty());
    assert_eq!(HabitService::get_archived_habits(&target).unwrap().len(), 1);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn importing_replaces_whatever_was_there() {
    let source = TestDb::new();
    seed(&source);
    let path = backup_path("replace");
    BackupService::export_to_file(&source, &path).expect("exported");

    let target = TestDb::new();
    HabitService::create_habit(
        &target,
        "Something else".to_string(),
        None,
        "#ff0000".to_string(),
        "mind".to_string(),
        None,
    )
    .unwrap();

    BackupService::import_from_file(&target, &path).expect("imported");

    let habits = HabitService::get_habits(&target).unwrap();
    assert_eq!(habits.len(), 1);
    assert_eq!(habits[0].name, "Walk");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_file_from_another_app_is_refused() {
    let db = TestDb::new();
    let path = backup_path("foreign");
    std::fs::write(
        &path,
        r#"{"format":1,"app":"something-else","exported_at":"2026-01-01T00:00:00Z","data":{}}"#,
    )
    .unwrap();

    let err = BackupService::import_from_file(&db, &path).unwrap_err();
    assert_eq!(err.kind(), "invalid-backup");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_newer_format_is_refused_rather_than_guessed_at() {
    let db = TestDb::new();
    let path = backup_path("future");
    std::fs::write(
        &path,
        r#"{"format":99,"app":"derevo","exported_at":"2026-01-01T00:00:00Z","data":{}}"#,
    )
    .unwrap();

    let err = BackupService::import_from_file(&db, &path).unwrap_err();
    assert_eq!(err.kind(), "unsupported-backup");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_file_that_is_not_json_is_refused() {
    let db = TestDb::new();
    let path = backup_path("garbage");
    std::fs::write(&path, "this is not a backup").unwrap();

    let err = BackupService::import_from_file(&db, &path).unwrap_err();
    assert_eq!(err.kind(), "invalid-backup");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_missing_file_reports_a_file_error() {
    let db = TestDb::new();
    let err = BackupService::import_from_file(&db, &backup_path("absent")).unwrap_err();
    assert_eq!(err.kind(), "io");
}

#[test]
fn a_log_pointing_at_no_habit_leaves_the_database_untouched() {
    let db = TestDb::new();
    HabitService::create_habit(
        &db,
        "Keep me".to_string(),
        None,
        "#8b5cf6".to_string(),
        "body".to_string(),
        None,
    )
    .unwrap();

    let path = backup_path("dangling");
    std::fs::write(
        &path,
        r#"{"format":1,"app":"derevo","exported_at":"2026-01-01T00:00:00Z","data":{
             "habit_logs":[{"id":"l1","habit_id":"ghost","completed_date":"2026-01-01"}]
           }}"#,
    )
    .unwrap();

    let err = BackupService::import_from_file(&db, &path).unwrap_err();
    assert_eq!(err.kind(), "invalid-backup");

    let habits = HabitService::get_habits(&db).unwrap();
    assert_eq!(habits.len(), 1, "the rollback should have kept the habit");
    assert_eq!(habits[0].name, "Keep me");

    let _ = std::fs::remove_file(&path);
}
