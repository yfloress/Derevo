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
    categories: string[]
    values: number[]
    maxValue: number
  }
  let { categories, values, maxValue = 1 }: Props = $props()
</script>

<div class="radar-chart">
  <div class="radar-title">Habit Radar</div>
  {#if categories.length === 0}
    <div class="radar-empty">No data yet</div>
  {:else}
    <div class="radar-bars">
      {#each categories as cat, i}
        <div class="radar-row">
          <span class="radar-label">{cat}</span>
          <div class="radar-bar-track">
            <div
              class="radar-bar-fill"
              style="width: {maxValue > 0 ? (values[i] / maxValue * 100) : 0}%"
            ></div>
          </div>
          <span class="radar-value">{values[i]}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .radar-chart { padding: 1rem; }
  .radar-title { font-size: 0.75rem; text-transform: uppercase; color: #888; margin-bottom: 0.75rem; }
  .radar-empty { color: #555; font-size: 0.85rem; }
  .radar-bars { display: flex; flex-direction: column; gap: 0.4rem; }
  .radar-row { display: flex; align-items: center; gap: 0.5rem; }
  .radar-label { width: 80px; font-size: 0.75rem; color: #aaa; text-align: right; flex-shrink: 0; }
  .radar-bar-track { flex: 1; height: 10px; background: #1a1a1a; border-radius: 5px; overflow: hidden; }
  .radar-bar-fill { height: 100%; background: #8b5cf6; border-radius: 5px; transition: width 0.5s; }
  .radar-value { width: 30px; font-size: 0.7rem; color: #888; text-align: right; flex-shrink: 0; }
</style>
