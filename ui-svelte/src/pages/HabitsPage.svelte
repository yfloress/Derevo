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
  // The webview's own window.confirm is not reliable inside Tauri; the plugin
  // opens a real native dialog.
  import { confirm as confirmDialog } from '@tauri-apps/plugin-dialog'
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as habitsApi from '../lib/api/habits'
  import { errorMessage } from '../lib/api/errors'
  import { startReminders } from '../lib/reminders'
  import MonthlyProgressChart from '../components/charts/MonthlyProgressChart.svelte'
  import SettingsModal from '../components/SettingsModal.svelte'
  import HabitFormModal from '../components/habits/HabitFormModal.svelte'
  import HabitHeatmap from '../components/habits/HabitHeatmap.svelte'
  import HabitRewardsPanel from '../components/habits/HabitRewardsPanel.svelte'
  import HabitGoalsPanel from '../components/habits/HabitGoalsPanel.svelte'
  import HabitMonthGrid from '../components/habits/HabitMonthGrid.svelte'
  import HabitWeekView from '../components/habits/HabitWeekView.svelte'
  import HabitSummaryCard from '../components/habits/HabitSummaryCard.svelte'
  import HabitAnalyticsPanel from '../components/habits/HabitAnalyticsPanel.svelte'
  import AchievementsTimeline from '../components/habits/AchievementsTimeline.svelte'
  import type {
    HabitDto, HabitsResponse, HabitSummary,
    HeatmapResponse, HabitAnalyticsResponse,
    StreakRewardDto, GoalDto, AchievementDto
  } from '../lib/types/habits'

  type Tab = 'habits' | 'rewards' | 'history'
  let activeTab = $state<Tab>('habits')
  let loading = $state(true)
  let monthLoading = $state(false)

  // Habits tab state
  let habitsData = $state<HabitsResponse | null>(null)
  let selectedHabit = $state<HabitDto | null>(null)
  let summary = $state<HabitSummary | null>(null)
  let heatmap = $state<HeatmapResponse | null>(null)
  let analytics = $state<HabitAnalyticsResponse | null>(null)
  let heatmapYear = $state(new Date().getFullYear())

  // Rewards tab state
  let rewards = $state<StreakRewardDto[]>([])
  let goals = $state<GoalDto[]>([])

  // History tab state
  let achievements = $state<AchievementDto[]>([])

  // Month navigation
  let month = $state(new Date().getMonth() + 1)
  let year = $state(new Date().getFullYear())

  // Responsive week view: phones show one week at a time (the full month grid is
  // too wide and horizontal scrolling is poor UX); desktop keeps the month grid.
  let isMobile = $state(typeof window !== 'undefined' && window.matchMedia('(max-width: 640px)').matches)
  let weekIndex = $state(0)
  // Set when the week pager crosses a month edge so the freshly loaded month
  // lands on its first/last week instead of today's.
  let pendingWeek: 'first' | 'last' | null = null

  // Modal state
  let showSettings = $state(false)
  let showAddHabit = $state(false)
  let showAddReward = $state(false)
  let showAddGoal = $state(false)
  let editingHabit = $state<HabitDto | null>(null)

  /**
   * `selectedHabit` holds an object from the load it was picked in. Every
   * refetch builds new ones, so without this the summary card — and the edit
   * form it opens — keep showing the values from before the change.
   */
  function syncSelectedHabit() {
    if (!selectedHabit) return
    const fresh = habitsData?.habits.find((h) => h.id === selectedHabit!.id) ?? null
    selectedHabit = fresh
    if (!fresh) summary = null
  }

  async function load() {
    loading = true
    try {
      habitsData = await habitsApi.fetchHabits(month, year)
      syncSelectedHabit()
      syncWeekIndex()
      heatmap = await habitsApi.fetchHeatmap(heatmapYear)
      analytics = await habitsApi.fetchHabitAnalytics()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      loading = false
      monthLoading = false
    }
  }

  async function loadRewards() {
    try {
      const [r, g] = await Promise.all([
        habitsApi.fetchRewards(),
        habitsApi.fetchGoals(),
      ])
      rewards = r
      goals = g
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function loadHistory() {
    try {
      achievements = await habitsApi.fetchAchievements()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function prevMonth() {
    if (month === 1) { month = 12; year-- }
    else { month-- }
    monthLoading = true
    load()
  }

  function nextMonth() {
    if (month === 12) { month = 1; year++ }
    else { month++ }
    monthLoading = true
    load()
  }

  // Mobile week pager. Moves within the month; at an edge it flips to the
  // adjacent month and lands on its last/first week.
  function prevWeek() {
    if (weekIndex > 0) weekIndex--
    else { pendingWeek = 'last'; prevMonth() }
  }

  function nextWeek() {
    if (weekIndex < weeks.length - 1) weekIndex++
    else { pendingWeek = 'first'; nextMonth() }
  }

  async function toggleDay(habitId: string, day: number) {
    const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    try {
      await habitsApi.toggleHabit(habitId, dateStr)
      const [newHabits, newHeatmap, newAnalytics] = await Promise.all([
        habitsApi.fetchHabits(month, year),
        habitsApi.fetchHeatmap(heatmapYear),
        habitsApi.fetchHabitAnalytics(),
      ])
      habitsData = newHabits
      heatmap = newHeatmap
      analytics = newAnalytics
      syncSelectedHabit()
      if (selectedHabit?.id === habitId) {
        summary = await habitsApi.fetchHabitSummary(habitId)
      }
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function selectHabit(habit: HabitDto) {
    if (selectedHabit?.id === habit.id) { selectedHabit = null; summary = null; return }
    selectedHabit = habit
    try {
      summary = await habitsApi.fetchHabitSummary(habit.id)
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function openAddHabit() {
    editingHabit = null
    showAddHabit = true
  }

  function openEditHabit(h: HabitDto) {
    editingHabit = h
    showAddHabit = true
  }

  /** Reversible. Permanent deletion lives in Settings, next to the archive. */
  async function archiveHabit(id: string) {
    if (!(await confirmDialog(i18n.t('habits-archive-confirm')))) return
    try {
      await habitsApi.archiveHabit(id)
      if (selectedHabit?.id === id) { selectedHabit = null; summary = null }
      await load()
      app.showToast(i18n.t('habits-toast-habit-archived'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  const monthNames = $derived([
    i18n.t('month-january','January'), i18n.t('month-february','February'), i18n.t('month-march','March'),
    i18n.t('month-april','April'), i18n.t('month-may','May'), i18n.t('month-june','June'),
    i18n.t('month-july','July'), i18n.t('month-august','August'), i18n.t('month-september','September'),
    i18n.t('month-october','October'), i18n.t('month-november','November'), i18n.t('month-december','December'),
  ])

  const WEEKDAYS = $derived([
    i18n.t('weekday-sun', 'S'), i18n.t('weekday-mon', 'M'), i18n.t('weekday-tue', 'T'),
    i18n.t('weekday-wed', 'W'), i18n.t('weekday-thu', 'T'), i18n.t('weekday-fri', 'F'),
    i18n.t('weekday-sat', 'S'),
  ])

  const now = new Date()
  const viewingCurrentMonth = $derived(now.getFullYear() === year && now.getMonth() + 1 === month)
  const todayDay = $derived(viewingCurrentMonth ? now.getDate() : -1)
  // For the progress chart: plot up to today this month, the full month otherwise.
  const progressLastDay = $derived(
    viewingCurrentMonth ? todayDay : (habitsData?.days_in_month ?? 0)
  )

  function weekdayOf(day: number): number {
    return new Date(year, month - 1, day).getDay()
  }

  // Group the month into Sunday-aligned calendar weeks, padding days that fall
  // outside the month with null. Powers the mobile week view.
  function buildWeeks(daysInMonth: number, firstWeekday: number): (number | null)[][] {
    const out: (number | null)[][] = []
    let cur: (number | null)[] = []
    for (let i = 0; i < firstWeekday; i++) cur.push(null)
    for (let d = 1; d <= daysInMonth; d++) {
      cur.push(d)
      if (cur.length === 7) { out.push(cur); cur = [] }
    }
    if (cur.length) { while (cur.length < 7) cur.push(null); out.push(cur) }
    return out
  }

  const weeks = $derived(habitsData ? buildWeeks(habitsData.days_in_month, weekdayOf(1)) : [])
  const currentWeek = $derived(weeks[Math.min(weekIndex, Math.max(weeks.length - 1, 0))] ?? [])
  const weekRangeLabel = $derived.by(() => {
    const days = currentWeek.filter((d): d is number => d !== null)
    if (days.length === 0) return ''
    return `${days[0]} – ${days[days.length - 1]} ${monthNames[month - 1].slice(0, 3)} ${year}`
  })

  // Pick a sensible week after a month load: the one crossed into via the pager,
  // else today's week (current month) or the first week.
  function syncWeekIndex() {
    if (!habitsData) { weekIndex = 0; pendingWeek = null; return }
    const wks = buildWeeks(habitsData.days_in_month, weekdayOf(1))
    if (pendingWeek === 'last') weekIndex = Math.max(wks.length - 1, 0)
    else if (pendingWeek === 'first') weekIndex = 0
    else {
      const wk = wks.findIndex((w) => w.includes(todayDay))
      weekIndex = viewingCurrentMonth && wk >= 0 ? wk : 0
    }
    pendingWeek = null
  }

  function trailingStreak(habit: HabitDto): number {
    const lastDay = viewingCurrentMonth ? todayDay : (habitsData?.days_in_month ?? 0)
    let s = 0
    for (let i = lastDay; i >= 1; i--) {
      if (habit.days[i]) s++
      else break
    }
    return s
  }

  $effect(() => { load() })
  $effect(() => { if (activeTab === 'rewards') loadRewards() })
  $effect(() => { if (activeTab === 'history') loadHistory() })

  // Reminders only make sense against the month on screen, which is the only
  // one whose day cells are loaded.
  $effect(() => startReminders(
    () => (viewingCurrentMonth ? habitsData?.habits ?? [] : []),
    () => todayDay,
  ))

  // Track viewport so we can swap between the desktop month grid and the mobile
  // week view.
  $effect(() => {
    const mq = window.matchMedia('(max-width: 640px)')
    const apply = () => { isMobile = mq.matches }
    apply()
    mq.addEventListener('change', apply)
    return () => mq.removeEventListener('change', apply)
  })
</script>

<div class="page" class:blurred={showAddHabit || showSettings}>
  <div class="page-header">
    <div class="brand">
      <img class="brand-logo" src="/logo.svg" alt="Derevo" />
      <h2>{i18n.t('habits-title', 'HABITS')}</h2>
    </div>
    {#if activeTab === 'habits' && !isMobile}
      <div class="month-nav">
        <button class="nav-arrow" aria-label="Previous month" onclick={prevMonth}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 19l-7-7 7-7"/></svg>
        </button>
        <span class="month-label">{monthNames[month - 1]} {year}</span>
        {#if monthLoading}
          <div class="mini-spinner"></div>
        {/if}
        <button class="nav-arrow" aria-label="Next month" onclick={nextMonth}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 5l7 7-7 7"/></svg>
        </button>
      </div>
    {/if}
    <button class="nav-arrow settings-btn" aria-label={i18n.t('settings-title', 'Settings')} onclick={() => showSettings = true}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
    </button>
  </div>

  <div class="tab-bar">
    <button class:active={activeTab === 'habits'} onclick={() => activeTab = 'habits'}>{i18n.t('habits-tab-habits', 'Habits')}</button>
    <button class:active={activeTab === 'rewards'} onclick={() => activeTab = 'rewards'}>{i18n.t('habits-tab-rewards', 'Rewards')}</button>
    <button class:active={activeTab === 'history'} onclick={() => activeTab = 'history'}>{i18n.t('habits-tab-history', 'History')}</button>
  </div>

  {#if loading}
    <div class="skeleton-page">
      <div class="skeleton" style="width:100%;height:110px;border-radius:var(--radius-lg);margin-bottom:16px"></div>
      <div class="skeleton" style="width:100%;height:80px;border-radius:var(--radius-lg);margin-bottom:16px"></div>
      <div class="skeleton-row">
        <div class="skeleton" style="flex:1;height:160px;border-radius:var(--radius-lg)"></div>
        <div class="skeleton" style="flex:1;height:160px;border-radius:var(--radius-lg)"></div>
      </div>
    </div>

  <!-- HABITS TAB -->
  {:else if activeTab === 'habits'}
    <div class="section-header">
      <h3>{i18n.t('habits-daily-tracking', 'Daily Tracking')}</h3>
      <button class="glass-btn" onclick={openAddHabit}>{i18n.t('habits-new-habit', 'New Habit')}</button>
    </div>

    {#if habitsData && habitsData.habits.length > 0}
      {#if isMobile}
        <HabitWeekView
          habits={habitsData.habits}
          week={currentWeek}
          weekLabel={weekRangeLabel}
          weekdays={WEEKDAYS}
          todayDay={todayDay}
          selectedId={selectedHabit?.id ?? null}
          streakOf={trailingStreak}
          onselect={selectHabit}
          ontoggle={toggleDay}
          onprev={prevWeek}
          onnext={nextWeek}
        />
      {:else}
        <HabitMonthGrid
          habits={habitsData.habits}
          daysInMonth={habitsData.days_in_month}
          todayDay={todayDay}
          weekdays={WEEKDAYS}
          selectedId={selectedHabit?.id ?? null}
          weekdayOf={weekdayOf}
          streakOf={trailingStreak}
          onselect={selectHabit}
          ontoggle={toggleDay}
          scrollKey={`${month}-${year}-${loading}`}
        />
      {/if}

      <!-- Selected habit metrics — inline, attached right under the grid -->
      {#if selectedHabit && summary}
        <HabitSummaryCard
          habit={selectedHabit}
          summary={summary}
          onedit={() => openEditHabit(selectedHabit!)}
          onarchive={() => archiveHabit(selectedHabit!.id)}
          onclose={() => { selectedHabit = null; summary = null }}
        />
      {/if}

      <!-- Monthly progress line -->
      {#if progressLastDay > 0}
        <div class="chart-card progress-card">
          <h3>{i18n.t('habits-monthly-progress', 'Monthly Progress')}</h3>
          <MonthlyProgressChart habits={habitsData.habits} lastDay={progressLastDay} />
        </div>
      {/if}

      <!-- Activity Heatmap -->
      {#if heatmap}
        <HabitHeatmap
          bind:heatmapYear={heatmapYear}
          heatmap={heatmap}
          onyearchange={async (year) => { heatmapYear = year; try { heatmap = await habitsApi.fetchHeatmap(heatmapYear) } catch (e) { app.showToast(errorMessage(e), true) } }}
        />
      {/if}

      {#if analytics}
        <HabitAnalyticsPanel analytics={analytics} />
      {/if}
    {:else}
      <p class="empty">{i18n.t('habits-no-habits', 'No habits yet. Create your first habit to start tracking.')}</p>
    {/if}

  <!-- REWARDS TAB -->
  {:else if activeTab === 'rewards'}
    <div class="rewards-section">
      <HabitRewardsPanel
        rewards={rewards}
        habits={habitsData?.habits ?? []}
        onrefresh={loadRewards}
        bind:showAddReward
      />

      <HabitGoalsPanel
        goals={goals}
        onrefresh={loadRewards}
        ongoalsupdate={(updated) => goals = updated}
        bind:showAddGoal
      />
    </div>

  <!-- HISTORY TAB -->
  {:else if activeTab === 'history'}
    <AchievementsTimeline achievements={achievements} />
  {/if}
</div>

<!-- Add/Edit Habit Modal -->
<HabitFormModal
  bind:show={showAddHabit}
  editing={editingHabit}
  onsubmit={load}
  onclose={() => showAddHabit = false}
/>

<!-- Settings Modal -->
<SettingsModal
  bind:show={showSettings}
  onclose={() => showSettings = false}
  ondatachange={load}
/>

<style>
  .page { padding: 24px 32px; max-width: 1000px; width: 100%; margin: 0 auto; }

  .page-header { display: flex; align-items: center; gap: 14px; margin-bottom: 16px; }
  .brand { display: flex; align-items: center; gap: 12px; margin-right: auto; }
  .brand-logo { height: 40px; width: auto; display: block; filter: drop-shadow(0 0 10px var(--accent-glow)); }
  .tab-bar { margin-bottom: 20px; }
  h2 { font-size: 1.5rem; letter-spacing: 0.05em; color: var(--text-primary); margin: 0; font-family: 'Unbounded', sans-serif; font-weight: 700; }

  .month-nav { display: flex; align-items: center; gap: 12px; }
  .nav-arrow { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .nav-arrow:hover { color: var(--text-primary); }
  .nav-arrow svg { width: 18px; height: 18px; }
  .month-label { font-size: 0.9rem; color: var(--text-secondary); min-width: 140px; text-align: center; }

  .skeleton-page { padding: 8px 0; }

  /* ── Mobile ───────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .page { padding: 20px 16px; }
    /* Reflow the header: brand + settings on top, month nav on its own
       centered row so the settings icon always has a stable home. */
    .page-header { flex-wrap: wrap; }
    .settings-btn { order: 0; }
    .month-nav { order: 1; width: 100%; justify-content: center; margin-top: 6px; }
  }
</style>
