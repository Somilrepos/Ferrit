use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn checkout(commit_id: &str) -> Result<(), String> {
    checkout_at(Path::new("."), commit_id)
}

pub(crate) fn checkout_at(root: &Path, commit_id: &str) -> Result<(), String> {
    let ferrit_path = root.join(".ferrit");
    let commit_path = ferrit_path.join("commits").join(commit_id);

    if !commit_path.exists() {
        return Err(format!("Commit with id {} does not exist", commit_id));
    }

    let target_files = read_commit_index(&commit_path)?;
    let current_files = read_current_commit_index(&ferrit_path)?;

    for filename in current_files.keys() {
        if !target_files.contains_key(filename) {
            let file_path = root.join(filename);
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to remove file {} from working directory: {e}", filename))?;
            }
        }
    }

    for (filename, filehash) in target_files {
        let object_path = ferrit_path.join("objects").join(&filehash);
        if !object_path.exists() {
            return Err(format!("Object with hash {} does not exist", filehash));
        }
        fs::copy(&object_path, root.join(&filename))
            .map_err(|e| format!("Failed to copy file {} from objects to working directory: {e}", filename))?;
    }

    fs::write(ferrit_path.join("HEAD"), &commit_id)
        .map_err(|e| format!("Failed to update HEAD: {e}"))?;

    Ok(())
}

fn read_current_commit_index(ferrit_path: &Path) -> Result<HashMap<String, String>, String> {
    let head_path = ferrit_path.join("HEAD");
    let current_commit_id = fs::read_to_string(&head_path)
        .map_err(|e| format!("Failed to read HEAD: {e}"))?;
    let current_commit_id = current_commit_id.trim();

    if current_commit_id.is_empty() {
        return Ok(HashMap::new());
    }

    let current_commit_path = ferrit_path.join("commits").join(current_commit_id);
    read_commit_index(&current_commit_path)
}

fn read_commit_index(commit_path: &Path) -> Result<HashMap<String, String>, String> {
    let index_path = commit_path.join("index");
    let index_contents = fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read commit index: {e}"))?;
    let mut files = HashMap::new();

    for line in index_contents.lines() {
        let mut parts = line.trim().split_whitespace();
        let filename = parts
            .next()
            .ok_or_else(|| "Commit index contains an invalid entry".to_string())?;
        let filehash = parts
            .next()
            .ok_or_else(|| "Commit index contains an invalid entry".to_string())?;

        files.insert(filename.to_string(), filehash.to_string());
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{add, commit, init};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn removes_files_absent_from_target_commit() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");

        fs::write(root.join("a.txt"), "a v1").expect("Failed to write a.txt");
        fs::write(root.join("b.txt"), "b v1").expect("Failed to write b.txt");
        add::add_at(&root, "a.txt").expect("Failed to add a.txt");
        add::add_at(&root, "b.txt").expect("Failed to add b.txt");
        commit::commit_at(&root, "first commit").expect("Failed to create first commit");
        let first_commit = fs::read_to_string(root.join(".ferrit").join("HEAD")).unwrap();

        fs::remove_file(root.join("b.txt")).expect("Failed to remove b.txt");
        fs::write(root.join("a.txt"), "a v2").expect("Failed to update a.txt");
        add::add_at(&root, "a.txt").expect("Failed to add updated a.txt");
        commit::commit_at(&root, "second commit").expect("Failed to create second commit");
        let second_commit = fs::read_to_string(root.join(".ferrit").join("HEAD")).unwrap();

        checkout_at(&root, first_commit.trim()).expect("Failed to checkout first commit");
        assert!(root.join("b.txt").exists());

        checkout_at(&root, second_commit.trim()).expect("Failed to checkout second commit");
        assert!(root.join("a.txt").exists());
        assert!(!root.join("b.txt").exists());

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
