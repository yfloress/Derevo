<!-- Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
     Copyright (C) 2026  yfloress

     This program is free software: you can redistribute it and/or modify
     it under the terms of the GNU Affero General Public License as
     published by the Free Software Foundation, either version 3 of the
     License, or (at your option) any later version.

     This program is distributed in the hope that it will be useful,
     but WITHOUT ANY WARRANTY; without even the implied warranty of
     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
     GNU Affero General Public License for more details.

     You should have received a copy of the GNU Affero General Public License
     along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>. -->



<script lang="ts">
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import * as habitsApi from '../../lib/api/habits'
  import { errorMessage } from '../../lib/api/errors'
  import type { HabitDto, ScheduleKind } from '../../lib/types/habits'

  const colors = ['#34d399', '#22c55e', '#4ade80', '#fbbf24', '#fb923c', '#f87171', '#f472b6', '#22d3ee']

  interface Props {
    show: boolean
    editing: HabitDto | null
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), editing, onsubmit, onclose }: Props = $props()

  let habitName = $state('')
  let habitDescription = $state('')
  let habitColor = $state('#34d399')
  let habitCategory = $state('')
  // Not <input type="time">: that control follows the system locale, so its
  // am/pm segment can read as placeholder text. Leaving it unset yields an empty
  // value and the reminder disappears without a word. Here the switch says
  // whether there is a reminder at all, and the box always holds a real time.
  const DEFAULT_REMINDER = '08:00'
  let habitReminderOn = $state(false)
  let habitReminderText = $state(toDisplay(DEFAULT_REMINDER))

  /** "07:05" → "7:05 am". Twelve-hour form, which is what the box shows. */
  function toDisplay(time: string): string {
    const [rawHour, rawMinute] = time.split(':')
    const hour = Number(rawHour)
    const suffix = hour < 12 ? 'am' : 'pm'
    const shown = hour % 12 === 0 ? 12 : hour % 12
    return `${shown}:${rawMinute} ${suffix}`
  }

  /**
   * Accepts what people actually type: "8:30 pm", "8.30pm", "20:30", "8 pm".
   * Returns "HH:MM" for the backend, or null when it cannot be read — the form
   * then refuses to save rather than dropping the reminder.
   */
  function toStored(text: string): string | null {
    const match = text.trim().toLowerCase().match(/^(\d{1,2})(?:[:.h]?(\d{2}))?\s*(a\.?m\.?|p\.?m\.?)?$/)
    if (!match) return null
    let hour = Number(match[1])
    const minute = Number(match[2] ?? '0')
    const suffix = match[3]?.replace(/\./g, '')
    if (minute > 59) return null
    if (suffix) {
      if (hour < 1 || hour > 12) return null
      if (suffix === 'am') hour = hour === 12 ? 0 : hour
      else hour = hour === 12 ? 12 : hour + 12
    } else if (hour > 23) {
      return null
    }
    return `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}`
  }

  // Half-hour steps for the dropdown; anything else can still be typed.
  const REMINDER_CHOICES = Array.from({ length: 48 }, (_, i) =>
    toDisplay(`${String(Math.floor(i / 2)).padStart(2, '0')}:${i % 2 === 0 ? '00' : '30'}`),
  )
  // Offered as autocomplete so a new habit lands in an existing category
  // instead of creating a near-duplicate of it.
  let categories = $state<string[]>([])

  let scheduleKind = $state<ScheduleKind>('daily')
  let scheduleDays = $state<number[]>([])
  let timesPerWeek = $state(3)
  // Sunday first, matching the tracking grid.
  const WEEKDAY_KEYS = [
    'weekday-long-sun', 'weekday-long-mon', 'weekday-long-tue', 'weekday-long-wed',
    'weekday-long-thu', 'weekday-long-fri', 'weekday-long-sat',
  ]
  const WEEKDAY_INITIALS = [
    'weekday-sun', 'weekday-mon', 'weekday-tue', 'weekday-wed',
    'weekday-thu', 'weekday-fri', 'weekday-sat',
  ]

  function toggleDay(day: number) {
    scheduleDays = scheduleDays.includes(day)
      ? scheduleDays.filter((d) => d !== day)
      : [...scheduleDays, day].sort((a, b) => a - b)
  }

  const scheduleReady = $derived(
    scheduleKind !== 'weekdays' || scheduleDays.length > 0,
  )

  $effect(() => {
    show
    editing
    if (show) {
      if (editing) {
        habitName = editing.name
        habitDescription = editing.description ?? ''
        habitColor = editing.color
        habitCategory = editing.category
        habitReminderOn = !!editing.reminder_time
        habitReminderText = toDisplay(editing.reminder_time ?? DEFAULT_REMINDER)
        scheduleKind = editing.schedule_kind
        scheduleDays = [...editing.schedule_days]
        timesPerWeek = editing.target_per_period ?? 3
      } else {
        habitName = ''
        habitDescription = ''
        habitColor = '#34d399'
        habitCategory = ''
        habitReminderOn = false
        habitReminderText = toDisplay(DEFAULT_REMINDER)
        scheduleKind = 'daily'
        scheduleDays = []
        timesPerWeek = 3
      }
      habitsApi.fetchCategories().then((c) => { categories = c }).catch(() => { categories = [] })
    }
  })

  async function submitHabit() {
    let reminder: string | null = null
    if (habitReminderOn) {
      reminder = toStored(habitReminderText)
      if (reminder === null) {
        app.showToast(i18n.t('habits-reminder-invalid'), true)
        return
      }
    }
    const form = {
      name: habitName,
      description: habitDescription || null,
      color: habitColor,
      category: habitCategory,
      reminder_time: reminder,
      schedule: {
        kind: scheduleKind,
        days: scheduleKind === 'weekdays' ? scheduleDays : [],
        target_per_period: scheduleKind === 'times_per_week' ? timesPerWeek : null,
      },
    }
    try {
      if (editing) {
        await habitsApi.updateHabit(editing.id, form)
      } else {
        await habitsApi.createHabit(form)
      }
      show = false
      await onsubmit()
      app.showToast(editing ? i18n.t('habits-toast-habit-updated', 'Habit updated') : i18n.t('habits-toast-habit-created', 'Habit created'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{editing ? i18n.t('habits-edit-habit', 'Edit Habit') : i18n.t('habits-new-habit-modal', 'New Habit')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('habits-name', 'Name')}
        <input type="text" bind:value={habitName} placeholder={i18n.t('habits-habit-name-placeholder', 'Habit name')} />
      </label>
      <label>
        {i18n.t('habits-description', 'Description')}
        <input type="text" bind:value={habitDescription} placeholder={i18n.t('habits-desc-placeholder', 'Optional description')} />
      </label>
      <label>
        {i18n.t('habits-color', 'Color')}
        <div class="color-palette">
          {#each colors as c}
            <button
              class="color-swatch"
              class:selected={habitColor === c}
              style="background: {c}"
              aria-label="Color {c}"
              onclick={() => habitColor = c}
            ></button>
          {/each}
        </div>
      </label>
      <label>
        {i18n.t('habits-category', 'Category')}
        <input
          type="text"
          list="habit-categories"
          bind:value={habitCategory}
          placeholder={i18n.t('habits-category-placeholder', 'e.g. health, learning')}
        />
        <datalist id="habit-categories">
          {#each categories as category}
            <option value={category}></option>
          {/each}
        </datalist>
      </label>
      <div class="mode-field">
        <span>{i18n.t('habits-schedule')}</span>
        <label class="mode-option" class:selected={scheduleKind === 'daily'}>
          <input type="radio" value="daily" bind:group={scheduleKind} />
          <span class="mode-text">
            {i18n.t('habits-schedule-daily')}
            <span class="mode-hint">{i18n.t('habits-schedule-daily-hint')}</span>
          </span>
        </label>
        <label class="mode-option" class:selected={scheduleKind === 'weekdays'}>
          <input type="radio" value="weekdays" bind:group={scheduleKind} />
          <span class="mode-text">
            {i18n.t('habits-schedule-weekdays')}
            <span class="mode-hint">{i18n.t('habits-schedule-weekdays-hint')}</span>
          </span>
        </label>
        {#if scheduleKind === 'weekdays'}
          <div class="day-picker">
            {#each WEEKDAY_INITIALS as key, day}
              <button
                type="button"
                class="day-toggle"
                class:on={scheduleDays.includes(day)}
                aria-pressed={scheduleDays.includes(day)}
                aria-label={i18n.t(WEEKDAY_KEYS[day])}
                onclick={() => toggleDay(day)}
              >{i18n.t(key)}</button>
            {/each}
          </div>
        {/if}
        <label class="mode-option" class:selected={scheduleKind === 'times_per_week'}>
          <input type="radio" value="times_per_week" bind:group={scheduleKind} />
          <span class="mode-text">
            {i18n.t('habits-schedule-weekly')}
            <span class="mode-hint">{i18n.t('habits-schedule-weekly-hint')}</span>
          </span>
        </label>
        {#if scheduleKind === 'times_per_week'}
          <label class="weekly-target">
            {i18n.t('habits-schedule-weekly-target')}
            <input type="number" min="1" max="7" bind:value={timesPerWeek} />
          </label>
        {/if}
      </div>

      <div class="reminder-field">
        <label class="reminder-toggle">
          <input type="checkbox" bind:checked={habitReminderOn} />
          <span>{i18n.t('habits-reminder', 'Reminder')}</span>
        </label>
        {#if habitReminderOn}
          <!-- A combobox: pick a half hour from the list, or type any other. -->
          <input
            type="text"
            list="reminder-times"
            bind:value={habitReminderText}
            aria-label={i18n.t('habits-reminder', 'Reminder')}
          />
          <datalist id="reminder-times">
            {#each REMINDER_CHOICES as choice}
              <option value={choice}></option>
            {/each}
          </datalist>
          <span class="field-hint">{i18n.t('habits-reminder-hint')}</span>
        {/if}
      </div>
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('habits-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitHabit} disabled={!habitName.trim() || !scheduleReady}>
          {editing ? i18n.t('habits-update', 'Update') : i18n.t('habits-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .field-hint { display: block; margin-top: 4px; font-size: 0.72rem; color: var(--text-tertiary); }
  /* Mirrors .form-grid label so the row lines up with the fields above it. */
  .reminder-field { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .reminder-toggle { display: flex; flex-direction: row; align-items: center; gap: 8px; cursor: pointer; }
  .reminder-toggle input { accent-color: var(--accent); width: 15px; height: 15px; cursor: pointer; }

  .day-picker { display: flex; gap: 4px; margin: 2px 0 2px 4px; }
  .day-toggle {
    flex: 1; padding: 6px 0; border-radius: var(--radius-sm);
    border: 1px solid var(--glass-border); background: var(--glass);
    color: var(--text-secondary); font-size: 0.75rem; font-weight: 600;
    cursor: pointer; transition: all 0.15s;
  }
  .day-toggle:hover { background: var(--glass-hover); color: var(--text-primary); }
  .day-toggle.on {
    background: linear-gradient(135deg, var(--accent) 0%, var(--accent-hover) 100%);
    border-color: transparent; color: var(--text-on-accent);
  }
  /* Beats the column layout .form-grid label gives every label. */
  .mode-field .weekly-target { flex-direction: row; align-items: center; gap: 10px; margin-left: 4px; }
  .mode-field .weekly-target input { width: 70px; }
  .color-palette { display: flex; gap: 6px; margin-top: 4px; }
  .color-swatch {
    width: 28px; height: 28px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; transition: border-color 0.15s, transform 0.15s;
  }
  .color-swatch:hover { transform: scale(1.2); }
  .color-swatch.selected { border-color: var(--text-primary); transform: scale(1.15); }
</style>
