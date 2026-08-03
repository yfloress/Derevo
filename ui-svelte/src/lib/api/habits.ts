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
import { invoke } from '@tauri-apps/api/core'
import type {
  HabitsResponse, HabitSummary, ArchivedHabitDto, ScheduleInput,
  HeatmapResponse, HabitAnalyticsResponse,
  StreakRewardDto, GoalDto, AchievementDto
} from '../types/habits'

export async function fetchHabits(month: number, year: number): Promise<HabitsResponse> {
  return invoke<HabitsResponse>('fetch_habits', { month, year })
}

export async function fetchArchivedHabits(): Promise<ArchivedHabitDto[]> {
  return invoke<ArchivedHabitDto[]>('fetch_archived_habits')
}

export async function fetchCategories(): Promise<string[]> {
  return invoke<string[]>('fetch_categories')
}

export interface HabitFormInput {
  name: string
  description: string | null
  color: string
  category: string
  reminder_time: string | null
  schedule: ScheduleInput
}

/** The Rust side reads the form as one object, so it cannot arrive half-set. */
function habitArg(form: HabitFormInput) {
  return {
    name: form.name,
    description: form.description,
    color: form.color,
    category: form.category,
    reminderTime: form.reminder_time,
    schedule: {
      kind: form.schedule.kind,
      days: form.schedule.days,
      targetPerPeriod: form.schedule.target_per_period,
    },
  }
}

export async function createHabit(form: HabitFormInput): Promise<void> {
  return invoke('create_habit', { habit: habitArg(form) })
}

export async function updateHabit(id: string, form: HabitFormInput): Promise<void> {
  return invoke('update_habit', { id, habit: habitArg(form) })
}

/** Reversible: keeps every log. `deleteHabit` is the one that does not. */
export async function archiveHabit(id: string): Promise<void> {
  return invoke('archive_habit', { id })
}

export async function restoreHabit(id: string): Promise<void> {
  return invoke('restore_habit', { id })
}

export async function deleteHabit(id: string): Promise<void> {
  return invoke('delete_habit', { id })
}

export async function toggleHabit(habit_id: string, date: string): Promise<void> {
  return invoke('toggle_habit', { habitId: habit_id, date })
}

export async function fetchHabitSummary(habit_id: string): Promise<HabitSummary> {
  return invoke<HabitSummary>('fetch_habit_summary', { habitId: habit_id })
}

export async function fetchHeatmap(year: number): Promise<HeatmapResponse> {
  return invoke<HeatmapResponse>('fetch_heatmap', { year })
}

export async function fetchHabitAnalytics(days?: number): Promise<HabitAnalyticsResponse> {
  return invoke<HabitAnalyticsResponse>('fetch_habit_analytics', { days: days ?? null })
}

export async function fetchRewards(): Promise<StreakRewardDto[]> {
  return invoke<StreakRewardDto[]>('fetch_rewards')
}

export async function createStreakReward(
  habit_id: string, is_consecutive: boolean, target_days: number, target_total: number
): Promise<void> {
  return invoke('create_streak_reward', { habitId: habit_id, isConsecutive: is_consecutive, targetDays: target_days, targetTotal: target_total })
}

export async function updateStreakReward(
  id: string, habit_id: string, is_consecutive: boolean,
  target_days: number, target_total: number,
  milestones: [number, string][]
): Promise<void> {
  return invoke('update_streak_reward', { id, habitId: habit_id, isConsecutive: is_consecutive, targetDays: target_days, targetTotal: target_total, milestones })
}

export async function deleteStreakReward(id: string): Promise<void> {
  return invoke('delete_streak_reward', { id })
}

export async function fetchGoals(): Promise<GoalDto[]> {
  return invoke<GoalDto[]>('fetch_goals')
}

export async function createGoal(
  name: string, description: string, reward_text: string, deadline: string
): Promise<void> {
  return invoke('create_goal', { name, description, rewardText: reward_text, deadline })
}

export async function updateGoal(
  id: string, name: string, description: string, reward_text: string, deadline: string
): Promise<void> {
  return invoke('update_goal', { id, name, description, rewardText: reward_text, deadline })
}

export async function deleteGoal(id: string): Promise<void> {
  return invoke('delete_goal', { id })
}

export async function completeGoal(id: string): Promise<void> {
  return invoke('complete_goal', { id })
}

export async function archiveGoal(id: string): Promise<void> {
  return invoke('archive_goal', { id })
}

export async function toggleCheckpoint(goal_id: string, checkpoint_id: string): Promise<void> {
  return invoke('toggle_checkpoint', { goalId: goal_id, checkpointId: checkpoint_id })
}

export interface GoalCheckpointInput {
  id: string
  text: string
}

export async function updateGoalWithCheckpoints(
  id: string, name: string, description: string, reward_text: string,
  deadline: string, checkpoints: GoalCheckpointInput[]
): Promise<void> {
  return invoke('update_goal_with_checkpoints', {
    id, name, description, rewardText: reward_text, deadline, checkpoints,
  })
}

export async function fetchAchievements(): Promise<AchievementDto[]> {
  return invoke<AchievementDto[]>('fetch_achievements')
}
