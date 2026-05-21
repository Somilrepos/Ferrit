use std::fs;
use std::path::Path;

pub fn init() -> Result<(), String> {
    init_at(Path::new("."))
}

pub(crate) fn init_at(root: &Path) -> Result<(), String> {
    let ferrit_dir = root.join(".ferrit");

    if ferrit_dir.exists() {
        return Err("Repository already exists!".to_string());
    }

    fs::create_dir(&ferrit_dir).map_err(|e| format!("Failed to create repository: {e}"))?;
    fs::File::create(ferrit_dir.join("HEAD")).map_err(|e| format!("Failed to create HEAD file: {e}"))?;
    fs::File::create(ferrit_dir.join("index")).map_err(|e| format!("Failed to create index file: {e}"))?;
    fs::create_dir(ferrit_dir.join("Commits"))
        .map_err(|e| format!("Failed to create Commits directory: {e}"))?;
    fs::create_dir(ferrit_dir.join("Objects"))
        .map_err(|e| format!("Failed to create Objects directory: {e}"))?;

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
        assert!(root.join(".ferrit").join("Commits").exists());
        assert!(root.join(".ferrit").join("Objects").exists());

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
