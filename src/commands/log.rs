use std::fs;
use std::path::Path;

pub fn log() -> Result<(), String> {
    let output = log_at(Path::new("."))?;
    print!("{output}");
    Ok(())
}

pub(crate) fn log_at(root: &Path) -> Result<String, String> {
    let ferrit_path = root.join(".ferrit");

    if !ferrit_path.exists() {
        return Err("Repository not initialized".to_string());
    }

    let log = fs::read_to_string(ferrit_path.join("log"))
        .map_err(|e| format!("Failed to read log file: {e}"))?;

    if log.trim().is_empty() {
        return Ok("No commits yet\n".to_string());
    }

    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{add, commit, init};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn prints_log_file_contents() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("README.md"), "hello").expect("Failed to write README.md");
        add::add_at(&root, "README.md").expect("Failed to add README.md");
        commit::commit_at(&root, "initial commit").expect("Failed to commit");

        let log = fs::read_to_string(root.join(".ferrit").join("log")).unwrap();
        let output = log_at(&root).expect("Failed to read log");

        assert_eq!(output, log);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_when_there_are_no_commits() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");

        let output = log_at(&root).expect("Failed to read log");

        assert_eq!(output, "No commits yet\n");

        fs::remove_dir_all(root).unwrap();
    }

    fn test_root() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let root = std::env::temp_dir().join(format!("ferrit-test-{timestamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
