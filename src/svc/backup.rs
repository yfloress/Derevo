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

use crate::db::Database;
use crate::dto::ImportSummary;
use crate::error::{AppError, DbError};
use crate::models::BackupData;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// On-disk format of a backup file. Older files are still readable; a file
/// claiming a newer version is refused rather than half-understood.
pub const FORMAT_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
    pub format: i32,
    pub app: String,
    pub exported_at: String,
    pub data: BackupData,
}

/// Read in two stages: the header is checked before the payload is parsed at
/// all. A file from a future version will not fit `BackupData`, and reporting
/// that as a malformed file would send the user hunting for the wrong problem.
#[derive(Deserialize)]
struct BackupHeader {
    format: i32,
    app: String,
    data: serde_json::Value,
}

pub struct BackupService;

impl BackupService {
    pub fn export_to_file(db: &Database, path: &Path) -> Result<ImportSummary, AppError> {
        let data = db.export_bundle()?;
        let summary = summarize(&data);
        let file = BackupFile {
            format: FORMAT_VERSION,
            app: "derevo".to_string(),
            exported_at: chrono::Local::now().to_rfc3339(),
            data,
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| AppError::Io(format!("could not encode the backup: {e}")))?;
        std::fs::write(path, json).map_err(|e| AppError::Io(e.to_string()))?;
        Ok(summary)
    }

    /// Replaces everything currently stored. The caller is responsible for
    /// warning the user first.
    pub fn import_from_file(db: &Database, path: &Path) -> Result<ImportSummary, AppError> {
        let raw = std::fs::read_to_string(path).map_err(|e| AppError::Io(e.to_string()))?;
        let header: BackupHeader =
            serde_json::from_str(&raw).map_err(|e| AppError::InvalidBackup(e.to_string()))?;

        if header.app != "derevo" {
            return Err(AppError::InvalidBackup(format!(
                "it was written by {}",
                header.app
            )));
        }
        if header.format > FORMAT_VERSION {
            return Err(AppError::UnsupportedBackup(header.format));
        }

        let data: BackupData = serde_json::from_value(header.data)
            .map_err(|e| AppError::InvalidBackup(e.to_string()))?;

        let summary = summarize(&data);
        db.replace_all(&data).map_err(as_import_error)?;
        Ok(summary)
    }
}

fn summarize(data: &BackupData) -> ImportSummary {
    ImportSummary {
        habits: data.habits.len() as i32,
        logs: data.habit_logs.len() as i32,
        rewards: data.streak_rewards.len() as i32,
        goals: data.goals.len() as i32,
        achievements: data.achievements.len() as i32,
    }
}

/// A constraint failure while restoring means the file's rows do not hang
/// together — a dangling habit_id, a duplicate key. That is a bad file, not a
/// broken database, and the user needs to hear the difference.
fn as_import_error(err: DbError) -> AppError {
    match err {
        DbError::Sqlite(rusqlite::Error::SqliteFailure(inner, msg))
            if inner.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::InvalidBackup(msg.unwrap_or_else(|| "inconsistent rows".to_string()))
        }
        other => AppError::Database(other),
    }
}
