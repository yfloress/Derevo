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

import { invoke } from '@tauri-apps/api/core'
import type { SettingsDto } from '../types/habits'

export async function fetchSettings(): Promise<SettingsDto> {
  return invoke<SettingsDto>('fetch_settings')
}

export async function setTheme(theme: string): Promise<void> {
  return invoke('set_theme', { theme })
}

export async function setLanguage(language: string): Promise<void> {
  return invoke('set_language', { language })
}
