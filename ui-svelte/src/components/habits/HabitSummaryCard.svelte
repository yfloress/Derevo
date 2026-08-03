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
  import type { HabitDto, HabitSummary } from '../../lib/types/habits'

  interface Props {
    habit: HabitDto
    summary: HabitSummary
    onedit: () => void
    onarchive: () => void
    onclose: () => void
  }

  let { habit, summary, onedit, onarchive, onclose }: Props = $props()
</script>

<div class="summary-card" style="border-left: 3px solid {habit.color}">
  <div class="summary-header">
    <h3>{habit.name}</h3>
    <div class="summary-actions">
      <button class="icon-btn" onclick={onedit}>{i18n.t('habits-edit', 'Edit')}</button>
      <!-- Archiving keeps the history; permanent deletion lives in Settings,
           behind the archived list, so it cannot be hit by accident here. -->
      <button class="icon-btn" onclick={onarchive}>{i18n.t('habits-archive-habit')}</button>
      <button class="icon-btn" aria-label={i18n.t('action-close', 'Close')} onclick={onclose}>✕</button>
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

<style>
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

  @media (max-width: 640px) {
    .stats-grid { grid-template-columns: repeat(2, 1fr); }
  }
</style>
