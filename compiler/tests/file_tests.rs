use camino::Utf8PathBuf;
use compiler::file::GlxFile;
use compiler::{
    PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH_PATH, PARSER_FILE_DESTRUCTURING_PATH, setup_assets,
};

const EXPECTED_SCRIPT: &str = r#"---
    let awesome = io.println("Gleam is awesome!")
---"#;

const EXPECTED_SCRIPT2: &str = r#"---
    // --- this is an edge case that is really annoying, but since humans are stupid, we have to
    // handle it
---"#;

// Inits assets by adding `parser_destructuring_test.glx` to assets
fn init() -> Result<(), Box<dyn std::error::Error>> {
    use compiler::{
        PARSER_FILE_DESTRUCTURING, PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH,
        PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH_PATH, PARSER_FILE_DESTRUCTURING_PATH,
    };
    setup_assets(&[
        (PARSER_FILE_DESTRUCTURING_PATH, PARSER_FILE_DESTRUCTURING),
        (
            PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH_PATH,
            PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH,
        ),
    ])
}

/// We destructure a file by calling the function located in [`compiler`] module.
/// Then we check the [`GlxFile`] struct and verify it match our expectations
#[test]
fn test_file_destrusturing() {
    init().unwrap();
    let path = Utf8PathBuf::from(PARSER_FILE_DESTRUCTURING_PATH);
    let parsed = GlxFile::parse_file(path)
        .unwrap_or_else(|_| panic!("Could not parse file. Debug compiler/src/file.rs"));
    let script = parsed.script_content.unwrap();
    dbg!(&script);
    assert_eq!(script, EXPECTED_SCRIPT);

    let path2 = Utf8PathBuf::from(PARSER_FILE_DESTRUCTURING_MULTIPLE_3DASH_PATH);
    let parsed2 = GlxFile::parse_file(path2)
        .unwrap_or_else(|_| panic!("Could not parse file. If above test passed, it means that the logic of the end_delimiter is broken. Debug compiler/file.rs"));
    let script2 = parsed2.script_content.unwrap();
    dbg!(&script2);
    assert_eq!(script2, EXPECTED_SCRIPT2);
}
