use std::path::PathBuf;

pub fn select_data_directory(packaged: Option<PathBuf>, unpackaged: PathBuf) -> PathBuf {
    packaged.unwrap_or(unpackaged)
}

pub fn data_directory(unpackaged: PathBuf) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::{ApplicationModel::Package, Storage::ApplicationData};

        if Package::Current().is_ok() {
            let local_folder = ApplicationData::Current()
                .and_then(|data| data.LocalFolder())
                .and_then(|folder| folder.Path())
                .map_err(|error| format!("Could not find the Store data directory: {error}"))?;
            return Ok(select_data_directory(
                Some(PathBuf::from(local_folder.to_string())),
                unpackaged,
            ));
        }
    }

    Ok(select_data_directory(None, unpackaged))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_data_directory_is_preferred() {
        assert_eq!(
            select_data_directory(Some(PathBuf::from("store")), PathBuf::from("local")),
            PathBuf::from("store")
        );
    }

    #[test]
    fn unpackaged_data_directory_is_used_without_package_identity() {
        assert_eq!(
            select_data_directory(None, PathBuf::from("local")),
            PathBuf::from("local")
        );
    }
}
