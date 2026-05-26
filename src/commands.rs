pub mod add;
pub mod checkout;
pub mod commit;
pub mod init;
pub mod log;
pub mod status;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Builds a unique temporary path under `parent` using `prefix`.
pub(crate) fn temp_path(parent: &Path, prefix: &str) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to read system time: {e}"))?
        .as_nanos();

    Ok(parent.join(format!("{prefix}-{timestamp}-{}", std::process::id())))
}
