use std::{fs, path::Path};

use rusqlite::{params, Connection};
use serde::Serialize;

const LATEST_SCHEMA_VERSION: i64 = 1;
const RECENT_NOTE_LIMIT: usize = 100;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i64,
    pub body: String,
    pub created_at: i64,
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(app_data_dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(app_data_dir).map_err(|error| {
            format!("Could not create the Scattered Thoughts data directory: {error}")
        })?;

        let connection = Connection::open(app_data_dir.join("quick-note.sqlite3"))
            .map_err(|error| format!("Could not open the Scattered Thoughts database: {error}"))?;
        let database = Self { connection };
        database.migrate()?;
        Ok(database)
    }

    fn migrate(&self) -> Result<(), String> {
        let version: i64 = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| format!("Could not read database schema version: {error}"))?;

        if version > LATEST_SCHEMA_VERSION {
            return Err(
                "The Scattered Thoughts database was created by a newer app version.".to_owned(),
            );
        }

        if version == 0 {
            self.connection
                .execute_batch(
                    "
                    BEGIN;
                    CREATE TABLE notes (
                      id INTEGER PRIMARY KEY,
                      body TEXT NOT NULL CHECK (length(trim(body)) > 0),
                      created_at INTEGER NOT NULL
                    );
                    PRAGMA user_version = 1;
                    COMMIT;
                    ",
                )
                .map_err(|error| {
                    format!("Could not create the Scattered Thoughts database: {error}")
                })?;
        }

        Ok(())
    }

    pub fn save_note(&self, body: &str, created_at: i64) -> Result<Note, String> {
        if body.trim().is_empty() {
            return Err("A note needs text.".to_owned());
        }

        self.connection
            .execute(
                "INSERT INTO notes (body, created_at) VALUES (?1, ?2)",
                params![body, created_at],
            )
            .map_err(|error| format!("Could not save note: {error}"))?;

        Ok(Note {
            id: self.connection.last_insert_rowid(),
            body: body.to_owned(),
            created_at,
        })
    }

    pub fn list_recent_notes(&self) -> Result<Vec<Note>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, body, created_at
                 FROM notes
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?1",
            )
            .map_err(|error| format!("Could not prepare recent notes query: {error}"))?;

        let rows = statement
            .query_map(params![RECENT_NOTE_LIMIT], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    body: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|error| format!("Could not load recent notes: {error}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read recent notes: {error}"))
    }

    #[cfg(test)]
    pub fn note_count(&self) -> Result<i64, String> {
        self.connection
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> (tempfile::TempDir, Database) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let database = Database::open(directory.path()).expect("database opens");
        (directory, database)
    }

    #[test]
    fn migration_creates_a_fresh_database() {
        let (_directory, database) = database();
        assert_eq!(database.note_count().expect("note count"), 0);
    }

    #[test]
    fn blank_notes_are_rejected() {
        let (_directory, database) = database();
        let error = database
            .save_note("  ", 1)
            .expect_err("blank note should fail");
        assert_eq!(error, "A note needs text.");
    }

    #[test]
    fn notes_keep_multiline_bodies_and_timestamps() {
        let (_directory, database) = database();
        let note = database.save_note("first\nsecond", 42).expect("note saves");
        assert_eq!(note.body, "first\nsecond");
        assert_eq!(note.created_at, 42);
    }

    #[test]
    fn recent_notes_are_newest_first_and_capped() {
        let (_directory, database) = database();
        for timestamp in 1..=101 {
            database
                .save_note(&format!("note {timestamp}"), timestamp)
                .expect("note saves");
        }

        let notes = database.list_recent_notes().expect("notes load");
        assert_eq!(notes.len(), RECENT_NOTE_LIMIT);
        assert_eq!(notes.first().expect("first note").body, "note 101");
        assert_eq!(notes.last().expect("last note").body, "note 2");
    }
}
