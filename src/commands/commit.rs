use std::fs;
use sha1::{Sha1, Digest};
use std::io::Read;
use std::path::Path;

pub fn commit() -> Result<(), String> {
    commit_at(Path::new("."))
}

pub(crate) fn commit_at(root: &Path) -> Result<(), String> {
    println!("Committing changes...");

    let ferrit_path = root.join(".ferrit");
    
    let index_path = ferrit_path.join("index");
    let mut index = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&index_path)
            .map_err(|e| format!("Not able to read the index file: {e}"))?;


    let mut index_contents = String::new();
    index.read_to_string(&mut index_contents).map_err(|e| format!("Not able to read the index file: {e}"))?;

    let commit_hash: String = Sha1::digest(index_contents.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let commit_path = ferrit_path.join("Commits").join(&commit_hash);

    fs::create_dir(&commit_path)
        .map_err(|e| format!("Failed to create Commits directory: {e}"))?;
    fs::write(commit_path.join("index"), index_contents)
        .map_err(|e| format!("Failed to write commit index: {e}"))?;    

    index.set_len(0).map_err(|e| format!("Not able to clear the current stage: {e}"))?;

    fs::write(ferrit_path.join("HEAD"), &commit_hash)
        .map_err(|e| format!("Failed to update HEAD: {e}"))?;
    

    Ok(())

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{add, init};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn clears_index_after_commit() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("README.md"), "hello").expect("Failed to create test file");
        add::add_at(&root, "README.md").expect("Failed to add test file");

        commit_at(&root).expect("Failed to commit");

        let index = fs::read_to_string(root.join(".ferrit").join("index")).unwrap();
        assert!(index.is_empty());

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
