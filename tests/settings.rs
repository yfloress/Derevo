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

mod common;

use common::TestDb;
use derevo::svc::SettingsService;
use derevo::svc::settings::{KEY_LANGUAGE, KEY_THEME};

#[test]
fn a_fresh_database_reports_the_defaults() {
    let db = TestDb::new();
    let settings = SettingsService::get_all(&db).unwrap();
    assert_eq!(settings.theme, "dark");
    assert_eq!(settings.language, "en");
}

#[test]
fn the_theme_and_language_survive_a_reopen() {
    let db = TestDb::new();
    SettingsService::set_theme(&db, "light").unwrap();
    SettingsService::set_language(&db, "es").unwrap();

    let settings = SettingsService::get_all(&db).unwrap();
    assert_eq!(settings.theme, "light");
    assert_eq!(settings.language, "es");
}

#[test]
fn setting_the_same_key_twice_overwrites_it() {
    let db = TestDb::new();
    SettingsService::set_theme(&db, "light").unwrap();
    SettingsService::set_theme(&db, "dark").unwrap();
    assert_eq!(SettingsService::get_all(&db).unwrap().theme, "dark");
}

#[test]
fn an_unknown_value_is_refused() {
    let db = TestDb::new();
    assert_eq!(
        SettingsService::set_theme(&db, "neon").unwrap_err().kind(),
        "validation"
    );
    assert_eq!(
        SettingsService::set_language(&db, "fr").unwrap_err().kind(),
        "validation"
    );
}

/// A value written by a future version, or edited by hand, must not leave the
/// interface stuck on something it cannot render.
#[test]
fn a_stored_value_the_app_does_not_know_falls_back_to_the_default() {
    let db = TestDb::new();
    db.set_setting(KEY_THEME, "solarized").unwrap();
    db.set_setting(KEY_LANGUAGE, "de").unwrap();

    let settings = SettingsService::get_all(&db).unwrap();
    assert_eq!(settings.theme, "dark");
    assert_eq!(settings.language, "en");
}
