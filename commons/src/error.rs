#![allow(clippy::enum_variant_names)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlxError {
    #[error("{0}")]
    ParseError(ParseError),
    #[error("{0}")]
    CompileError(CompileError),
    #[error("{0}")]
    IoError(IoError),
    #[error("Config Error: {what} — {msg}")]
    ConfigError { what: String, msg: String },
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{msg}")]
    InvalidFile { msg: String },
    #[error("toml: {path}\n{msg}")]
    TomlError { path: String, msg: String },
    #[error("{0}")]
    IoError(#[from] IoError),
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("{msg}")]
    InvalidScript { msg: String },
}

#[derive(Debug, Error)]
pub enum IoError {
    #[error("File IO: {source}")]
    FileError {
        #[from]
        source: std::io::Error,
    },

    #[error("Failed on {path}: {source}")]
    FileErrorWithPath {
        path: String,
        source: std::io::Error,
    },

    #[error("Dir IO: {source}")]
    DirError { source: std::io::Error },

    #[error("Failed on {path}: {source}")]
    DirErrorWithPath {
        path: String,
        source: std::io::Error,
    },
}

impl IoError {
    pub fn with_path(path: impl Into<String>, source: std::io::Error) -> Self {
        IoError::FileErrorWithPath {
            path: path.into(),
            source,
        }
    }

    pub fn dir_with_path(path: impl Into<String>, source: std::io::Error) -> Self {
        IoError::FileErrorWithPath {
            path: path.into(),
            source,
        }
    }
}
