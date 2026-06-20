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

let currentLang = $state('en')

const translations: Record<string, Record<string, string>> = {
  en: {
    'app-name': 'DEREVO',
    'habits-title': 'HABITS',
    'habits-tab-habits': 'Habits',
    'habits-tab-rewards': 'Rewards',
    'habits-tab-history': 'History',
    'habits-loading': 'Loading...',
    'habits-no-habits': 'No habits yet.',
    'rewards-new-reward': 'New Reward',
    'rewards-no-rewards': 'No streak rewards yet.',
    'goals-title': 'Goals',
    'goals-no-goals': 'No goals yet.',
    'achievements-title': 'Achievements',
    'form-save': 'Save',
    'form-cancel': 'Cancel',
    'form-delete': 'Delete',
    'form-name': 'Name',
    'form-description': 'Description',
    'form-color': 'Color',
  },
  es: {
    'app-name': 'DEREVO',
    'habits-title': 'HÁBITOS',
    'habits-tab-habits': 'Hábitos',
    'habits-tab-rewards': 'Recompensas',
    'habits-tab-history': 'Historial',
    'habits-loading': 'Cargando...',
    'habits-no-habits': 'Aún no tienes hábitos.',
    'rewards-new-reward': 'Nueva Recompensa',
    'rewards-no-rewards': 'Aún no hay recompensas.',
    'goals-title': 'Metas',
    'goals-no-goals': 'Aún no hay metas.',
    'achievements-title': 'Logros',
    'form-save': 'Guardar',
    'form-cancel': 'Cancelar',
    'form-delete': 'Eliminar',
  },
}

export const i18n = {
  get lang() { return currentLang },
  setLanguage(lang: string) { currentLang = lang },
  t(key: string): string {
    return translations[currentLang]?.[key] ?? translations.en?.[key] ?? key
  },
}
