#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupStatus {
    Enabled,
    Disabled,
    DisabledByUser,
    DisabledByPolicy,
    Unavailable,
}

const STARTUP_TASK_ID: &str = "ScatteredThoughtsStartup";

#[cfg(target_os = "windows")]
fn status_from_windows(state: windows::ApplicationModel::StartupTaskState) -> StartupStatus {
    use windows::ApplicationModel::StartupTaskState;

    match state {
        StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy => StartupStatus::Enabled,
        StartupTaskState::Disabled => StartupStatus::Disabled,
        StartupTaskState::DisabledByUser => StartupStatus::DisabledByUser,
        StartupTaskState::DisabledByPolicy => StartupStatus::DisabledByPolicy,
        _ => StartupStatus::Unavailable,
    }
}

#[cfg(target_os = "windows")]
fn task() -> Result<windows::ApplicationModel::StartupTask, String> {
    use windows::{
        core::HSTRING,
        ApplicationModel::{Package, StartupTask},
    };

    Package::Current()
        .map_err(|_| "Scattered Thoughts is not running as a Store package.".to_owned())?;
    StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
        .and_then(|operation| operation.get())
        .map_err(|error| format!("Could not read the Windows startup setting: {error}"))
}

pub fn status() -> StartupStatus {
    #[cfg(target_os = "windows")]
    {
        task()
            .and_then(|task| task.State().map_err(|error| error.to_string()))
            .map(status_from_windows)
            .unwrap_or(StartupStatus::Unavailable)
    }

    #[cfg(not(target_os = "windows"))]
    {
        StartupStatus::Unavailable
    }
}

pub fn toggle() -> Result<StartupStatus, String> {
    match status() {
        StartupStatus::Enabled => {
            #[cfg(target_os = "windows")]
            {
                task()?
                    .Disable()
                    .map_err(|error| format!("Could not disable launch at sign-in: {error}"))?;
                Ok(StartupStatus::Disabled)
            }
            #[cfg(not(target_os = "windows"))]
            Err("Launch at sign-in is only available in the Microsoft Store edition.".to_owned())
        }
        StartupStatus::Disabled => {
            #[cfg(target_os = "windows")]
            {
                let new_state = task()?
                    .RequestEnableAsync()
                    .and_then(|operation| operation.get())
                    .map_err(|error| format!("Could not enable launch at sign-in: {error}"))?;
                Ok(status_from_windows(new_state))
            }
            #[cfg(not(target_os = "windows"))]
            Err("Launch at sign-in is only available in the Microsoft Store edition.".to_owned())
        }
        StartupStatus::DisabledByUser => Err(
            "Windows has disabled Scattered Thoughts at sign-in. Re-enable it in Windows Startup Apps or Task Manager."
                .to_owned(),
        ),
        StartupStatus::DisabledByPolicy => Err(
            "Windows policy has disabled Scattered Thoughts at sign-in.".to_owned(),
        ),
        StartupStatus::Unavailable => {
            Err("Launch at sign-in is available in the Microsoft Store edition of Scattered Thoughts.".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_status_is_not_enabled() {
        assert_ne!(StartupStatus::Unavailable, StartupStatus::Enabled);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn maps_all_windows_startup_states() {
        use windows::ApplicationModel::StartupTaskState;

        assert_eq!(
            status_from_windows(StartupTaskState::Enabled),
            StartupStatus::Enabled
        );
        assert_eq!(
            status_from_windows(StartupTaskState::Disabled),
            StartupStatus::Disabled
        );
        assert_eq!(
            status_from_windows(StartupTaskState::DisabledByUser),
            StartupStatus::DisabledByUser
        );
        assert_eq!(
            status_from_windows(StartupTaskState::DisabledByPolicy),
            StartupStatus::DisabledByPolicy
        );
    }
}
