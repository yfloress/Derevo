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

mod commands;

use derevo::db::Database;
use std::sync::Arc;
use tauri::Manager;

#[cfg(not(target_os = "android"))]
use directories::ProjectDirs;

#[cfg(not(target_os = "android"))]
fn get_app_data_dir() -> std::path::PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "Derevo") {
        let data_dir = proj_dirs.data_dir().to_path_buf();
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            log::error!("Failed to create data directory: {}", e);
        }
        data_dir
    } else {
        log::error!("Could not determine application data directory, using current directory");
        std::path::PathBuf::from(".")
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let detected_lang = derevo::i18n::detect_system_language();
    derevo::i18n::init(&detected_lang);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            #[cfg(not(target_os = "android"))]
            let app_data_dir = get_app_data_dir();
            #[cfg(target_os = "android")]
            let app_data_dir = {
                let dir = app.path().app_data_dir()?;
                std::fs::create_dir_all(&dir).ok();
                dir
            };

            let db_path = app_data_dir.join("derevo.db");
            let db = Database::init(db_path).expect("Failed to initialize database");
            app.manage(Arc::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch_habits,
            commands::create_habit,
            commands::update_habit,
            commands::delete_habit,
            commands::toggle_habit,
            commands::fetch_habit_summary,
            commands::fetch_heatmap,
            commands::fetch_habit_analytics,
            commands::fetch_rewards,
            commands::create_streak_reward,
            commands::update_streak_reward,
            commands::delete_streak_reward,
            commands::add_milestone,
            commands::fetch_goals,
            commands::create_goal,
            commands::update_goal,
            commands::update_goal_with_checkpoints,
            commands::delete_goal,
            commands::complete_goal,
            commands::archive_goal,
            commands::add_checkpoint,
            commands::update_checkpoint,
            commands::delete_checkpoint,
            commands::toggle_checkpoint,
            commands::fetch_achievements,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
