use std::{fmt::Write, fs, path::Path};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

const LATEST_SCHEMA_VERSION: i64 = 2;
const RECENT_NOTE_LIMIT: usize = 100;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i64,
    pub body: String,
    pub created_at: i64,
    pub pinned: bool,
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
                      created_at INTEGER NOT NULL,
                      pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1))
                    );
                    CREATE INDEX notes_pinned_created_at ON notes (pinned DESC, created_at DESC, id DESC);
                    CREATE TABLE app_settings (
                      key TEXT PRIMARY KEY,
                      value TEXT NOT NULL
                    );
                    PRAGMA user_version = 2;
                    COMMIT;
                    ",
                )
                .map_err(|error| {
                    format!("Could not create the Scattered Thoughts database: {error}")
                })?;
        } else if version == 1 {
            self.connection
                .execute_batch(
                    "
                    BEGIN;
                    ALTER TABLE notes ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1));
                    CREATE INDEX notes_pinned_created_at ON notes (pinned DESC, created_at DESC, id DESC);
                    CREATE TABLE app_settings (
                      key TEXT PRIMARY KEY,
                      value TEXT NOT NULL
                    );
                    PRAGMA user_version = 2;
                    COMMIT;
                    ",
                )
                .map_err(|error| {
                    format!("Could not migrate the Scattered Thoughts database: {error}")
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
            pinned: false,
        })
    }

    pub fn list_notes(&self, query: &str) -> Result<Vec<Note>, String> {
        let query = query.trim();
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, body, created_at, pinned
                 FROM notes
                 WHERE ?1 = '' OR instr(lower(body), lower(?1)) > 0
                 ORDER BY pinned DESC, created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|error| format!("Could not prepare notes query: {error}"))?;

        let rows = statement
            .query_map(params![query, RECENT_NOTE_LIMIT], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    body: row.get(1)?,
                    created_at: row.get(2)?,
                    pinned: row.get::<_, i64>(3)? != 0,
                })
            })
            .map_err(|error| format!("Could not load notes: {error}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read notes: {error}"))
    }

    pub fn set_note_pinned(&self, id: i64, pinned: bool) -> Result<(), String> {
        let updated = self
            .connection
            .execute(
                "UPDATE notes SET pinned = ?1 WHERE id = ?2",
                params![i64::from(pinned), id],
            )
            .map_err(|error| format!("Could not update the note: {error}"))?;
        if updated == 0 {
            return Err("That note no longer exists.".to_owned());
        }
        Ok(())
    }

    pub fn delete_note(&self, id: i64) -> Result<(), String> {
        let deleted = self
            .connection
            .execute("DELETE FROM notes WHERE id = ?1", params![id])
            .map_err(|error| format!("Could not delete the note: {error}"))?;
        if deleted == 0 {
            return Err("That note no longer exists.".to_owned());
        }
        Ok(())
    }

    pub fn note_body(&self, id: i64) -> Result<String, String> {
        self.connection
            .query_row("SELECT body FROM notes WHERE id = ?1", params![id], |row| {
                row.get(0)
            })
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => "That note no longer exists.".to_owned(),
                _ => format!("Could not read the note: {error}"),
            })
    }

    pub fn diagnostics_enabled(&self) -> Result<bool, String> {
        self.connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'diagnostics_enabled'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map(|value| value.as_deref() == Some("true"))
            .map_err(|error| format!("Could not read diagnostics settings: {error}"))
    }

    pub fn set_diagnostics_enabled(&self, enabled: bool) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO app_settings (key, value) VALUES ('diagnostics_enabled', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![if enabled { "true" } else { "false" }],
            )
            .map_err(|error| format!("Could not save diagnostics settings: {error}"))?;
        Ok(())
    }

    pub fn export_markdown(&self) -> Result<String, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT body, pinned,
                        strftime('%Y-%m-%dT%H:%M:%SZ', created_at, 'unixepoch')
                 FROM notes
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|error| format!("Could not prepare note export: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? != 0,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|error| format!("Could not export notes: {error}"))?;

        let mut output = String::from("# Scattered Thoughts export\n\n");
        for row in rows {
            let (body, pinned, created_at) =
                row.map_err(|error| format!("Could not read an exported note: {error}"))?;
            let pin_label = if pinned { " — Pinned" } else { "" };
            writeln!(output, "## {created_at}{pin_label}\n")
                .map_err(|error| format!("Could not format note export: {error}"))?;
            let fence_length = body.split('`').map(str::len).max().unwrap_or(0).max(2) + 1;
            let fence = "`".repeat(fence_length);
            writeln!(output, "{fence}text")
                .and_then(|_| write!(output, "{body}"))
                .and_then(|_| {
                    if body.ends_with('\n') {
                        Ok(())
                    } else {
                        writeln!(output)
                    }
                })
                .and_then(|_| writeln!(output, "{fence}\n"))
                .map_err(|error| format!("Could not format note export: {error}"))?;
        }
        Ok(output)
    }

    pub fn backup_to(&self, destination: &Path) -> Result<(), String> {
        self.connection
            .backup("main", destination, None)
            .map_err(|error| format!("Could not back up the database: {error}"))
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

        let notes = database.list_notes("").expect("notes load");
        assert_eq!(notes.len(), RECENT_NOTE_LIMIT);
        assert_eq!(notes.first().expect("first note").body, "note 101");
        assert_eq!(notes.last().expect("last note").body, "note 2");
    }

    #[test]
    fn v1_database_migrates_without_losing_notes() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("quick-note.sqlite3");
        let connection = Connection::open(&path).expect("v1 database opens");
        connection
            .execute_batch(
                "
                CREATE TABLE notes (
                  id INTEGER PRIMARY KEY,
                  body TEXT NOT NULL CHECK (length(trim(body)) > 0),
                  created_at INTEGER NOT NULL
                );
                INSERT INTO notes (body, created_at) VALUES ('saved before migration', 1);
                PRAGMA user_version = 1;
                ",
            )
            .expect("v1 database is created");
        drop(connection);

        let database = Database::open(directory.path()).expect("database migrates");
        let notes = database.list_notes("").expect("notes load");
        assert_eq!(notes[0].body, "saved before migration");
        assert!(!notes[0].pinned);
        assert!(!database.diagnostics_enabled().expect("setting loads"));
    }

    #[test]
    fn search_pinning_and_deletion_work() {
        let (_directory, database) = database();
        let first = database
            .save_note("Orchard project", 1)
            .expect("first note");
        let second = database
            .save_note("orchard reminder", 2)
            .expect("second note");
        database.set_note_pinned(first.id, true).expect("note pins");

        let notes = database.list_notes("ORCHARD").expect("notes search");
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].id, first.id);
        assert!(notes[0].pinned);

        database.delete_note(second.id).expect("note deletes");
        assert_eq!(database.note_count().expect("note count"), 1);
    }

    #[test]
    fn exports_multiline_notes_and_creates_valid_backup() {
        let (directory, database) = database();
        let note = database
            .save_note("first line\nsecond line", 1_700_000_000)
            .expect("note saves");
        database.set_note_pinned(note.id, true).expect("note pins");

        let export = database.export_markdown().expect("notes export");
        assert!(export.contains("2023-11-14T22:13:20Z — Pinned"));
        assert!(export.contains("first line\nsecond line"));

        let backup = directory.path().join("backup.sqlite3");
        database.backup_to(&backup).expect("backup creates");
        let backup_connection = Connection::open(backup).expect("backup opens");
        let count: i64 = backup_connection
            .query_row("SELECT count(*) FROM notes", [], |row| row.get(0))
            .expect("backup queries");
        assert_eq!(count, 1);
    }
}
