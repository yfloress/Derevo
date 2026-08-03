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
use derevo::svc::{HabitService, RewardsService};

fn habit(db: &TestDb, name: &str) -> String {
    HabitService::create_habit(db, habit_input(name, "body")).expect("habit created")
}

fn complete(db: &TestDb, habit_id: &str, days: &[i64]) {
    for day in days {
        HabitService::toggle_habit_completion(db, habit_id.to_string(), days_ago(*day)).unwrap();
    }
}

#[test]
fn a_consecutive_streak_counts_back_from_today() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    complete(&db, &id, &[0, 1, 2]);

    assert_eq!(db.get_streak_progress(&id, true, None).unwrap(), 3);
}

#[test]
fn today_being_unticked_does_not_break_the_streak() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    complete(&db, &id, &[1, 2, 3]);

    // The day is not over yet, so yesterday's streak still stands.
    assert_eq!(db.get_streak_progress(&id, true, None).unwrap(), 3);
}

#[test]
fn a_missed_day_ends_the_streak() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    complete(&db, &id, &[0, 1, 3, 4, 5]);

    assert_eq!(db.get_streak_progress(&id, true, None).unwrap(), 2);
}

#[test]
fn a_habit_with_no_logs_has_no_streak() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");

    assert_eq!(db.get_streak_progress(&id, true, None).unwrap(), 0);
}

#[test]
fn accumulative_progress_ignores_the_gaps() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    complete(&db, &id, &[0, 4, 9, 20]);

    assert_eq!(db.get_streak_progress(&id, false, Some(30)).unwrap(), 4);
}

#[test]
fn milestones_unlock_once_the_streak_reaches_them() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    let reward =
        RewardsService::create_streak_reward(&db, id.clone(), true, 30, 0).expect("reward");
    RewardsService::add_milestone(&db, reward.clone(), 2, "Coffee".to_string()).unwrap();
    RewardsService::add_milestone(&db, reward.clone(), 5, "Book".to_string()).unwrap();

    complete(&db, &id, &[0, 1, 2]);
    let unlocked = RewardsService::check_and_unlock_milestones(&db, &reward).unwrap();

    assert_eq!(unlocked.len(), 1, "only the 2-day milestone is reached");
    let milestones = RewardsService::get_milestones(&db, &reward).unwrap();
    let two_day = milestones.iter().find(|m| m.target_days == 2).unwrap();
    let five_day = milestones.iter().find(|m| m.target_days == 5).unwrap();
    assert!(two_day.unlocked);
    assert!(two_day.unlocked_at.is_some());
    assert!(!five_day.unlocked);
}

#[test]
fn an_unlocked_milestone_is_not_unlocked_twice() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    let reward =
        RewardsService::create_streak_reward(&db, id.clone(), true, 30, 0).expect("reward");
    RewardsService::add_milestone(&db, reward.clone(), 1, "Coffee".to_string()).unwrap();
    complete(&db, &id, &[0]);

    assert_eq!(
        RewardsService::check_and_unlock_milestones(&db, &reward)
            .unwrap()
            .len(),
        1
    );
    assert!(
        RewardsService::check_and_unlock_milestones(&db, &reward)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn editing_a_reward_keeps_the_milestones_that_were_already_unlocked() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    let reward =
        RewardsService::create_streak_reward(&db, id.clone(), true, 30, 0).expect("reward");
    RewardsService::add_milestone(&db, reward.clone(), 2, "Coffee".to_string()).unwrap();
    complete(&db, &id, &[0, 1]);
    RewardsService::check_and_unlock_milestones(&db, &reward).unwrap();
    let unlocked_at = RewardsService::get_milestones(&db, &reward).unwrap()[0]
        .unlocked_at
        .clone();

    RewardsService::update_streak_reward_with_milestones(
        &db,
        reward.clone(),
        id,
        true,
        30,
        0,
        vec![(2, "Better coffee".to_string())],
    )
    .expect("reward updated");

    let milestones = RewardsService::get_milestones(&db, &reward).unwrap();
    assert_eq!(milestones.len(), 1);
    assert!(milestones[0].unlocked);
    assert_eq!(milestones[0].reward_text, "Better coffee");
    assert_eq!(
        milestones[0].unlocked_at, unlocked_at,
        "the original unlock date should be preserved"
    );
}

