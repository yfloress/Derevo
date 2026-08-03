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

use crate::db::Database;
use crate::error::{AppError, DbError};
use crate::models::{Achievement, Checkpoint, Goal, Milestone, StreakReward};
use uuid::Uuid;

pub struct RewardsService;

impl RewardsService {
    // ── Streak Rewards ──

    pub fn create_streak_reward(
        db: &Database,
        habit_id: String,
        is_consecutive: bool,
        target: i32,
        window_days: i32,
    ) -> Result<String, AppError> {
        let (target_days, target_total) = reward_targets(is_consecutive, target, window_days)?;
        let id = Uuid::new_v4().to_string();
        let reward = StreakReward::new(
            id.clone(),
            habit_id,
            is_consecutive,
            target_days,
            target_total,
        );
        db.create_streak_reward(&reward)?;
        Ok(id)
    }

    pub fn get_streak_rewards(db: &Database) -> Result<Vec<StreakReward>, DbError> {
        db.get_streak_rewards()
    }

    pub fn get_streak_rewards_by_habit(
        db: &Database,
        habit_id: &str,
    ) -> Result<Vec<StreakReward>, DbError> {
        db.get_streak_rewards_by_habit(habit_id)
    }

    pub fn delete_streak_reward(db: &Database, id: String) -> Result<(), DbError> {
        db.delete_streak_reward(&id)
    }

    pub fn update_streak_reward_with_milestones(
        db: &Database,
        id: String,
        habit_id: String,
        is_consecutive: bool,
        target: i32,
        window_days: i32,
        milestones: Vec<(i32, String)>,
    ) -> Result<(), AppError> {
        let (target_days, target_total) = reward_targets(is_consecutive, target, window_days)?;
        db.with_transaction(|conn| {
            let existing = Database::get_milestones_on(conn, &id)?;
            let reward = StreakReward::new(
                id.clone(),
                habit_id,
                is_consecutive,
                target_days,
                target_total,
            );
            Database::update_streak_reward_on(conn, &reward)?;
            Database::delete_milestones_by_reward_on(conn, &id)?;
            let progress = Database::get_streak_progress_on(
                conn,
                &reward.habit_id,
                reward.is_consecutive,
                reward.target_total,
            )?;

            for (days, text) in milestones {
                let milestone_id = Uuid::new_v4().to_string();
                let mut milestone = Milestone::new(milestone_id, id.clone(), days, text);

                let was_previously_unlocked =
                    existing.iter().any(|m| m.target_days == days && m.unlocked);
                let should_be_unlocked = was_previously_unlocked || progress >= days;

                if should_be_unlocked {
                    if let Some(existing_m) = existing
                        .iter()
                        .find(|m| m.target_days == days && m.unlocked)
                    {
                        milestone.unlocked = true;
                        milestone.unlocked_at = existing_m.unlocked_at.clone();
                    } else {
                        milestone.unlock();
                    }
                }
                Database::create_milestone_on(conn, &milestone)?;
            }
            Ok(())
        })
        .map_err(AppError::Database)
    }

    pub fn get_streak_progress(db: &Database, reward: &StreakReward) -> Result<i32, DbError> {
        db.get_streak_progress(&reward.habit_id, reward.is_consecutive, reward.target_total)
    }

    // ── Milestones ──

    pub fn add_milestone(
        db: &Database,
        reward_id: String,
        target_days: i32,
        reward_text: String,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let milestone = Milestone::new(id.clone(), reward_id, target_days, reward_text);
        db.create_milestone(&milestone)?;
        Ok(id)
    }

    pub fn get_milestones(db: &Database, reward_id: &str) -> Result<Vec<Milestone>, DbError> {
        db.get_milestones(reward_id)
    }

    pub fn check_and_unlock_milestones(
        db: &Database,
        reward_id: &str,
    ) -> Result<Vec<String>, DbError> {
        let reward = match db.get_streak_reward(reward_id)? {
            Some(r) => r,
            None => return Ok(vec![]),
        };
        let progress =
            db.get_streak_progress(&reward.habit_id, reward.is_consecutive, reward.target_total)?;
        let milestones = db.get_milestones(reward_id)?;
        let mut unlocked_ids = vec![];
        for mut milestone in milestones {
            if !milestone.unlocked && progress >= milestone.target_days {
                milestone.unlock();
                db.update_milestone(&milestone)?;
                unlocked_ids.push(milestone.id);
            }
        }
        Ok(unlocked_ids)
    }

    // ── Goals ──

    pub fn create_goal(
        db: &Database,
        name: String,
        description: Option<String>,
        reward_text: String,
        deadline: Option<String>,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let goal = Goal::new(id.clone(), name, description, reward_text, deadline);
        db.create_goal(&goal)?;
        Ok(id)
    }

    pub fn get_goals(db: &Database) -> Result<Vec<Goal>, DbError> {
        db.get_goals()
    }

    pub fn delete_goal(db: &Database, id: String) -> Result<(), DbError> {
        db.delete_goal(&id)
    }

    pub fn archive_goal(db: &Database, id: String) -> Result<(), DbError> {
        db.archive_goal(&id)
    }

    pub fn update_goal(
        db: &Database,
        id: String,
        name: String,
        description: String,
        reward_text: String,
        deadline: String,
    ) -> Result<(), DbError> {
        let mut goal = match db.get_goal(&id)? {
            Some(g) => g,
            None => return Err(DbError::GoalNotFound),
        };
        goal.name = name;
        goal.description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        goal.reward_text = reward_text;
        goal.deadline = if deadline.is_empty() {
            None
        } else {
            Some(deadline)
        };
        db.update_goal(&goal)
    }

