use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const LOG_FILE_NAME: &str = "diagnostics.log";
const PREVIOUS_LOG_FILE_NAME: &str = "diagnostics.previous.log";
const MAX_LOG_BYTES: u64 = 256 * 1024;

pub struct Diagnostics {
    directory: PathBuf,
    enabled: AtomicBool,
    max_log_bytes: u64,
}

impl Diagnostics {
    pub fn new(directory: PathBuf, enabled: bool) -> Self {
        Self {
            directory,
            enabled: AtomicBool::new(enabled),
            max_log_bytes: MAX_LOG_BYTES,
        }
    }

    #[cfg(test)]
    fn with_limit(directory: PathBuf, enabled: bool, max_log_bytes: u64) -> Self {
        Self {
            directory,
            enabled: AtomicBool::new(enabled),
            max_log_bytes,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn record(&self, event: &str) {
        if !self.is_enabled() {
            return;
        }

        let _ = self.append(event);
    }

    fn append(&self, event: &str) -> Result<(), String> {
        fs::create_dir_all(&self.directory)
            .map_err(|error| format!("Could not create diagnostics directory: {error}"))?;
        let current = self.directory.join(LOG_FILE_NAME);
        if current
            .metadata()
            .map(|metadata| metadata.len() >= self.max_log_bytes)
            .unwrap_or(false)
        {
            let previous = self.directory.join(PREVIOUS_LOG_FILE_NAME);
            let _ = fs::remove_file(&previous);
            fs::rename(&current, previous)
                .map_err(|error| format!("Could not rotate diagnostics log: {error}"))?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("Could not read system clock: {error}"))?
            .as_secs();
        fs::write(
            &current,
            format!(
                "{}{} {}\n",
                fs::read_to_string(&current).unwrap_or_default(),
                timestamp,
                event
            ),
        )
        .map_err(|error| format!("Could not write diagnostics log: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_is_disabled_until_enabled() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let diagnostics = Diagnostics::new(directory.path().to_path_buf(), false);
        diagnostics.record("save-note-failed");
        assert!(!directory.path().join(LOG_FILE_NAME).exists());

        diagnostics.set_enabled(true);
        diagnostics.record("save-note-failed");
        let contents = fs::read_to_string(directory.path().join(LOG_FILE_NAME)).expect("log reads");
        assert!(contents.contains("save-note-failed"));
    }

    #[test]
    fn logs_rotate_at_the_size_limit() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let diagnostics = Diagnostics::with_limit(directory.path().to_path_buf(), true, 1);
        diagnostics.record("first");
        diagnostics.record("second");

        assert!(directory.path().join(LOG_FILE_NAME).exists());
        assert!(directory.path().join(PREVIOUS_LOG_FILE_NAME).exists());
    }
}
