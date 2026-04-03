//! Module that contains utilities to interact with gleam so we can compile, build and check gleam
//! code
#![allow(unused)]

use crate::config_toml::{self, TomlConfigFile};
use crate::path::search_toml;
use camino::Utf8PathBuf;
use commons::error::GlxError;
use std::{env::join_paths, fs, process::Command};
use toml::Table;

/// Call it with [`GlxFile`].script_content
pub fn compile_script(config_path: Utf8PathBuf, script: &str) -> Result<(), GlxError> {
    // TODO: Retrieve the generated content in build/dev/javascript/
    // Update the function to take as argument the name of the js file to retrieve (based on config)
    let cwd_bindings = config_path.to_string();
    let cwd = cwd_bindings
        .strip_suffix("glx.toml")
        .ok_or_else(|| panic!("Invalid workspace: Failed to strip_suffix *glx.toml*"))?;

    let gleam_cfg_path = search_toml(cwd, "gleam".to_string())?;
    let parsed_gleam_cfg_file = gleam_cfg_path.as_str().parse::<Table>()
        .map_err(|err| GlxError::ConfigError {
            what: "Could not parse gleam.toml into toml::Table".to_string(),
            msg: format!("Make sure that you're config file at root is {gleam_cfg_path} and that gleam compiles and validate the file") 
        })?;
    // TODO: better error handling
    let project_name =
        parsed_gleam_cfg_file["name"]
            .as_str()
            .ok_or_else(|| GlxError::ConfigError {
                what: format!(
                    "Could not get *name* key from {gleam_cfg_path}, parsed into toml::Table"
                ),
                msg: "Could not determine the error".to_string(),
            })?;

    // `project_name` is the name of the project, so the name of the build script of a project
    // We have to handle multiples cases, and name our gen JS file. Or do preprocessing on files
    // and make a _glx/ folder where we create a tmp workspace that we can put in build

    Ok(())
}