    pub fn complete_goal(db: &Database, id: String) -> Result<Option<String>, DbError> {
        let mut goal = match db.get_goal(&id)? {
            Some(g) => g,
            None => return Ok(None),
        };
        if goal.is_completed {
            return Ok(None);
        }
        goal.complete();
        db.update_goal(&goal)?;
        let ach_id = create_achievement_internal(
            db,
            goal.name.clone(),
            format!("Completed: {}", goal.reward_text),
            "trophy.svg".into(),
            "goal".into(),
            id,
        )?;
        Ok(Some(ach_id))
    }

    pub fn update_goal_with_checkpoints(
        db: &Database,
        goal_id: String,
        name: String,
        description: String,
        reward_text: String,
        deadline: String,
        checkpoints: Vec<(Option<String>, String, i32)>,
    ) -> Result<(), DbError> {
        db.with_transaction(|conn| {
            let mut goal = match Database::get_goal_on(conn, &goal_id)? {
                Some(g) => g,
                None => return Err(DbError::GoalNotFound),
            };
            goal.name = name;
            goal.description = if description.is_empty() {
                None
            } else {
                Some(description)
            };
            goal.reward_text = reward_text;
            goal.deadline = if deadline.is_empty() {
                None
            } else {
                Some(deadline)
            };
            Database::update_goal_on(conn, &goal)?;

            let existing = Database::get_checkpoints_on(conn, &goal_id)?;
            let keep_ids: Vec<String> = checkpoints
                .iter()
                .filter_map(|(id, _, _)| id.clone())
                .collect();
            for cp in existing {
                if !keep_ids.iter().any(|id| id == &cp.id) {
                    Database::delete_checkpoint_on(conn, &cp.id)?;
                }
            }

            for (cp_id, text, order) in checkpoints {
                if let Some(checkpoint_id) = cp_id {
                    match Database::get_checkpoint_on(conn, &checkpoint_id)? {
                        Some(mut c) => {
                            c.description = text;
                            c.sort_order = order;
                            Database::update_checkpoint_on(conn, &c)?;
                        }
                        None => {
                            let nid = Uuid::new_v4().to_string();
                            Database::create_checkpoint_on(
                                conn,
                                &Checkpoint::new(nid, goal_id.clone(), text, order),
                            )?;
                        }
                    }
                } else {
                    let nid = Uuid::new_v4().to_string();
                    Database::create_checkpoint_on(
                        conn,
                        &Checkpoint::new(nid, goal_id.clone(), text, order),
                    )?;
                }
            }
            Ok(())
        })
    }

    // ── Checkpoints ──

    pub fn add_checkpoint(
        db: &Database,
        goal_id: String,
        description: String,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let order = db.get_next_checkpoint_order(&goal_id)?;
        db.create_checkpoint(&Checkpoint::new(id.clone(), goal_id, description, order))?;
        Ok(id)
    }

    pub fn get_checkpoints(db: &Database, goal_id: &str) -> Result<Vec<Checkpoint>, DbError> {
        db.get_checkpoints(goal_id)
    }

    pub fn toggle_checkpoint(
        db: &Database,
        goal_id: String,
        checkpoint_id: String,
    ) -> Result<bool, DbError> {
        let mut checkpoint = match db.get_checkpoint(&checkpoint_id)? {
            Some(c) => c,
            None => return Ok(false),
        };
        let is_completed = checkpoint.toggle();
        db.update_checkpoint(&checkpoint)?;
        let (total, completed) = db.get_checkpoint_counts(&goal_id)?;
        if total > 0 && total == completed {
            let goal_opt = db.get_goal(&goal_id)?;
            if let Some(mut goal) = goal_opt
                && !goal.is_completed
            {
                goal.complete();
                db.update_goal(&goal)?;
                let _ = create_achievement_internal(
                    db,
                    goal.name.clone(),
                    format!("Completed: {}", goal.reward_text),
                    "trophy.svg".into(),
                    "goal".into(),
                    goal_id,
                );
            }
        }
        Ok(is_completed)
    }

    // ── Achievements ──

    pub fn get_achievements(db: &Database) -> Result<Vec<Achievement>, DbError> {
        db.get_achievements()
    }
}

/// Splits the two numbers a reward carries.
///
/// `target_days` is what the progress bar counts up to and is meaningful in both
/// modes — it used to be dropped for consecutive rewards, which left the bar
/// dividing by nothing and sitting at 100% from day one.
///
/// `target_total` is only the look-back window of the accumulative mode, in
/// days; None there means the default of 30.
fn reward_targets(
    is_consecutive: bool,
    target: i32,
    window_days: i32,
) -> Result<(Option<i32>, Option<i32>), AppError> {
    if target < 1 {
        return Err(AppError::Validation(
            "The reward target must be at least 1".into(),
        ));
    }
    if !is_consecutive && window_days < 0 {
        return Err(AppError::Validation("The window cannot be negative".into()));
    }
    let window = if is_consecutive || window_days == 0 {
        None
    } else {
        Some(window_days)
    };
    Ok((Some(target), window))
}

fn create_achievement_internal(
    db: &Database,
    title: String,
    description: String,
    icon_path: String,
    achievement_type: String,
    source_id: String,
) -> Result<String, DbError> {
    if db.achievement_exists(&source_id, &achievement_type)? {
        return Ok(String::new());
    }
    let id = Uuid::new_v4().to_string();
    db.create_achievement(&Achievement::new(
        id.clone(),
        title,
        description,
        icon_path,
        achievement_type,
        source_id,
    ))?;
    Ok(id)
}
