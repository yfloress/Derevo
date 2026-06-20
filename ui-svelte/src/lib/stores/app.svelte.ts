// Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
// Copyright (C) 2026  Kyronix
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

let currentPage = $state('habits')
let sidebarCollapsed = $state(false)
let modal = $state<{ component: string; props: Record<string, unknown> } | null>(null)
let toastMessage = $state<string | null>(null)
const showSidebar: string[] = []

export const app = {
  get currentPage() { return currentPage },
  set currentPage(v: string) { currentPage = v },
  get sidebarCollapsed() { return sidebarCollapsed },
  set sidebarCollapsed(v: boolean) { sidebarCollapsed = v },
  get modal() { return modal },
  setModal(comp: string, props: Record<string, unknown> = {}) { modal = { component: comp, props } },
  closeModal() { modal = null },
  get toast() { return toastMessage },
  setToast(msg: string | null) { toastMessage = msg },
  get showSidebar() { return showSidebar },
}
