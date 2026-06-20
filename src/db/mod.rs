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

mod habits;
mod migrations;
mod rewards;

use crate::error::DbError;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct Database {
    writer: Mutex<Connection>,
    pool: r2d2::Pool<SqliteConnectionManager>,
    pub path: PathBuf,
}

impl Database {
    pub fn init(db_path: PathBuf) -> Result<Self, DbError> {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|_| DbError::AppDataDir)?;
            }
            #[cfg(unix)]
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                .map_err(|_| DbError::AppDataDir)?;
        }

        let pool_size = std::thread::available_parallelism()
            .map(|n| n.get().clamp(2, 8))
            .unwrap_or(4) as u32;

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = r2d2::Pool::builder()
            .max_size(pool_size)
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(manager)?;

        // Configure the writer connection
        let writer_conn = Connection::open(&db_path)?;
        writer_conn
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(DbError::Sqlite)?;
        writer_conn
            .pragma_update(None, "foreign_keys", true)
            .map_err(DbError::Sqlite)?;
        writer_conn
            .pragma_update(None, "busy_timeout", 5000)
            .map_err(DbError::Sqlite)?;

        let db = Database {
            writer: Mutex::new(writer_conn),
            pool,
            path: db_path,
        };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn read(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, DbError> {
        self.pool.get().map_err(DbError::Pool)
    }

    pub fn write(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.writer.lock().unwrap()
    }

    pub fn with_transaction<T, F>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, DbError>,
    {
        let conn = self.write();
        conn.execute("BEGIN IMMEDIATE", [])?;
        match f(&conn) {
            Ok(value) => {
                conn.execute("COMMIT", [])?;
                Ok(value)
            }
            Err(err) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(err)
            }
        }
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        use rusqlite::params;
        let conn = self.read()?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        use rusqlite::params;
        let conn = self.write();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.write();
        let current_version = migrations::get_current_version(&conn)?;
        if current_version < migrations::SCHEMA_VERSION {
            migrations::run_pending(&conn, current_version, migrations::SCHEMA_VERSION)?;
        }
        Ok(())
    }
}
