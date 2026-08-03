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

use derevo::db::Database;
use std::ops::Deref;
use std::path::PathBuf;

/// A real on-disk database in a temporary directory. SQLite's `:memory:` is
/// per-connection, and Database keeps a writer plus a reader pool, so the tests
/// need a file for the two to see the same data.
pub struct TestDb {
    db: Database,
    dir: PathBuf,
}

impl TestDb {
    pub fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("derevo-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let db = Database::init(dir.join("derevo.db")).expect("database");
        TestDb { db, dir }
    }
}

impl Deref for TestDb {
    type Target = Database;

    fn deref(&self) -> &Database {
        &self.db
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// `days_ago(0)` is today. Matches the format stored in `habit_logs`.
// Every test binary compiles this module separately, so a helper only some of
// them need reads as dead code in the others.
#[allow(dead_code)]
pub fn days_ago(days: i64) -> String {
    (chrono::Local::now().date_naive() - chrono::Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}
