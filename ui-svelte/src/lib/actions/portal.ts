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

export function portal(node: HTMLElement, target: string | HTMLElement = document.body) {
  let el: HTMLElement | null = null
  const t = typeof target === 'string' ? document.querySelector(target) : target

  if (t) {
    el = node.cloneNode(true) as HTMLElement
    t.appendChild(el)
    node.style.display = 'none'
  }

  return {
    update(newTarget: string | HTMLElement) {
      const nt = typeof newTarget === 'string' ? document.querySelector(newTarget) : newTarget
      if (el && nt) {
        el.remove()
        nt.appendChild(el)
      }
    },
    destroy() {
      if (el) el.remove()
    },
  }
}
