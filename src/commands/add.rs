use std::fs;
use std::io::{Write, Read};
use std::path::Path;

use sha1::{Sha1, Digest};


pub fn add(file: &str) -> Result<(), String> {
    add_at(Path::new("."), file)
}

pub(crate) fn add_at(root: &Path, file: &str) -> Result<(), String> {
    let ferrit_dir = root.join(".ferrit");
    
    // Check if repository is initialized
    if !ferrit_dir.exists() {
        return Err("Repository not initialized".to_string());
    }
    
    let file_path = root.join(file);
    
    // check if file exists
    if !file_path.exists() {
        return Err(format!("File {file} does not exist"));
    }
    
    let index_path = ferrit_dir.join("index");

    let mut index = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(&index_path)
        .map_err(|e| format!("Not able to read the index file: {e}"))?;


    let mut index_contents = String::new();
    index.read_to_string(&mut index_contents).map_err(|e| format!("Not able to read the index file: {e}"))?;

    let file_contents: String = fs::read_to_string(&file_path)
        .map_err(|e| format!("Not able to read the file: {e}"))?;

    // compute the hash of the file contents 
    let hash: String = Sha1::digest(file_contents.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let result = file.to_string() + " " + &hash;
    
    let mut file_changed = false;
    let mut change_idx: usize = 0;

    // check if it is already being tracked
    for (idx, line) in index_contents.lines().enumerate() {
        let mut line = line.trim().split_whitespace();
        let filename = line.next().unwrap_or("");
        let filehash = line.next().unwrap_or("");


        if filename == file {
            if filehash == hash {
                return Err(format!("File {file} is already being tracked"));
            } else {

                // copy the new file to the objects directory       
                let new_file_path = ferrit_dir.join("Objects").join(&hash);
                fs::copy(&file_path, &new_file_path).map_err(|e| format!("Not able to copy the new file to objects directory: {e}"))?;                 
                
                file_changed = true; 
                change_idx = idx;
            }
        }
    }
    
    if file_changed == true{
        // means file is being tracked but has changed, so we need to update the index file
        let mut new_index_contents = String::new();
        for (idx, line) in index_contents.lines().enumerate() {
            if idx == change_idx as usize {
                new_index_contents.push_str(&result);
                new_index_contents.push('\n');
            } else {
                new_index_contents.push_str(line);
                new_index_contents.push('\n');              
            }
        }
        fs::write(&index_path, new_index_contents).map_err(|e| format!("Not able to update the index file: {e}"))?;
    } else {
        // means file is not being tracked, so we need to copy the new file to the objects directory
        let new_file_path = ferrit_dir.join("Objects").join(&hash);
        fs::copy(&file_path, &new_file_path).map_err(|e| format!("Not able to copy the new file to objects directory: {e}"))?;          
        index.write_all((result+"\n").as_bytes()).map_err(|e| format!("Not able to update the index file: {e}"))?;         
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init;
    use std::time::{SystemTime,UNIX_EPOCH};

    #[test]
    fn tracks_new_file() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::File::create(root.join("main.py")).expect("Failed to create test file");
        add_at(&root, "main.py").map_err(|e| format!("Failed to add file for tracking: {e}")).unwrap();

        fs::remove_dir_all(root).expect("Failed to clean test directory");
    }

    #[test]
    fn tracks_changed_file(){
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        let mut file = fs::File::create(root.join("main.py")).expect("Failed to create test file");
        file.write_all(b"print('Hello, World!')").expect("Failed to write to test file");
        add_at(&root, "main.py").map_err(|e| format!("Failed to add file for tracking: {e}")).unwrap();

        file.write_all(b"print('2+2 is 4')").expect("Failed to write to test file");
        add_at(&root, "main.py").map_err(|e| format!("Failed to track modified file: {e}")).unwrap();

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

        let root = std::env::temp_dir().join(format!(
            "ferrit-test-{timestamp}",
        ));

        fs::create_dir_all(&root).expect("Failed to create test directory");
        root
    }
}
