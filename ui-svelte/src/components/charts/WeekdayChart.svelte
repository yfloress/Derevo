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
  interface Props {
    labels: string[]
    values: number[]
  }
  let { labels, values }: Props = $props()
</script>

<div class="weekday-chart">
  <div class="chart-title">Weekday Efficiency</div>
  {#if labels.length === 0}
    <div class="chart-empty">No data yet</div>
  {:else}
    <div class="chart-bars">
      {#each labels as label, i}
        <div class="bar-col">
          <div class="bar-track">
            <div class="bar-fill" style="height: {(values[i] * 100)}%"></div>
          </div>
          <span class="bar-label">{label}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .weekday-chart { padding: 1rem; }
  .chart-title { font-size: 0.75rem; text-transform: uppercase; color: #888; margin-bottom: 0.75rem; }
  .chart-empty { color: #555; font-size: 0.85rem; }
  .chart-bars { display: flex; justify-content: space-between; align-items: flex-end; height: 120px; gap: 0.25rem; }
  .bar-col { display: flex; flex-direction: column; align-items: center; flex: 1; height: 100%; justify-content: flex-end; }
  .bar-track { width: 100%; max-width: 36px; height: 80%; background: #1a1a1a; border-radius: 4px 4px 0 0; position: relative; display: flex; align-items: flex-end; }
  .bar-fill { width: 100%; background: #8b5cf6; border-radius: 4px 4px 0 0; transition: height 0.5s; min-height: 2px; }
  .bar-label { font-size: 0.65rem; color: #666; margin-top: 0.35rem; }
</style>