/// Consecutive rewards used to be stored with no target at all, which left the
/// progress bar dividing by nothing and showing "3 / ?".
#[test]
fn a_consecutive_reward_keeps_its_target() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    let reward = RewardsService::create_streak_reward(&db, id, true, 30, 0).expect("reward");

    let stored = db.get_streak_reward(&reward).unwrap().unwrap();
    assert_eq!(stored.target_days, Some(30));
    assert_eq!(stored.target_total, None, "the window is accumulative-only");
}

#[test]
fn an_accumulative_reward_keeps_both_the_target_and_the_window() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    let reward = RewardsService::create_streak_reward(&db, id, false, 12, 21).expect("reward");

    let stored = db.get_streak_reward(&reward).unwrap().unwrap();
    assert_eq!(stored.target_days, Some(12));
    assert_eq!(stored.target_total, Some(21));
}

#[test]
fn a_reward_target_below_one_is_refused() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    let err = RewardsService::create_streak_reward(&db, id, true, 0, 0).unwrap_err();
    assert_eq!(err.kind(), "validation");
}

#[test]
fn an_empty_window_falls_back_to_the_default() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    complete(&db, &id, &[0, 5, 40]);
    let reward = RewardsService::create_streak_reward(&db, id, false, 10, 0).expect("reward");

    let stored = db.get_streak_reward(&reward).unwrap().unwrap();
    // None means the 30-day default, so the 40-day-old entry stays out.
    assert_eq!(stored.target_total, None);
    assert_eq!(
        RewardsService::get_streak_progress(&db, &stored).unwrap(),
        2
    );
}

#[test]
fn a_goal_completes_when_its_last_checkpoint_is_ticked() {
    let db = TestDb::new();
    let goal = RewardsService::create_goal(
        &db,
        "Certification".to_string(),
        None,
        "Dinner out".to_string(),
        None,
    )
    .expect("goal");
    let first = RewardsService::add_checkpoint(&db, goal.clone(), "Study".to_string()).unwrap();
    let second = RewardsService::add_checkpoint(&db, goal.clone(), "Exam".to_string()).unwrap();

    RewardsService::toggle_checkpoint(&db, goal.clone(), first).unwrap();
    assert!(!db.get_goal(&goal).unwrap().unwrap().is_completed);

    RewardsService::toggle_checkpoint(&db, goal.clone(), second).unwrap();
    let completed = db.get_goal(&goal).unwrap().unwrap();
    assert!(completed.is_completed);
    assert!(completed.completed_at.is_some());
}

#[test]
fn completing_a_goal_creates_exactly_one_achievement() {
    let db = TestDb::new();
    let goal = RewardsService::create_goal(
        &db,
        "Certification".to_string(),
        None,
        "Dinner out".to_string(),
        None,
    )
    .expect("goal");

    RewardsService::complete_goal(&db, goal.clone()).unwrap();
    RewardsService::complete_goal(&db, goal).unwrap();

    let achievements = RewardsService::get_achievements(&db).unwrap();
    assert_eq!(achievements.len(), 1);
    assert_eq!(achievements[0].achievement_type, "goal");
}

#[test]
fn deleting_a_habit_takes_its_rewards_with_it() {
    let db = TestDb::new();
    let id = habit(&db, "Walk");
    RewardsService::create_streak_reward(&db, id.clone(), true, 30, 0).expect("reward");

    HabitService::delete_habit(&db, id).unwrap();

    assert!(RewardsService::get_streak_rewards(&db).unwrap().is_empty());
}
