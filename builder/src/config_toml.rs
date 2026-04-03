//! [`glx.toml`] is not a replacment of [`gleam.toml`], we just specify how glx behaves, without
//! any hex, or license fields
#![allow(
    unused,
    clippy::new_ret_no_self,
    clippy::wrong_self_convention,
    clippy::to_string_trait_impl
)]

use std::{fs, io::Write};

use camino::Utf8PathBuf;

use commons::error::{IoError, ParseError};

#[derive(Debug, serde::Deserialize)]
pub struct TomlConfigFile {
    /// Name is assigned to the builded mjs file from gleam
    name: String,
    runtime: String,
    manifest_path: Option<String>,

    deps: Vec<Dependency>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Dependency {
    pkg: String,
    version: String,
}

pub enum TomlField {
    Name(String),
    Runtime(String),
    ManifestPath(Option<String>),
    Deps(Vec<Dependency>),
}

impl ToString for Dependency {
    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("{pkg}: {version}");
        out
    }
}

impl TomlConfigFile {
    pub fn new(&self, path: &Utf8PathBuf) -> Result<Utf8PathBuf, IoError> {
        assert!(path.ends_with("glx.toml"));
        assert!(path.exists());
        let filename = match path.file_name() {
            Some(p) => p,
            _ => panic!("Could not get filename"),
        };
        let content = self.build_default();
        let mut file =
            fs::File::create(path).map_err(|err| IoError::with_path(path.clone(), err))?;

        file.write_all(content.as_bytes())
            .map_err(|err| IoError::with_path(path.clone(), err))?;

        file.sync_all()
            .map_err(|err| IoError::with_path(path.clone(), err))?;

        Ok(path.clone())
    }

    // Returns a ready to write file content, no parsing needed
    fn build_default(&self) -> String {
        let mut out = String::new();
        out.push_str(format!("name: {}\n", self.name).as_str());
        out.push_str(format!("run: {}\n", self.runtime).as_str());

        out.push_str("\n[dependencies]\n");
        for dep in &self.deps {
            out.push_str(format!("{}: {}\n", dep.pkg, dep.version).as_str());
        }

        out
    }

    pub fn parse(path: &Utf8PathBuf) -> Result<Self, ParseError> {
        assert!(path.ends_with("glx.toml"));
        assert!(path.exists());
        let content =
            fs::read_to_string(path).map_err(|err| IoError::with_path(path.to_string(), err))?;
        let parsed: TomlConfigFile =
            toml::from_str(&content).map_err(|err| ParseError::TomlError {
                path: path.to_string(),
                msg: err.to_string(),
            })?;

        Ok(parsed)
    }

    pub fn get(&self, key: &str, path: Utf8PathBuf) -> Option<TomlField> {
        match key {
            "name" => Some(TomlField::Name(self.name.clone())),
            "runtime" => Some(TomlField::Runtime(self.runtime.clone())),
            "deps" => Some(TomlField::Deps(self.deps.clone())),
            "manifest_path" => Some(TomlField::ManifestPath(self.manifest_path.clone())),
            _ => None,
        }
    }
}
