use std::fs;
use std::path::Path;

/// Initializes a ferrit repository in the current working directory.
pub fn init() -> Result<(), String> {
    init_at(Path::new("."))
}

/// Creates the repository metadata layout under `root/.ferrit`.
///
/// This helper keeps filesystem tests isolated by allowing tests to pass a
/// temporary root instead of writing to the real project directory.
pub(crate) fn init_at(root: &Path) -> Result<(), String> {
    let ferrit_dir = root.join(".ferrit");

    if ferrit_dir.exists() {
        return Err("Repository already exists!".to_string());
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_ferrit_dir = root.join(format!(".ferrit-{timestamp}-{}", std::process::id()));

    fs::create_dir(&temp_ferrit_dir).map_err(|e| format!("Failed to create repository: {e}"))?;
    fs::File::create(temp_ferrit_dir.join("HEAD")).map_err(|e| format!("Failed to create HEAD file: {e}"))?;
    fs::File::create(temp_ferrit_dir.join("index")).map_err(|e| format!("Failed to create index file: {e}"))?;
    fs::File::create(temp_ferrit_dir.join("log")).map_err(|e| format!("Failed to create log file: {e}"))?;
    fs::create_dir(temp_ferrit_dir.join("commits"))
        .map_err(|e| format!("Failed to create commits directory: {e}"))?;
    fs::create_dir(temp_ferrit_dir.join("objects"))
        .map_err(|e| format!("Failed to create objects directory: {e}"))?;

    fs::rename(temp_ferrit_dir, ferrit_dir).map_err(|e| format!("Failed to finalize repository initialization: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime,UNIX_EPOCH};

    #[test]
    fn creates_repository_metadata_directory() {
        let root = test_root();

        let result = init_at(&root);

        assert!(result.is_ok());
        assert!(root.join(".ferrit").exists());
        assert!(root.join(".ferrit").join("HEAD").exists());
        assert!(root.join(".ferrit").join("index").exists());
        assert!(root.join(".ferrit").join("log").exists());
        assert!(root.join(".ferrit").join("commits").exists());
        assert!(root.join(".ferrit").join("objects").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn returns_error_when_repository_already_exists() {
        let root = test_root();
        fs::create_dir_all(root.join(".ferrit")).unwrap();

        let result = init_at(&root);

        assert!(result.is_err());

        fs::remove_dir_all(root).unwrap();
    }

    fn test_root() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let root = std::env::temp_dir().join(format!(
            "ferrit-test-{timestamp}",
    ));

        fs::create_dir_all(&root).unwrap();
        root
    }
}
