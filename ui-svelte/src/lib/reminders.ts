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

import {
  isPermissionGranted, requestPermission, sendNotification,
} from '@tauri-apps/plugin-notification'
import { i18n } from './stores/i18n.svelte'
import { isDueOnWeekday } from './types/habits'
import type { HabitDto } from './types/habits'

const CHECK_INTERVAL_MS = 30_000

/**
 * How long after its time a reminder is still worth sending. Without this,
 * opening the app in the evening would fire every reminder of the day at once.
 */
const WINDOW_MINUTES = 120

/** Names listed before the message switches to "and N more". */
const NAMES_SHOWN = 3

/** How long a streak has to be before it is worth mentioning. */
const STREAK_WORTH_MENTIONING = 3

/** Sent reminders, as `slot:YYYY-MM-DD`, so each slot arrives once a day. */
const sent = new Set<string>()

export async function permissionGranted(): Promise<boolean> {
  try {
    return await isPermissionGranted()
  } catch {
    return false
  }
}

export async function askForPermission(): Promise<boolean> {
  try {
    return (await requestPermission()) === 'granted'
  } catch {
    return false
  }
}

function minutesOfDay(time: string): number | null {
  const [hours, minutes] = time.split(':').map(Number)
  if (!Number.isFinite(hours) || !Number.isFinite(minutes)) return null
  return hours * 60 + minutes
}

function todayKey(now: Date): string {
  return [
    now.getFullYear(),
    String(now.getMonth() + 1).padStart(2, '0'),
    String(now.getDate()).padStart(2, '0'),
  ].join('-')
}

/** Days ticked in a row up to and including today, within the loaded month. */
function trailingStreak(habit: HabitDto, today: number): number {
  let streak = 0
  for (let day = today; day >= 1; day--) {
    if (habit.days[day]) streak++
    else break
  }
  return streak
}

/**
 * The names, then the count of whatever did not fit: "Walk, Read and 2 more".
 * Listing eight habits in a notification is a wall nobody reads.
 */
function nameList(habits: HabitDto[]): string {
  const shown = habits.slice(0, NAMES_SHOWN).map((h) => h.name)
  const rest = habits.length - shown.length
  if (rest > 0) shown.push(i18n.t('reminder-and-more').replace('{n}', String(rest)))
  if (shown.length === 1) return shown[0]
  return `${shown.slice(0, -1).join(', ')} ${i18n.t('reminder-and')} ${shown[shown.length - 1]}`
}

/**
 * Picks the line that goes on top. A streak worth protecting is the strongest
 * nudge there is, so it wins; otherwise one of the rotating lines, chosen by the
 * date so the same message does not repeat twice in a day.
 */
function title(pending: HabitDto[], today: number, day: string): string {
  const longest = Math.max(...pending.map((h) => trailingStreak(h, today)), 0)
  if (longest >= STREAK_WORTH_MENTIONING) {
    return i18n.t('reminder-streak-title').replace('{n}', String(longest))
  }
  const variants = i18n.t('reminder-titles').split('|')
  const index = Number(day.replaceAll('-', '')) % variants.length
  return variants[index]
}

async function check(habits: HabitDto[], today: number) {
  const now = new Date()
  const nowMinutes = now.getHours() * 60 + now.getMinutes()
  const day = todayKey(now)
  const weekday = now.getDay()

  // One notification per reminder time, not per habit: five habits at 08:00
  // should be one message, not five.
  const slots = new Map<string, HabitDto[]>()
  for (const habit of habits) {
    if (!habit.reminder_time) continue
    if (habit.days[today]) continue
    if (!isDueOnWeekday(habit, weekday)) continue

    const due = minutesOfDay(habit.reminder_time)
    if (due === null) continue
    if (nowMinutes < due || nowMinutes > due + WINDOW_MINUTES) continue

    const slot = slots.get(habit.reminder_time) ?? []
    slot.push(habit)
    slots.set(habit.reminder_time, slot)
  }

  for (const [time, pending] of slots) {
    const key = `${time}:${day}`
    if (sent.has(key)) continue
    sent.add(key)

    try {
      await sendNotification({
        title: title(pending, today, day),
        body: `${i18n.t('reminder-pending').replace('{n}', String(pending.length))} ${nameList(pending)}`,
      })
    } catch {
      // Permission was revoked, or the platform refused. Retrying every 30
      // seconds would only produce noise, so the day's slot stays marked.
    }
  }
}

/**
 * Watches the given habits and notifies about the ones still pending. Only runs
 * while the app is open — Derevo schedules nothing with the operating system.
 * Returns a stop function.
 */
export function startReminders(
  habits: () => HabitDto[],
  today: () => number,
): () => void {
  let stopped = false

  const tick = async () => {
    if (stopped) return
    if (!(await permissionGranted())) return
    const currentDay = today()
    if (currentDay < 1) return
    await check(habits(), currentDay)
  }

  void tick()
  const timer = setInterval(() => void tick(), CHECK_INTERVAL_MS)

  return () => {
    stopped = true
    clearInterval(timer)
  }
}
