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
  import { confirm as confirmDialog, open, save } from '@tauri-apps/plugin-dialog'
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as habitsApi from '../lib/api/habits'
  import * as backupApi from '../lib/api/backup'
  import { errorMessage } from '../lib/api/errors'
  import { askForPermission, permissionGranted } from '../lib/reminders'
  import type { ArchivedHabitDto } from '../lib/types/habits'

  interface Props {
    show: boolean
    onclose: () => void
    /** Called after anything that changes what the tracking grid should show. */
    ondatachange: () => Promise<void>
  }

  let { show = $bindable(false), onclose, ondatachange }: Props = $props()

  let archived = $state<ArchivedHabitDto[]>([])
  let notificationsAllowed = $state(true)
  let busy = $state(false)

  $effect(() => {
    if (show) {
      void loadArchived()
      void permissionGranted().then((granted) => { notificationsAllowed = granted })
    }
  })

  async function loadArchived() {
    try {
      archived = await habitsApi.fetchArchivedHabits()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function restore(id: string) {
    try {
      await habitsApi.restoreHabit(id)
      await loadArchived()
      await ondatachange()
      app.showToast(i18n.t('habits-toast-habit-restored'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function purge(id: string) {
    if (!(await confirmDialog(i18n.t('habits-delete-confirm')))) return
    try {
      await habitsApi.deleteHabit(id)
      await loadArchived()
      await ondatachange()
      app.showToast(i18n.t('habits-toast-habit-deleted', 'Habit deleted'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function exportBackup() {
    const path = await save({
      defaultPath: backupApi.defaultBackupName(),
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path) return
    busy = true
    try {
      const summary = await backupApi.exportBackup(path)
      app.showToast(`${i18n.t('settings-toast-exported')} (${summary.habits})`)
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      busy = false
    }
  }

  async function importBackup() {
    const path = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (typeof path !== 'string') return
    if (!(await confirmDialog(i18n.t('settings-import-confirm')))) return
    busy = true
    try {
      const summary = await backupApi.importBackup(path)
      await loadArchived()
      await ondatachange()
      app.showToast(`${i18n.t('settings-toast-imported')} (${summary.habits})`)
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      busy = false
    }
  }

  async function allowNotifications() {
    notificationsAllowed = await askForPermission()
  }

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

      <div class="settings-scroll">
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

        {#if !notificationsAllowed}
          <div class="setting-row">
            <span class="setting-label">{i18n.t('settings-notifications')}</span>
            <button class="secondary-btn small" onclick={allowNotifications}>
              {i18n.t('settings-notifications-request')}
            </button>
          </div>
        {/if}

        <div class="setting-block">
          <span class="setting-label">{i18n.t('settings-data')}</span>
          <p class="setting-hint">{i18n.t('settings-export-hint')}</p>
          <div class="setting-buttons">
            <button class="secondary-btn small" onclick={exportBackup} disabled={busy}>
              {i18n.t('settings-export')}
            </button>
            <button class="secondary-btn small" onclick={importBackup} disabled={busy}>
              {i18n.t('settings-import')}
            </button>
          </div>
        </div>

        <div class="setting-block">
          <span class="setting-label">{i18n.t('settings-archived')}</span>
          {#if archived.length === 0}
            <p class="setting-hint">{i18n.t('settings-archived-empty')}</p>
          {:else}
            <ul class="archived-list">
              {#each archived as habit}
                <li class="archived-row" style="--habit-accent: {habit.color}">
                  <span class="archived-name">
                    {habit.name}
                    <span class="archived-meta">{habit.log_count} {i18n.t('settings-archived-logs')}</span>
                  </span>
                  <span class="archived-actions">
                    <button class="secondary-btn small" onclick={() => restore(habit.id)}>
                      {i18n.t('settings-restore')}
                    </button>
                    <button class="secondary-btn small danger" onclick={() => purge(habit.id)}>
                      {i18n.t('form-delete', 'Delete')}
                    </button>
                  </span>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('action-close', 'Close')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-modal { max-width: 460px; }
  /* The archived list can run long; the modal itself must not grow past the
     viewport or the Close button walks off the bottom. */
  .settings-scroll { max-height: min(60vh, 460px); overflow-y: auto; }

  .setting-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 0; border-bottom: 1px solid var(--glass-border); gap: 16px;
  }
  .setting-row:first-of-type { padding-top: 4px; }
  .setting-block { padding: 12px 0; border-bottom: 1px solid var(--glass-border); }
  .setting-label { font-size: 0.9rem; color: var(--text-secondary); }
  .setting-hint { margin: 6px 0 0; font-size: 0.75rem; color: var(--text-tertiary); }
  .setting-buttons { display: flex; gap: 8px; margin-top: 10px; }

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

  .small { padding: 5px 12px; font-size: 0.75rem; }
  .danger { color: var(--danger, #f87171); }

  .archived-list { list-style: none; margin: 10px 0 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  .archived-row {
    display: flex; align-items: center; justify-content: space-between; gap: 10px;
    padding: 8px 10px; border-radius: 8px;
    background: var(--glass); border: 1px solid var(--glass-border);
    box-shadow: inset 3px 0 0 var(--habit-accent);
  }
  .archived-name { display: flex; flex-direction: column; gap: 2px; font-size: 0.85rem; color: var(--text-primary); overflow: hidden; }
  .archived-meta { font-size: 0.7rem; color: var(--text-tertiary); }
  .archived-actions { display: flex; gap: 6px; flex-shrink: 0; }
</style>
