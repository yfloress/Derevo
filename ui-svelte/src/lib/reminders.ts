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
import type { HabitDto } from './types/habits'

const CHECK_INTERVAL_MS = 30_000

/**
 * How long after its time a reminder is still worth sending. Without this,
 * opening the app in the evening would fire every reminder of the day at once.
 */
const WINDOW_MINUTES = 120

/** Fired reminders, as `habitId:YYYY-MM-DD`, so each one arrives once a day. */
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

async function check(habits: HabitDto[], today: number) {
  const now = new Date()
  const nowMinutes = now.getHours() * 60 + now.getMinutes()
  const day = todayKey(now)

  for (const habit of habits) {
    if (!habit.reminder_time) continue
    if (habit.days[today]) continue

    const due = minutesOfDay(habit.reminder_time)
    if (due === null) continue
    if (nowMinutes < due || nowMinutes > due + WINDOW_MINUTES) continue

    const key = `${habit.id}:${day}`
    if (sent.has(key)) continue
    sent.add(key)

    try {
      await sendNotification({
        title: habit.name,
        body: i18n.t('habits-reminder-body'),
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
