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

interface ToastAction {
  label: string
  handler: () => void | Promise<void>
}

interface Toast {
  message: string
  isError: boolean
  action: ToastAction | null
}

function loadDarkMode(): boolean {
  try {
    return localStorage.getItem('derevo-theme') !== 'light'
  } catch {
    return true
  }
}

class AppState {
  darkMode = $state(loadDarkMode())
  toast = $state<Toast | null>(null)

  setDarkMode(dark: boolean) {
    this.darkMode = dark
    try { localStorage.setItem('derevo-theme', dark ? 'dark' : 'light') } catch { /* ignore */ }
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
