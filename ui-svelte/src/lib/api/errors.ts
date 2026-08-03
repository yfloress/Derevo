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

import { i18n } from '../stores/i18n.svelte'
import type { ErrorDto } from '../types/habits'

/** Mirrors `AppError::kind()` in Rust. Keep the two lists in step. */
const MESSAGE_KEYS: Record<string, string> = {
  database: 'error-database',
  validation: 'error-validation',
  'goal-not-found': 'error-goal-not-found',
  'habit-not-found': 'error-habit-not-found',
  'app-data-dir': 'error-app-data-dir',
  io: 'error-io',
  'invalid-backup': 'error-invalid-backup',
  'unsupported-backup': 'error-unsupported-backup',
}

function isErrorDto(value: unknown): value is ErrorDto {
  return (
    typeof value === 'object' && value !== null &&
    typeof (value as ErrorDto).kind === 'string' &&
    typeof (value as ErrorDto).message === 'string'
  )
}

/**
 * Turns whatever a rejected `invoke` threw into something worth showing.
 * Anything that is not a backend error — a thrown JS exception, a plugin
 * failure — falls through to its own text rather than being disguised.
 */
export function errorMessage(error: unknown): string {
  if (isErrorDto(error)) {
    const key = MESSAGE_KEYS[error.kind]
    if (key) return i18n.t(key)
    return error.message
  }
  if (error instanceof Error) return error.message
  return String(error)
}
