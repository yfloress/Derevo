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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'

  interface Props {
    show: boolean
    onclose: () => void
  }

  let { show = $bindable(false), onclose }: Props = $props()

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal settings-modal">
      <h3>{i18n.t('settings-title', 'Settings')}</h3>

      <div class="setting-row">
        <span class="setting-label">{i18n.t('settings-appearance', 'Appearance')}</span>
        <div class="segmented">
          <button class:active={app.darkMode} onclick={() => app.setDarkMode(true)}>
            {i18n.t('settings-theme-dark', 'Dark')}
          </button>
          <button class:active={!app.darkMode} onclick={() => app.setDarkMode(false)}>
            {i18n.t('settings-theme-light', 'Light')}
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span class="setting-label">{i18n.t('settings-language', 'Language')}</span>
        <div class="segmented">
          <button class:active={i18n.lang === 'en'} onclick={() => i18n.setLanguage('en')}>EN</button>
          <button class:active={i18n.lang === 'es'} onclick={() => i18n.setLanguage('es')}>ES</button>
        </div>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('action-close', 'Close')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-modal { max-width: 400px; }
  .setting-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 0; border-bottom: 1px solid var(--glass-border); gap: 16px;
  }
  .setting-row:first-of-type { padding-top: 4px; }
  .setting-label { font-size: 0.9rem; color: var(--text-secondary); }
  .segmented {
    display: flex; gap: 2px; padding: 3px;
    background: var(--glass); border: 1px solid var(--glass-border);
    border-radius: 10px;
  }
  .segmented button {
    padding: 6px 16px; border: none; background: transparent;
    color: var(--text-secondary); font-size: 0.8rem; font-weight: 500;
    border-radius: 7px; cursor: pointer; transition: all 0.18s;
  }
  .segmented button:hover:not(.active) { color: var(--text-primary); background: var(--glass-hover); }
  .segmented button.active {
    color: var(--text-on-accent);
    background: linear-gradient(135deg, var(--accent) 0%, var(--accent-hover) 100%);
    box-shadow: 0 2px 8px var(--accent-glow);
  }
</style>
