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
  import type { HabitDto } from '../../lib/types/habits'

  interface Props {
    habits: HabitDto[]
    /** Seven slots; null where the week runs past the month's edge. */
    week: (number | null)[]
    weekLabel: string
    weekdays: string[]
    todayDay: number
    selectedId: string | null
    streakOf: (habit: HabitDto) => number
    onselect: (habit: HabitDto) => void
    ontoggle: (habitId: string, day: number) => void
    onprev: () => void
    onnext: () => void
  }

  let {
    habits, week, weekLabel, weekdays, todayDay, selectedId,
    streakOf, onselect, ontoggle, onprev, onnext,
  }: Props = $props()
</script>

<div class="week-view">
  <div class="week-pager">
    <button class="nav-arrow" aria-label="Previous week" onclick={onprev}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 19l-7-7 7-7"/></svg>
    </button>
    <span class="week-range">{weekLabel}</span>
    <button class="nav-arrow" aria-label="Next week" onclick={onnext}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 5l7 7-7 7"/></svg>
    </button>
  </div>
  <div class="wk-dow">
    {#each weekdays as wd, i}
      <span class:weekend={i === 0 || i === 6}>{wd}</span>
    {/each}
  </div>
  <div class="wk-dates">
    {#each week as d}
      <span class:is-today={d === todayDay}>{d ?? ''}</span>
    {/each}
  </div>
  {#each habits as habit}
    {@const streak = streakOf(habit)}
    <div class="wk-row" class:selected={selectedId === habit.id}>
      <button
        class="wk-name"
        class:active={selectedId === habit.id}
        onclick={() => onselect(habit)}
        style="--habit-accent: {habit.color}"
        aria-expanded={selectedId === habit.id}
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
        {#each week as d}
          {#if d === null}
            <span class="wk-cell empty"></span>
          {:else}
            <button
              class="wk-cell"
              class:done={habit.days[d]}
              class:is-today={d === todayDay}
              style={habit.days[d] ? `background: ${habit.color}; border-color: ${habit.color}` : ''}
              onclick={() => ontoggle(habit.id, d)}
              aria-label="Day {d}"
            ></button>
          {/if}
        {/each}
      </div>
    </div>
  {/each}
</div>

<style>
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
  .nav-arrow { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .nav-arrow:hover { color: var(--text-primary); }
  .nav-arrow svg { width: 18px; height: 18px; }
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
  .wk-name .habit-name-text { white-space: nowrap; }
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
</style>
