use sha1::{Digest, Sha1};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

pub fn status() -> Result<(), String> {
    let output = status_at(Path::new("."))?;
    print!("{output}");
    Ok(())
}

pub(crate) fn status_at(root: &Path) -> Result<String, String> {
    let ferrit_path = root.join(".ferrit");
    if !ferrit_path.exists() {
        return Err("Repository not initialized".to_string());
    }

    let committed = read_head_index(root)?;
    let staged = read_index(&ferrit_path.join("index"))?;
    let working = read_working_files(root)?;

    let mut staged_changes = Vec::new();
    let mut unstaged_changes = Vec::new();
    let mut untracked_files = Vec::new();

    let mut staged_paths = BTreeSet::new();
    for path in committed.keys() {
        staged_paths.insert(path.clone());
    }
    for path in staged.keys() {
        staged_paths.insert(path.clone());
    }

    for path in staged_paths {
        match (committed.get(&path), staged.get(&path)) {
            (None, Some(_)) => staged_changes.push(format!("added: {path}")),
            (Some(old_hash), Some(new_hash)) if old_hash != new_hash => {
                staged_changes.push(format!("modified: {path}"));
            }
            _ => {}
        }
    }

    for (path, working_hash) in &working {
        if staged.contains_key(path) {
            if staged.get(path) != Some(working_hash) {
                unstaged_changes.push(format!("modified: {path}"));
            }
        } else if committed.contains_key(path) {
            if committed.get(path) != Some(working_hash) {
                unstaged_changes.push(format!("modified: {path}"));
            }
        } else {
            untracked_files.push(path.clone());
        }
    }

    for path in committed.keys() {
        if !working.contains_key(path) {
            unstaged_changes.push(format!("deleted: {path}"));
        }
    }

    staged_changes.sort();
    unstaged_changes.sort();
    untracked_files.sort();

    let mut output = String::new();

    if staged_changes.is_empty() && unstaged_changes.is_empty() && untracked_files.is_empty() {
        output.push_str("No changes\n");
        return Ok(output);
    }

    if !staged_changes.is_empty() {
        output.push_str("Staged:\n");
        for change in staged_changes {
            output.push_str(&format!("  {change}\n"));
        }
    }

    if !unstaged_changes.is_empty() {
        output.push_str("Unstaged:\n");
        for change in unstaged_changes {
            output.push_str(&format!("  {change}\n"));
        }
    }

    if !untracked_files.is_empty() {
        output.push_str("Untracked:\n");
        for file in untracked_files {
            output.push_str(&format!("  {file}\n"));
        }
    }

    Ok(output)
}

fn read_head_index(root: &Path) -> Result<HashMap<String, String>, String> {
    let ferrit_path = root.join(".ferrit");
    let head = fs::read_to_string(ferrit_path.join("HEAD"))
        .map_err(|e| format!("Failed to read HEAD: {e}"))?;
    let head = head.trim();

    if head.is_empty() {
        return Ok(HashMap::new());
    }

    read_index(&ferrit_path.join("commits").join(head).join("index"))
}

fn read_index(index_path: &Path) -> Result<HashMap<String, String>, String> {
    let contents = fs::read_to_string(index_path)
        .map_err(|e| format!("Failed to read index: {e}"))?;
    let mut entries = HashMap::new();

    for line in contents.lines() {
        let mut parts = line.trim().split_whitespace();
        let path = parts
            .next()
            .ok_or_else(|| "Index contains an invalid entry".to_string())?;
        let hash = parts
            .next()
            .ok_or_else(|| "Index contains an invalid entry".to_string())?;

        entries.insert(path.to_string(), hash.to_string());
    }

    Ok(entries)
}

fn read_working_files(root: &Path) -> Result<HashMap<String, String>, String> {
    let mut files = HashMap::new();

    for entry in fs::read_dir(root).map_err(|e| format!("Failed to read working directory: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read working directory entry: {e}"))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| "File path is not valid UTF-8".to_string())?;
        let bytes = fs::read(&path).map_err(|e| format!("Failed to read file {file_name}: {e}"))?;
        let hash: String = Sha1::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        files.insert(file_name, hash);
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{add, commit, init};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reports_untracked_file() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("notes.txt"), "hello").expect("Failed to write notes.txt");

        let output = status_at(&root).expect("Failed to read status");

        assert!(output.contains("Untracked:"));
        assert!(output.contains("notes.txt"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_staged_file() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("notes.txt"), "hello").expect("Failed to write notes.txt");
        add::add_at(&root, "notes.txt").expect("Failed to add notes.txt");

        let output = status_at(&root).expect("Failed to read status");

        assert!(output.contains("Staged:"));
        assert!(output.contains("added: notes.txt"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_no_changes_after_commit() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("notes.txt"), "hello").expect("Failed to write notes.txt");
        add::add_at(&root, "notes.txt").expect("Failed to add notes.txt");
        commit::commit_at(&root, "initial commit").expect("Failed to commit");

        let output = status_at(&root).expect("Failed to read status");

        assert_eq!(output, "No changes\n");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_modified_file_after_commit() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("notes.txt"), "hello").expect("Failed to write notes.txt");
        add::add_at(&root, "notes.txt").expect("Failed to add notes.txt");
        commit::commit_at(&root, "initial commit").expect("Failed to commit");
        fs::write(root.join("notes.txt"), "changed").expect("Failed to change notes.txt");

        let output = status_at(&root).expect("Failed to read status");

        assert!(output.contains("Unstaged:"));
        assert!(output.contains("modified: notes.txt"));

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
