use std::fs;
use std::path::Path;

use super::temp_path;
use sha1::{Digest, Sha1};

/// Stages a file from the current working directory.
pub fn add(file: &str) -> Result<(), String> {
    add_at(Path::new("."), file)
}

/// Stores the file contents as an object and records `path hash` in the index.
///
/// If the file is already staged with a different hash, the index entry is
/// replaced so the next commit uses the latest staged contents.
pub(crate) fn add_at(root: &Path, file: &str) -> Result<(), String> {
    let ferrit_dir = root.join(".ferrit");

    if !ferrit_dir.exists() {
        return Err("Repository not initialized".to_string());
    }

    let file_path = root.join(file);

    if !file_path.exists() {
        return Err(format!("File {file} does not exist"));
    }

    let index_path = ferrit_dir.join("index");
    let index_contents = fs::read_to_string(&index_path)
        .map_err(|e| format!("Not able to read the index file: {e}"))?;
    let file_contents =
        fs::read(&file_path).map_err(|e| format!("Not able to read the file: {e}"))?;

    let hash: String = Sha1::digest(&file_contents)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let index_entry = format!("{file} {hash}");
    let mut new_index_contents = String::new();
    let mut file_changed = false;

    for line in index_contents.lines() {
        let mut parts = line.trim().split_whitespace();
        let filename = parts.next().unwrap_or("");
        let filehash = parts.next().unwrap_or("");

        if filename == file {
            if filehash == hash {
                return Err(format!("File {file} is already being tracked"));
            }

            new_index_contents.push_str(&index_entry);
            file_changed = true;
        } else {
            new_index_contents.push_str(line);
        }

        new_index_contents.push('\n');
    }

    if !file_changed {
        new_index_contents.push_str(&index_entry);
        new_index_contents.push('\n');
    }

    let new_file_path = ferrit_dir.join("objects").join(&hash);
    if !new_file_path.exists() {
        fs::write(&new_file_path, &file_contents)
            .map_err(|e| format!("Not able to write the new file to objects directory: {e}"))?;
    }

    let temp_index_path = temp_path(&ferrit_dir, "index")?;
    fs::write(&temp_index_path, new_index_contents)
        .map_err(|e| format!("Not able to update the temporary index file: {e}"))?;
    fs::rename(&temp_index_path, &index_path)
        .map_err(|e| format!("Not able to replace the index file: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn tracks_new_file() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::File::create(root.join("main.py")).expect("Failed to create test file");
        add_at(&root, "main.py")
            .map_err(|e| format!("Failed to add file for tracking: {e}"))
            .unwrap();

        fs::remove_dir_all(root).expect("Failed to clean test directory");
    }

    #[test]
    fn tracks_changed_file() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        let mut file = fs::File::create(root.join("main.py")).expect("Failed to create test file");
        file.write_all(b"print('Hello, World!')")
            .expect("Failed to write to test file");
        add_at(&root, "main.py")
            .map_err(|e| format!("Failed to add file for tracking: {e}"))
            .unwrap();

        file.write_all(b"print('2+2 is 4')")
            .expect("Failed to write to test file");
        add_at(&root, "main.py")
            .map_err(|e| format!("Failed to track modified file: {e}"))
            .unwrap();

        fs::remove_dir_all(root).expect("Failed to clean test directory");
    }

    #[test]
    fn returns_error_when_file_is_already_tracked() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::File::create(root.join("main.py")).expect("Failed to create test file");
        add_at(&root, "main.py").expect("Failed to add file for tracking");

        let result = add_at(&root, "main.py");

        fs::remove_dir_all(root).expect("Failed to clean test directory");

        assert!(result.is_err());
    }

    fn test_root() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let root = std::env::temp_dir().join(format!("ferrit-test-{timestamp}",));

        fs::create_dir_all(&root).expect("Failed to create test directory");
        root
    }
}
