//! [`GLX`] compiler
//! It acts on 2 parts:
//! 1. compile script to javascript (using gleam)
//! 2. compile modded HTML

use camino::Utf8PathBuf;

use commons::error::GlxError;

pub mod ast;
pub mod file;

/// Entry point of [`GLX`] compiler. Make sure to call it with the root of the user cwd' project
/// It returns the buildt AST for the HTML and the script content. We CANT build the project
/// without all the files, and since we accept a SINGLE file, we have to handle it in cli. We may
/// create an other crate for that to avoid confusing codebase with cli. I want the CLI to just
/// call the compiler by this entry point.
pub fn start(_filepath: Utf8PathBuf) -> Result<(), GlxError> {
    Ok(())
}

//////////////////
// Tests utilities

pub fn setup_assets(assets: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    let mkdir = std::fs::exists(ASSETS_PATH)?;

    if !mkdir {
        std::fs::create_dir(ASSETS_PATH)?;
    }

    for (path, content) in assets {
        // Ensure parent directory exists for the asset
        if let Some(parent) = std::path::Path::new(path).parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
    }

    Ok(())
}

/////////////////
// Tests onstants

const ASSETS_PATH: &str = "test_assets";
pub const PARSER_FILE_DESTRUCTURING_PATH: &str = "test_assets/parser_destructuring_test.glx";
pub const PARSER_FILE_DESTRUCTURING: &str = r#"---
    let awesome = io.println("Gleam is awesome!")
---

I love God
"#;
pub const PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH_PATH: &str =
    "tests/parser_destructuring_multiple_triple_dashes_tests.glx";
pub const PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH: &str = r#"---
    // --- this is an edge case that is really annoying, but since humans are stupid, we have to
    // handle it
---

God is Good"#;
