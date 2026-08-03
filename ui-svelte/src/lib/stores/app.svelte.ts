// Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

import * as settingsApi from '../api/settings'
import { i18n } from './i18n.svelte'

interface ToastAction {
  label: string
  handler: () => void | Promise<void>
}

interface Toast {
  message: string
  isError: boolean
  action: ToastAction | null
}

class AppState {
  darkMode = $state(true)
  toast = $state<Toast | null>(null)

  /**
   * Pulls theme and language from the database. Settings live there rather than
   * in localStorage so they travel with a backup and survive the webview being
   * cleared.
   */
  async load() {
    try {
      const settings = await settingsApi.fetchSettings()
      this.darkMode = settings.theme !== 'light'
      i18n.apply(settings.language)
    } catch {
      // A failure here is not worth a toast on startup: the defaults are fine
      // and every other call will surface the same problem with context.
    }
  }

  async setDarkMode(dark: boolean) {
    this.darkMode = dark
    await settingsApi.setTheme(dark ? 'dark' : 'light')
  }

  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  showToast(message: string, isError = false, durationMs = 3000, action: ToastAction | null = null) {
    if (this.toastTimeout) clearTimeout(this.toastTimeout)
    this.toast = { message, isError, action }
    this.toastTimeout = setTimeout(() => {
      this.toast = null
      this.toastTimeout = null
    }, durationMs)
  }

  async runToastAction() {
    const action = this.toast?.action
    if (!action) return
    this.dismissToast()
    await action.handler()
  }

  dismissToast() {
    if (this.toastTimeout) clearTimeout(this.toastTimeout)
    this.toast = null
    this.toastTimeout = null
  }
}

export const app = new AppState()
