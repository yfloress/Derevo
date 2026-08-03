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
  import type { AchievementDto } from '../../lib/types/habits'

  interface Props {
    achievements: AchievementDto[]
  }

  let { achievements }: Props = $props()

  /** Stored as RFC 3339; only the day matters here. */
  function day(timestamp: string): string {
    const parsed = new Date(timestamp)
    if (Number.isNaN(parsed.getTime())) return timestamp
    return parsed.toLocaleDateString(i18n.lang)
  }
</script>

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
          <span class="ach-date">{day(ach.achieved_at)}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
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
