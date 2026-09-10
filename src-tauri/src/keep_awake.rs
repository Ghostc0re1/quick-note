use std::sync::{mpsc, Mutex};

#[cfg(target_os = "windows")]
use std::thread;

pub struct KeepAwake {
    enabled: Mutex<bool>,
    #[cfg(target_os = "windows")]
    sender: mpsc::Sender<Command>,
}

#[cfg(target_os = "windows")]
enum Command {
    Set {
        enabled: bool,
        reply: mpsc::SyncSender<Result<(), String>>,
    },
}

impl KeepAwake {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || {
                for command in receiver {
                    match command {
                        Command::Set { enabled, reply } => {
                            let _ = reply.send(set_execution_state(enabled));
                        }
                    }
                }
            });
            Self {
                enabled: Mutex::new(false),
                sender,
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self {
                enabled: Mutex::new(false),
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.lock().map(|enabled| *enabled).unwrap_or(false)
    }

    pub fn set_enabled(&self, requested: bool) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let (reply_sender, reply_receiver) = mpsc::sync_channel(0);
            self.sender
                .send(Command::Set {
                    enabled: requested,
                    reply: reply_sender,
                })
                .map_err(|_| {
                    "Keep PC awake is unavailable because its worker stopped.".to_owned()
                })?;
            let result = reply_receiver
                .recv()
                .map_err(|_| "Keep PC awake did not receive a response from Windows.".to_owned())?;
            let mut enabled = self
                .enabled
                .lock()
                .map_err(|_| "Keep PC awake state is unavailable.".to_owned())?;
            *enabled = applied_enabled_state(*enabled, requested, result.is_ok());
            result
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = requested;
            Err("Keep PC awake is available only on Windows.".to_owned())
        }
    }
}

#[cfg(target_os = "windows")]
fn set_execution_state(enabled: bool) -> Result<(), String> {
    use windows::Win32::{
        Foundation::{GetLastError, SetLastError, ERROR_SUCCESS},
        System::Power::SetThreadExecutionState,
    };

    let flags = execution_state_flags(enabled);
    unsafe {
        SetLastError(ERROR_SUCCESS);
        if SetThreadExecutionState(flags).0 == 0 {
            let error = GetLastError();
            if error != ERROR_SUCCESS {
                return Err(format!(
                    "Windows could not update Keep PC awake (error {}).",
                    error.0
                ));
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn execution_state_flags(enabled: bool) -> windows::Win32::System::Power::EXECUTION_STATE {
    use windows::Win32::System::Power::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED};

    if enabled {
        ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
    } else {
        ES_CONTINUOUS
    }
}

fn applied_enabled_state(current: bool, requested: bool, succeeded: bool) -> bool {
    if succeeded {
        requested
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn enabled_request_keeps_system_and_display_awake() {
        use windows::Win32::System::Power::{
            ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
        };

        let flags = execution_state_flags(true);
        assert!(flags.contains(ES_CONTINUOUS));
        assert!(flags.contains(ES_SYSTEM_REQUIRED));
        assert!(flags.contains(ES_DISPLAY_REQUIRED));
        assert_eq!(execution_state_flags(false), ES_CONTINUOUS);
    }

    #[test]
    fn failed_request_keeps_the_previous_menu_state() {
        assert!(!applied_enabled_state(false, true, false));
        assert!(applied_enabled_state(true, false, false));
        assert!(applied_enabled_state(false, true, true));
        assert!(!applied_enabled_state(true, false, true));
    }
}
