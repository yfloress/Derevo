<!-- Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
     Copyright (C) 2026  Kyronix

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
  import './app.css'
  import { app } from './lib/stores/app.svelte'
  import HabitsPage from './pages/HabitsPage.svelte'
  import Toast from './components/Toast.svelte'

  // Reflect the chosen theme on <html> so the .light-mode CSS variables apply.
  $effect(() => {
    document.documentElement.classList.toggle('light-mode', !app.darkMode)
  })
</script>

<main class="app-shell">
  <div class="dragon-bg"></div>
  <HabitsPage />
</main>

<Toast />
