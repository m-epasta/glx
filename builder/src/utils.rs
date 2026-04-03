use commons::error::{GlxError, IoError};
use std::fs;

pub fn read_dir<P: AsRef<std::path::Path> + ToString + Clone>(
    cwd: P,
) -> Result<Vec<String>, GlxError>
where
    String: From<P>,
{
    let dir_entries: Vec<String> = fs::read_dir(cwd.as_ref())
        .map_err(|err| GlxError::IoError(IoError::dir_with_path(cwd.clone(), err)))?
        .filter_map(|entry| {
            entry
                .ok()
                .map(|e| e.file_name().to_string_lossy().to_string())
        })
        .collect();

    Ok(dir_entries)
}
