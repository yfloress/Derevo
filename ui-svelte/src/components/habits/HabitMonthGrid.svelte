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
  import type { HabitDto } from '../../lib/types/habits'

  interface Props {
    habits: HabitDto[]
    daysInMonth: number
    todayDay: number
    weekdays: string[]
    selectedId: string | null
    /** Weekday index (0 = Sunday) of a given day of the month. */
    weekdayOf: (day: number) => number
    streakOf: (habit: HabitDto) => number
    onselect: (habit: HabitDto) => void
    ontoggle: (habitId: string, day: number) => void
    /** Changes whenever the grid should re-scroll — the month, typically. */
    scrollKey: string
  }

  let {
    habits, daysInMonth, todayDay, weekdays, selectedId,
    weekdayOf, streakOf, onselect, ontoggle, scrollKey,
  }: Props = $props()

  // When the month grid is wider than the screen, scroll it so today sits at
  // the right edge (recent days stay in view). When the whole month fits there
  // is no overflow, so it stays put. Other months (no "today") start at day 1.
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
</script>

<div class="habit-grid">
  <div class="grid-split">
    <!-- Frozen names pane: blends with the card; the days scroll/clip on
         their own to its right, so days vanish at a clean line + gap. -->
    <div class="grid-names">
      <div class="gn-row gn-row-wk"></div>
      <div class="gn-row gn-row-dn">{i18n.t('habits-habit-col', 'Habit')}</div>
      {#each habits as habit}
        {@const streak = streakOf(habit)}
        <button
          class="gn-name"
          class:active={selectedId === habit.id}
          onclick={() => onselect(habit)}
          style="--habit-accent: {habit.color}"
          aria-expanded={selectedId === habit.id}
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
    <div class="habit-grid-scroll" use:autoScrollToToday={scrollKey}>
      <div class="grid-header">
        {#each Array(daysInMonth) as _, i}
          {@const wd = weekdayOf(i + 1)}
          <span class="weekday" class:weekend={wd === 0 || wd === 6}>{weekdays[wd]}</span>
        {/each}
      </div>
      <div class="grid-header">
        {#each Array(daysInMonth) as _, i}
          <span class="day-num" class:is-today={i + 1 === todayDay}>{i + 1}</span>
        {/each}
      </div>
      {#each habits as habit}
        <div class="grid-row" class:selected={selectedId === habit.id}>
          {#each habit.days.slice(1, daysInMonth + 1) as done, i}
            <button
              class="day-cell"
              class:done
              class:is-today={i + 1 === todayDay}
              style={done ? `background: ${habit.color}; border-color: ${habit.color}` : ''}
              onclick={() => ontoggle(habit.id, i + 1)}
              aria-label="Day {i + 1}"
            ></button>
          {/each}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
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
  /* Subtle vertical "today" guide across the whole grid column. */
  .day-num.is-today::before {
    content: ''; position: absolute; top: -3px; left: 50%; transform: translateX(-50%);
    width: 26px; height: 1px; background: var(--accent); opacity: 0.4;
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
</style>
