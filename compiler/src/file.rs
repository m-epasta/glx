#![allow(unused)]

use camino::Utf8PathBuf;
use commons::error::ParseError;
use std::fs;

pub struct GlxFile {
    filename: Utf8PathBuf,
    /// Files can contains no script at all
    pub script_content: Option<String>,
    pub rest: String,
}

impl GlxFile {
    pub fn parse_file(filename: Utf8PathBuf) -> Result<Self, ParseError> {
        let content_owner = fs::read_to_string(&filename)
            .unwrap_or_else(|_| panic!("Could not read {:?}", filename));
        let content = content_owner.as_str();

        if content.starts_with("<>") {
            return Ok(Self::handle_no_script(filename, content));
        }

        let cursor1 = first_line(content, "---").ok_or_else(|| ParseError::InvalidFile {
            msg: format!(
                "Could not determine if given file ({}) starts with script or without",
                Utf8PathBuf::to_string(&filename)
            ),
        })?;

        let after_opening = &content[cursor1 + 3..]; // Skip past the first "---"

        let closing_pos = find_closing_delimiter(after_opening)?;
        let script_end = cursor1 + 3 + closing_pos; // +3 for the opening "---"

        let (script, rest) = content.split_at(script_end + 3); // +3 for the closing "---"

        Ok(Self {
            filename,
            script_content: Some(script.to_string()),
            rest: rest.to_string(),
        })
    }

    // Actually, we don't care if the file is in one line or not, so if it starts with *<>* we're
    // all good
    fn handle_no_script(filename: Utf8PathBuf, content: &str) -> Self {
        Self {
            filename,
            script_content: None,
            rest: content.to_string(),
        }
    }
}

/// Advance to the next line on a **NON TOKENIZED** content
/// Does not return special errors
#[inline]
pub(crate) fn advance(input: &str) -> Result<&str, String> {
    match input.find("\n") {
        Some(pos) => Ok(&input[pos + 1..]),
        None => Err("".to_string()),
    }
}

pub(crate) fn first_line(content: &str, starts_with: &str) -> Option<usize> {
    let mut current_pos = 0;

    for line in content.lines() {
        if line.trim().is_empty() {
            current_pos += line.len() + 1; // TODO: handle \r\n properly
            continue;
        }

        if line.starts_with(starts_with) {
            return Some(current_pos);
        }

        return None;
    }

    None
}

fn find_closing_delimiter(content: &str) -> Result<usize, ParseError> {
    let mut current_pos = 0;
    let mut in_html = false;

    for line in content.lines() {
        if line.trim() == "---" && !in_html {
            let offset = line.find("---").unwrap();
            return Ok(current_pos + offset);
        }

        // Simple in_html tracking
        for ch in line.chars() {
            match ch {
                '<' => in_html = true,
                '>' => in_html = false,
                _ => {}
            }
        }

        current_pos += line.len() + 1;
    }

    Err(ParseError::InvalidFile {
        msg: "Missing closing '---' delimiter".to_string(),
    })
}
