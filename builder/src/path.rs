use crate::utils::read_dir;
use camino::Utf8PathBuf;
use commons::error::GlxError;

/// Returns [`ConfigError`] or [`IoError`]
pub fn search_toml(path_str: &str, glx_or_gleam: String) -> Result<Utf8PathBuf, GlxError> {
    // Assume that `path_str` is cwd of current project
    let cwd = Utf8PathBuf::from(path_str);
    assert!(cwd.exists());
    assert!(cwd.is_dir());
    assert!(glx_or_gleam == "glx" || glx_or_gleam == "gleam");

    let entries = read_dir(cwd.clone())?;
    if !entries.contains(&glx_or_gleam) {
        return Err(GlxError::ConfigError {
            what: "Missing gleam.toml file".to_string(),
            msg: "If you have a gleam.toml file. Move it at root or specify the path in glx.toml"
                .to_string(),
        });
    }

    Ok(Utf8PathBuf::from(format!("{}/{}", &cwd, glx_or_gleam)))
}
