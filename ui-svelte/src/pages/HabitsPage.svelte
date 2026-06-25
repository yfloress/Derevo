<!-- Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
     Copyright (C) 2026  Kyronix

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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as habitsApi from '../lib/api/habits'
  import RadarChart from '../components/charts/RadarChart.svelte'
  import WeekdayChart from '../components/charts/WeekdayChart.svelte'
  import MonthlyProgressChart from '../components/charts/MonthlyProgressChart.svelte'
  import SettingsModal from '../components/SettingsModal.svelte'
  import HabitFormModal from '../components/habits/HabitFormModal.svelte'
  import HabitHeatmap from '../components/habits/HabitHeatmap.svelte'
  import HabitRewardsPanel from '../components/habits/HabitRewardsPanel.svelte'
  import HabitGoalsPanel from '../components/habits/HabitGoalsPanel.svelte'
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

  async function load() {
    loading = true
    try {
      habitsData = await habitsApi.fetchHabits(month, year)
      syncWeekIndex()
      heatmap = await habitsApi.fetchHeatmap(heatmapYear)
      analytics = await habitsApi.fetchHabitAnalytics()
    } catch (e) {
      app.showToast(String(e), true)
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
      app.showToast(String(e), true)
    }
  }

  async function loadHistory() {
    try {
      achievements = await habitsApi.fetchAchievements()
    } catch (e) {
      app.showToast(String(e), true)
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
      if (selectedHabit?.id === habitId) {
        summary = await habitsApi.fetchHabitSummary(habitId)
      }
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function selectHabit(habit: HabitDto) {
    if (selectedHabit?.id === habit.id) { selectedHabit = null; summary = null; return }
    selectedHabit = habit
    try {
      summary = await habitsApi.fetchHabitSummary(habit.id)
    } catch (e) {
      app.showToast(String(e), true)
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

  async function deleteHabit(id: string) {
    try {
      await habitsApi.deleteHabit(id)
      if (selectedHabit?.id === id) { selectedHabit = null; summary = null }
      await load()
      app.showToast(i18n.t('habits-toast-habit-deleted', 'Habit deleted'))
    } catch (e) {
      app.showToast(String(e), true)
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
    return `${days[0]} – ${days[days.length - 1]} ${monthNames[month - 1].slice(0, 3)}`
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

  // When the month grid is wider than the screen, scroll it so today sits at
  // the right edge (recent days stay in view). When the whole month fits there
  // is no overflow, so it stays put. Other months (no "today") start at day 1.
  // `_dep` re-runs it on month/data changes.
  function autoScrollToToday(node: HTMLElement, _dep: unknown) {
    const update = () => requestAnimationFrame(() => {
      const target = node.querySelector('.day-num.is-today') as HTMLElement | null
      if (!target) { node.scrollLeft = 0; return }
      const nodeRect = node.getBoundingClientRect()
      const tRect = target.getBoundingClientRect()
      // Put today near the scroll area's right edge, inset enough to clear the
      // edge fade; the card padding provides the gap to the border.
      const delta = tRect.right - nodeRect.right + 18
      node.scrollTo({ left: node.scrollLeft + delta })
    })
    update()
    return { update }
  }

  function trailingStreak(habit: HabitDto, daysInMonth: number): number {
    const lastDay = viewingCurrentMonth ? todayDay : daysInMonth
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
        <!-- Mobile: single-week view (no horizontal scroll) -->
        <div class="week-view">
          <div class="week-pager">
            <button class="nav-arrow" aria-label="Previous week" onclick={prevWeek}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 19l-7-7 7-7"/></svg>
            </button>
            <span class="week-range">{weekRangeLabel} {year}</span>
            <button class="nav-arrow" aria-label="Next week" onclick={nextWeek}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 5l7 7-7 7"/></svg>
            </button>
          </div>
          <div class="wk-dow">
            {#each WEEKDAYS as wd, i}
              <span class:weekend={i === 0 || i === 6}>{wd}</span>
            {/each}
          </div>
          <div class="wk-dates">
            {#each currentWeek as d}
              <span class:is-today={d === todayDay}>{d ?? ''}</span>
            {/each}
          </div>
          {#each habitsData.habits as habit}
            {@const streak = trailingStreak(habit, habitsData.days_in_month)}
            <div class="wk-row" class:selected={selectedHabit?.id === habit.id}>
              <button
                class="wk-name"
                class:active={selectedHabit?.id === habit.id}
                onclick={() => selectHabit(habit)}
                style="--habit-accent: {habit.color}"
                aria-expanded={selectedHabit?.id === habit.id}
              >
                <span class="habit-name-text">{habit.name}</span>
                {#if streak > 0}
                  <span class="streak-pill" style="color: {habit.color}; border-color: {habit.color}40">
                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2s4 4 4 8a4 4 0 01-8 0c0-1 .3-2 .8-3C10 8 10 6 12 2zm0 11a3 3 0 110 6 3 3 0 010-6z"/></svg>
                    {streak}
                  </span>
                {/if}
              </button>
              <div class="wk-cells">
                {#each currentWeek as d}
                  {#if d === null}
                    <span class="wk-cell empty"></span>
                  {:else}
                    <button
                      class="wk-cell"
                      class:done={habit.days[d]}
                      class:is-today={d === todayDay}
                      style={habit.days[d] ? `background: ${habit.color}; border-color: ${habit.color}` : ''}
                      onclick={() => toggleDay(habit.id, d)}
                      aria-label="Day {d}"
                    ></button>
                  {/if}
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {:else}
      <div class="habit-grid">
        <div class="grid-split">
          <!-- Frozen names pane: blends with the card; the days scroll/clip on
               their own to its right, so days vanish at a clean line + gap. -->
          <div class="grid-names">
            <div class="gn-row gn-row-wk"></div>
            <div class="gn-row gn-row-dn">{i18n.t('habits-habit-col', 'Habit')}</div>
            {#each habitsData.habits as habit}
              {@const streak = trailingStreak(habit, habitsData.days_in_month)}
              <button
                class="gn-name"
                class:active={selectedHabit?.id === habit.id}
                onclick={() => selectHabit(habit)}
                style="--habit-accent: {habit.color}"
                aria-expanded={selectedHabit?.id === habit.id}
              >
                <span class="name-left">
                  <svg class="row-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><path d="M9 5l7 7-7 7"/></svg>
                  <span class="habit-name-text">{habit.name}</span>
                </span>
                {#if streak > 0}
                  <span class="streak-pill" style="color: {habit.color}; border-color: {habit.color}40">
                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2s4 4 4 8a4 4 0 01-8 0c0-1 .3-2 .8-3C10 8 10 6 12 2zm0 11a3 3 0 110 6 3 3 0 010-6z"/></svg>
                    {streak}
                  </span>
                {/if}
              </button>
            {/each}
          </div>
          <div class="habit-grid-scroll" use:autoScrollToToday={`${month}-${year}-${loading}`}>
            <div class="grid-header">
              {#each Array(habitsData.days_in_month) as _, i}
                {@const wd = weekdayOf(i + 1)}
                <span class="weekday" class:weekend={wd === 0 || wd === 6}>{WEEKDAYS[wd]}</span>
              {/each}
            </div>
            <div class="grid-header">
              {#each Array(habitsData.days_in_month) as _, i}
                <span class="day-num" class:is-today={i + 1 === todayDay}>{i + 1}</span>
              {/each}
            </div>
            {#each habitsData.habits as habit}
              <div class="grid-row" class:selected={selectedHabit?.id === habit.id}>
                {#each habit.days.slice(1, habitsData.days_in_month + 1) as done, i}
                  <button
                    class="day-cell"
                    class:done
                    class:is-today={i + 1 === todayDay}
                    style={done ? `background: ${habit.color}; border-color: ${habit.color}` : ''}
                    onclick={() => toggleDay(habit.id, i + 1)}
                    aria-label="Day {i + 1}"
                  ></button>
                {/each}
              </div>
            {/each}
          </div>
        </div>
      </div>
      {/if}

      <!-- Selected habit metrics — inline, attached right under the grid -->
      {#if selectedHabit && summary}
        <div class="summary-card" style="border-left: 3px solid {selectedHabit.color}">
          <div class="summary-header">
            <h3>{selectedHabit.name}</h3>
            <div class="summary-actions">
              <button class="icon-btn" onclick={() => openEditHabit(selectedHabit!)}>{i18n.t('habits-edit', 'Edit')}</button>
              <button class="icon-btn danger" onclick={() => deleteHabit(selectedHabit!.id)}>{i18n.t('habits-delete', 'Delete')}</button>
              <button class="icon-btn" aria-label="Close" onclick={() => { selectedHabit = null; summary = null }}>✕</button>
            </div>
          </div>
          <div class="stats-grid">
            <div class="stat">
              <svg class="stat-icon flame" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2s4 4 4 8a4 4 0 01-8 0c0-1 .3-2 .8-3C10 8 10 6 12 2zm0 11a3 3 0 110 6 3 3 0 010-6z"/></svg>
              <span class="stat-val">{summary.current_streak}</span>
              <span class="stat-lbl">{i18n.t('habits-current-streak', 'Current Streak')}</span>
            </div>
            <div class="stat">
              <svg class="stat-icon trophy" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M6 3h12v4a6 6 0 01-12 0V3zM4 5h2M18 5h2M12 13v4M8 21h8M10 17h4"/></svg>
              <span class="stat-val">{summary.best_streak}</span>
              <span class="stat-lbl">{i18n.t('habits-best-streak', 'Best Streak')}</span>
            </div>
            <div class="stat">
              <svg class="stat-icon pct" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M8 15l8-8M9 9h.01M15 15h.01"/></svg>
              <span class="stat-val">{(summary.completion_rate * 100).toFixed(0)}%</span>
              <span class="stat-lbl">{i18n.t('habits-completion', 'Completion')}</span>
            </div>
            <div class="stat">
              <svg class="stat-icon cal" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18M8 3v4M16 3v4"/></svg>
              <span class="stat-val">{summary.last_30_days}</span>
              <span class="stat-lbl">{i18n.t('habits-last-30-days', 'Last 30 Days')}</span>
            </div>
          </div>
        </div>
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
          onyearchange={async (year) => { heatmapYear = year; try { heatmap = await habitsApi.fetchHeatmap(heatmapYear) } catch (e) { app.showToast(String(e), true) } }}
        />
      {/if}

      <!-- Analytics -->
      {#if analytics}
        <div class="analytics-section">
          {#if analytics.radar.categories.length > 0}
            <div class="chart-card">
              <h3>{i18n.t('habits-habit-radar', 'Habit Radar')}</h3>
              <RadarChart data={analytics.radar} />
            </div>
          {/if}
          {#if analytics.weekday_efficiency.labels.length > 0}
            <div class="chart-card">
              <h3>{i18n.t('habits-weekday-efficiency', 'Weekday Efficiency')}</h3>
              <WeekdayChart data={analytics.weekday_efficiency} />
            </div>
          {/if}
          <div class="analytics-card">
            <p class="insight">{analytics.weekly_summary}</p>
          </div>
        </div>
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
    {#if achievements.length === 0}
      <p class="empty">{i18n.t('habits-no-achievements', 'No achievements yet.')}</p>
    {:else}
      <div class="timeline">
        {#each achievements as ach}
          <div class="timeline-item">
            <div class="timeline-dot"></div>
            <div class="timeline-content">
              <span class="ach-title">{ach.title}</span>
              <span class="ach-desc">{ach.description}</span>
              <span class="ach-date">{ach.achieved_at}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
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
<SettingsModal bind:show={showSettings} onclose={() => showSettings = false} />

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
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: var(--text-secondary); text-transform: uppercase; margin: 0; }

  /* Habit grid */
  .habit-grid {
    position: relative;
    overflow: hidden; margin-bottom: 24px;
    /* Padding lives on the card (not the scroll area) so the inner gap survives
       horizontal scrolling — WebKitGTK drops a scroll container's end padding. */
    padding: 14px 16px;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    box-shadow: var(--card-shadow);
  }
  .habit-grid::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
    z-index: 2;
  }
  /* Two-pane layout: frozen names (left) + scrolling/clipping days (right),
     separated by a gap. Days disappear cleanly at the days pane's edge. */
  .grid-split { display: flex; align-items: flex-start; gap: 10px; }

  /* Names pane — static, blends with the card (no panel, no scroll). Row
     heights mirror the days pane so rows line up. */
  .grid-names { flex-shrink: 0; width: 160px; }
  .gn-row { display: flex; align-items: center; font-size: 0.82rem; color: var(--text-secondary); }
  .gn-row-wk { height: 17px; }                     /* weekday header: 14px + 3px margin */
  .gn-row-dn { height: 21px; padding-left: 8px; }  /* day-number header: 18px + 3px margin */
  .gn-name {
    display: flex; align-items: center; justify-content: space-between; gap: 6px;
    width: 100%; height: 32px;                      /* .grid-row: 26px cell + 6px */
    background: none; border: none; cursor: pointer; color: var(--text-secondary);
    text-align: left; font-size: 0.82rem; overflow: hidden; white-space: nowrap;
    padding: 0 8px 0 10px; border-radius: 5px;
    box-shadow: inset 3px 0 0 var(--habit-accent);
    transition: background 0.15s, color 0.15s;
  }
  .gn-name:hover { background: var(--glass-hover); color: var(--text-primary); }
  .gn-name:hover .row-chevron { color: var(--text-secondary); }
  .gn-name.active { background: var(--glass-elevated); color: var(--text-primary); }
  .gn-name.active .row-chevron { transform: rotate(90deg); color: var(--accent); }
  .name-left { display: flex; align-items: center; gap: 5px; overflow: hidden; }
  .row-chevron { width: 12px; height: 12px; flex-shrink: 0; color: var(--text-tertiary); transition: transform 0.18s ease, color 0.15s; }
  .habit-name-text { overflow: hidden; text-overflow: ellipsis; }

  /* Days pane — scrolls horizontally; clips its content at both edges, with a
     small fade so days dissolve into the line / right edge instead of cutting. */
  .habit-grid-scroll {
    flex: 1; min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
    -webkit-mask-image: linear-gradient(to right, transparent 0, #000 14px, #000 calc(100% - 14px), transparent 100%);
    mask-image: linear-gradient(to right, transparent 0, #000 14px, #000 calc(100% - 14px), transparent 100%);
  }
  .habit-grid-scroll::-webkit-scrollbar { display: none; }
  .grid-header, .grid-row { display: flex; align-items: center; gap: 3px; min-width: max-content; }
  .grid-header { margin-bottom: 3px; }
  .grid-row { height: 32px; border-radius: 8px; transition: background 0.15s; }
  .grid-row.selected { background: var(--glass-elevated); box-shadow: inset 0 0 0 1px var(--glass-border); }

  .streak-pill {
    display: inline-flex; align-items: center; gap: 3px; flex-shrink: 0;
    padding: 1px 7px 1px 4px; border-radius: 999px;
    border: 1px solid currentColor;
    font-size: 0.68rem; font-weight: 600;
    background: rgba(255, 255, 255, 0.03);
  }
  .streak-pill svg { width: 10px; height: 10px; }

  .weekday {
    width: 26px; height: 14px; display: flex; align-items: center; justify-content: center;
    font-size: 0.6rem; font-weight: 600; color: var(--text-tertiary);
    text-transform: uppercase; letter-spacing: 0.05em;
  }
  .weekday.weekend { color: var(--accent); opacity: 0.55; }

  .day-num {
    width: 26px; height: 18px; display: flex; align-items: center; justify-content: center;
    font-size: 0.72rem; color: var(--text-tertiary); position: relative;
  }
  .day-num.is-today { color: var(--accent); font-weight: 700; }
  .day-num.is-today::after {
    content: ''; position: absolute; bottom: -2px; left: 50%; transform: translateX(-50%);
    width: 4px; height: 4px; border-radius: 50%; background: var(--accent);
  }

  .day-cell {
    width: 26px; height: 26px; border-radius: 6px; border: 1px solid var(--glass-border);
    background: var(--glass); cursor: pointer; padding: 0;
    transition: transform 0.12s ease, border-color 0.15s, background 0.15s;
  }
  .day-cell:hover {
    border-color: var(--glass-border-hover); background: var(--glass-hover);
    transform: scale(1.15); z-index: 1;
  }
  .day-cell.done { box-shadow: 0 0 8px rgba(255,255,255,0.1); }
  .day-cell.done:hover { transform: scale(1.2); }
  .day-cell.is-today:not(.done) { box-shadow: 0 0 0 1px var(--accent) inset; }
  /* Subtle vertical "today" guide across the whole grid column. */
  .day-num.is-today::before {
    content: ''; position: absolute; top: -3px; left: 50%; transform: translateX(-50%);
    width: 26px; height: 1px; background: var(--accent); opacity: 0.4;
  }

  /* ── Mobile week view ─────────────────────────────────────── */
  .week-view {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    box-shadow: var(--card-shadow);
    padding: 12px 14px 14px; margin-bottom: 24px;
  }
  .week-view::before {
    content: ''; position: absolute; top: 0; left: 0; right: 0;
    height: 1px; background: var(--card-accent-line); opacity: 0.5;
  }
  .week-pager { display: flex; align-items: center; justify-content: center; gap: 14px; margin-bottom: 12px; }
  .week-range { font-size: 0.9rem; color: var(--text-secondary); min-width: 120px; text-align: center; }
  /* Fixed-size cells (capped, shrinkable) spread edge-to-edge — never overflow
     the card and don't blow up to fill the full width. */
  .wk-dow, .wk-dates, .wk-cells {
    display: grid; grid-template-columns: repeat(7, minmax(0, 30px));
    justify-content: space-between;
  }
  .wk-dow span {
    text-align: center; font-size: 0.62rem; font-weight: 600; color: var(--text-tertiary);
    text-transform: uppercase; letter-spacing: 0.05em;
  }
  .wk-dow span.weekend { color: var(--accent); opacity: 0.55; }
  .wk-dates { margin: 2px 0 6px; }
  .wk-dates span { text-align: center; font-size: 0.7rem; color: var(--text-tertiary); }
  .wk-dates span.is-today { color: var(--accent); font-weight: 700; }
  .wk-row { padding: 8px 0; border-top: 1px solid var(--glass-border); border-radius: 8px; }
  .wk-row.selected { background: var(--glass-elevated); }
  .wk-name {
    display: flex; align-items: center; justify-content: space-between; gap: 8px;
    width: 100%; background: none; border: none; cursor: pointer;
    color: var(--text-secondary); text-align: left;
    padding: 2px 4px 8px 10px; margin-bottom: 2px; font-size: 0.9rem;
    box-shadow: inset 3px 0 0 var(--habit-accent); border-radius: 4px;
  }
  .wk-name.active { color: var(--text-primary); }
  .wk-name .habit-name-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .wk-cell {
    aspect-ratio: 1; min-width: 0; border-radius: 7px;
    border: 1px solid var(--glass-border); background: var(--glass);
    cursor: pointer; padding: 0;
    transition: transform 0.1s ease, border-color 0.15s, background 0.15s;
  }
  .wk-cell.empty { background: none; border: none; cursor: default; }
  .wk-cell.done { box-shadow: 0 0 8px rgba(255, 255, 255, 0.1); }
  .wk-cell.is-today:not(.done) { box-shadow: 0 0 0 1px var(--accent) inset; }
  .wk-cell:not(.empty):active { transform: scale(0.93); }

  /* Summary card */
  .summary-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 20px; margin-bottom: 20px; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .summary-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .summary-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .summary-header h3 { margin: 0; font-size: 1rem; color: var(--text-primary); }
  .summary-actions { display: flex; gap: 8px; }
  .icon-btn {
    background: none; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    color: var(--text-secondary); cursor: pointer; padding: 4px 12px; font-size: 0.8rem;
    transition: all 0.15s;
  }
  .icon-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .icon-btn.danger:hover { color: var(--danger); border-color: var(--danger-border); background: var(--danger-bg); }

  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
  .stat {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
    padding: 10px 8px; border-radius: var(--radius-sm);
    background: var(--glass-elevated);
    border: 1px solid var(--glass-border);
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .stat:hover { border-color: var(--glass-border-hover); box-shadow: var(--glass-shadow); }
  .stat-icon { width: 16px; height: 16px; color: var(--text-tertiary); margin-bottom: 2px; }
  .stat-icon.flame { color: #fb923c; }
  .stat-icon.trophy { color: var(--warning); }
  .stat-icon.pct { color: var(--accent); }
  .stat-icon.cal { color: var(--success); }
  .stat-val { font-size: 1.35rem; font-weight: 700; color: var(--text-primary); line-height: 1.1; }
  .stat-lbl { font-size: 0.68rem; color: var(--text-tertiary); margin-top: 2px; text-align: center; }

  .analytics-section { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 24px; }

  /* ── Mobile ───────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .page { padding: 20px 16px; }
    .stats-grid { grid-template-columns: repeat(2, 1fr); }
    .analytics-section { grid-template-columns: 1fr; }
    /* Reflow the header: brand + settings on top, month nav on its own
       centered row so the settings icon always has a stable home. */
    .page-header { flex-wrap: wrap; }
    .settings-btn { order: 0; }
    .month-nav { order: 1; width: 100%; justify-content: center; margin-top: 6px; }
  }
  .chart-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .chart-card::before, .analytics-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .chart-card h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 0 0 8px; }
  .progress-card { margin-bottom: 24px; }
  .analytics-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; grid-column: 1 / -1; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .insight { font-size: 0.85rem; color: var(--text-secondary); margin: 4px 0; }

  /* Timeline */
  .timeline { display: flex; flex-direction: column; gap: 0; padding-left: 20px; }
  .timeline-item { display: flex; gap: 16px; padding: 12px 0; border-left: 2px solid var(--glass-border); padding-left: 16px; position: relative; }
  .timeline-dot {
    position: absolute; left: -7px; top: 16px; width: 12px; height: 12px; border-radius: 50%;
    background: var(--accent); border: 2px solid var(--bg-base);
    box-shadow: 0 0 8px var(--accent-glow);
  }
  .timeline-content { display: flex; flex-direction: column; gap: 2px; }
  .ach-title { font-weight: 600; color: var(--text-primary); font-size: 0.9rem; }
  .ach-desc { font-size: 0.8rem; color: var(--text-secondary); }
  .ach-date { font-size: 0.7rem; color: var(--text-tertiary); }

</style>
