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

import { app } from '../stores/app.svelte'

export const chartLight = {
  tooltipBg: '#ffffff',
  tooltipBorder: 'rgba(5, 150, 105, 0.22)',
  tooltipText: '#0f1f18',
  muted: '#7a8f84',
  axisLine: 'rgba(15, 31, 24, 0.14)',
  splitLine: 'rgba(15, 31, 24, 0.07)',
  label: '#3c5448',
  labelDim: '#5c7567',
  sliceBorder: '#ffffff',
  centerLabel: '#0f1f18',
  accent: '#059669',
  positive: '#16a34a',
  negative: '#dc2626',
} as const

export function pick<T>(darkVal: T, lightVal: T): T {
  return app.darkMode ? darkVal : lightVal
}
