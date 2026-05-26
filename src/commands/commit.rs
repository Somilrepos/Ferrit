use std::fs;
use sha1::{Sha1, Digest};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates a commit in the current working directory using the staged index.
pub fn commit(message: &str) -> Result<(), String> {
    commit_at(Path::new("."), message)
}

/// Writes a commit snapshot, metadata, and log entry under `root/.ferrit`.
///
/// The index is treated as the staging area. Each commit records the previous
/// `HEAD` as its parent, then clears the index after the commit is stored.
pub(crate) fn commit_at(root: &Path, message: &str) -> Result<(), String> {
    println!("Committing changes...");

    if message.trim().is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }

    let ferrit_path = root.join(".ferrit");
    
    let index_path = ferrit_path.join("index");
    let mut index = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&index_path)
            .map_err(|e| format!("Not able to read the index file: {e}"))?;


    let mut index_contents = String::new();
    index.read_to_string(&mut index_contents).map_err(|e| format!("Not able to read the index file: {e}"))?;

    if index_contents.trim().is_empty() {
        return Err("No changes to commit".to_string());
    }

    let parent_commit = fs::read_to_string(ferrit_path.join("HEAD"))
        .map_err(|e| format!("Failed to read HEAD: {e}"))?;
    // The first commit has no parent, so store an explicit sentinel.
    let parent_commit = match parent_commit.trim() {
        "" => "None".to_string(),
        commit_id => commit_id.to_string(),
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to read system time: {e}"))?
        .as_secs();
    let formatted_time = format_unix_timestamp(timestamp);
    let commit_input = format!("parent:{parent_commit}\ntime:{timestamp}\nmessage:{message}\n{index_contents}");
    let commit_hash: String = Sha1::digest(commit_input.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let commit_path = ferrit_path.join("commits").join(&commit_hash);

    fs::create_dir(&commit_path)
        .map_err(|e| format!("Failed to create commits directory: {e}"))?;
    fs::write(commit_path.join("index"), index_contents)
        .map_err(|e| format!("Failed to write commit index: {e}"))?;    
    fs::write(
        commit_path.join("metadata"),
        format!("commit {commit_hash}\nparent {parent_commit}\ntime {formatted_time}\nmessage {message}\n"),
    )
    .map_err(|e| format!("Failed to write commit metadata: {e}"))?;

    index.set_len(0).map_err(|e| format!("Not able to clear the current stage: {e}"))?;

    fs::write(ferrit_path.join("HEAD"), &commit_hash)
        .map_err(|e| format!("Failed to update HEAD: {e}"))?;
    append_log(&ferrit_path, &commit_hash, &parent_commit, &formatted_time, message)?;
    

    Ok(())

}

/// Appends the commit metadata to the human-readable repository log.
fn append_log(
    ferrit_path: &Path,
    commit_hash: &str,
    parent_commit: &str,
    formatted_time: &str,
    message: &str,
) -> Result<(), String> {
    let mut log = fs::OpenOptions::new()
        .append(true)
        .open(ferrit_path.join("log"))
        .map_err(|e| format!("Failed to open log file: {e}"))?;

    writeln!(log, "commit {commit_hash}").map_err(|e| format!("Failed to write log file: {e}"))?;
    writeln!(log, "parent {parent_commit}").map_err(|e| format!("Failed to write log file: {e}"))?;
    writeln!(log, "time {formatted_time}").map_err(|e| format!("Failed to write log file: {e}"))?;
    writeln!(log, "message {message}").map_err(|e| format!("Failed to write log file: {e}"))?;
    writeln!(log).map_err(|e| format!("Failed to write log file: {e}"))?;

    Ok(())
}

/// Formats a Unix timestamp as a UTC wall-clock string without external crates.
fn format_unix_timestamp(timestamp: u64) -> String {
    let seconds_per_day = 86_400;
    let days = (timestamp / seconds_per_day) as i64;
    let seconds_of_day = timestamp % seconds_per_day;
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let (year, month, day) = civil_from_days(days);

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

/// Converts days since the Unix epoch to a Gregorian calendar date.
fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u64, u64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year, month as u64, day as u64)
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

        commit_at(&root, "initial commit").expect("Failed to commit");

        let index = fs::read_to_string(root.join(".ferrit").join("index")).unwrap();
        assert!(index.is_empty());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writes_metadata_and_updates_log() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("README.md"), "hello").expect("Failed to create test file");
        add::add_at(&root, "README.md").expect("Failed to add test file");

        commit_at(&root, "initial commit").expect("Failed to commit");

        let head = fs::read_to_string(root.join(".ferrit").join("HEAD")).unwrap();
        let commit_path = root.join(".ferrit").join("commits").join(head.trim());
        let metadata = fs::read_to_string(commit_path.join("metadata")).unwrap();
        let log = fs::read_to_string(root.join(".ferrit").join("log")).unwrap();

        assert!(metadata.contains(&format!("commit {}", head.trim())));
        assert!(metadata.contains("parent None"));
        assert!(metadata.contains("time "));
        assert!(metadata.contains("message initial commit"));
        assert!(log.contains(&metadata));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn records_parent_commit_after_first_commit() {
        let root = test_root();

        init::init_at(&root).expect("Repository initialization failed");
        fs::write(root.join("README.md"), "hello").expect("Failed to create test file");
        add::add_at(&root, "README.md").expect("Failed to add test file");
        commit_at(&root, "first commit").expect("Failed to commit");
        let first_commit = fs::read_to_string(root.join(".ferrit").join("HEAD")).unwrap();

        fs::write(root.join("README.md"), "hello again").expect("Failed to update test file");
        add::add_at(&root, "README.md").expect("Failed to add updated test file");
        commit_at(&root, "second commit").expect("Failed to commit");
        let second_commit = fs::read_to_string(root.join(".ferrit").join("HEAD")).unwrap();

        let metadata = fs::read_to_string(
            root.join(".ferrit")
                .join("commits")
                .join(second_commit.trim())
                .join("metadata"),
        )
        .unwrap();

        assert!(metadata.contains(&format!("parent {}", first_commit.trim())));
        assert!(metadata.contains("message second commit"));

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
