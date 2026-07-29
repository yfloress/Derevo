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

mod v001_initial_schema;

use super::DbError;
use rusqlite::Connection;

pub const SCHEMA_VERSION: i64 = 1;

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: fn(&Connection) -> Result<(), DbError>,
}

pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        name: "initial_schema",
        up: v001_initial_schema::up,
    }]
}

pub fn get_current_version(conn: &Connection) -> Result<i64, DbError> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(DbError::Sqlite)
}

pub fn run_pending(conn: &Connection, from_version: i64, to_version: i64) -> Result<(), DbError> {
    for migration in get_migrations() {
        if migration.version > from_version && migration.version <= to_version {
            log::info!(
                "[Migration] Applying v{}: {}",
                migration.version,
                migration.name
            );
            (migration.up)(conn)?;
            conn.pragma_update(None, "user_version", migration.version)?;
            log::info!("[Migration] Completed v{}", migration.version);
        }
    }
    Ok(())
}
