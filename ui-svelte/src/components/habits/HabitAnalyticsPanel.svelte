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
  import { i18n } from '../../lib/stores/i18n.svelte'
  import RadarChart from '../charts/RadarChart.svelte'
  import WeekdayChart from '../charts/WeekdayChart.svelte'
  import type { HabitAnalyticsResponse } from '../../lib/types/habits'

  interface Props {
    analytics: HabitAnalyticsResponse
  }

  let { analytics }: Props = $props()

  // The backend sends weekday positions, never rendered names, so the labels
  // and the summary sentence both follow the interface language.
  const SHORT_KEYS = [
    'weekday-short-mon', 'weekday-short-tue', 'weekday-short-wed', 'weekday-short-thu',
    'weekday-short-fri', 'weekday-short-sat', 'weekday-short-sun',
  ]
  const LONG_KEYS = [
    'weekday-long-mon', 'weekday-long-tue', 'weekday-long-wed', 'weekday-long-thu',
    'weekday-long-fri', 'weekday-long-sat', 'weekday-long-sun',
  ]

  const weekdayData = $derived({
    labels: analytics.weekday_efficiency.labels.map((label, i) =>
      i < SHORT_KEYS.length ? i18n.t(SHORT_KEYS[i]) : label,
    ),
    values: analytics.weekday_efficiency.values,
  })

  const summary = $derived.by(() => {
    const best = analytics.best_weekday
    if (best === null || best < 0 || best >= LONG_KEYS.length) {
      return i18n.t('habits-no-data-yet')
    }
    return `${i18n.t('habits-best-day')} ${i18n.t(LONG_KEYS[best])}`
  })
</script>

<div class="analytics-section">
  {#if analytics.radar.categories.length > 0}
    <div class="chart-card">
      <h3>{i18n.t('habits-habit-radar', 'Habit Radar')}</h3>
      <RadarChart data={analytics.radar} />
    </div>
  {/if}
  {#if weekdayData.labels.length > 0}
    <div class="chart-card">
      <h3>{i18n.t('habits-weekday-efficiency', 'Weekday Efficiency')}</h3>
      <WeekdayChart data={weekdayData} />
    </div>
  {/if}
  <div class="analytics-card">
    <p class="insight">{summary}</p>
  </div>
</div>

<style>
  .analytics-section { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 24px; }

  @media (max-width: 640px) {
    .analytics-section { grid-template-columns: 1fr; }
  }
</style>
