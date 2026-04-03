//! Scanner (Lexer) for HTML/GLX syntax.

use super::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenTag,      // <
    CloseTag,     // >
    SelfClose,    // />
    CloseTagOpen, // </
    CommentOpen,  // <!--
    CommentClose, // -->
    DoctypeOpen,  // <!DOCTYPE or <!
    ExprOpen,     // {
    ExprClose,    // }
    Equals,       // =
    DoubleQuote,  // "
    SingleQuote,  // '
    Word,         // identifiers, attributes, etc.
    String,       // quoted content
    Text,         // text content
    Whitespace,   // whitespace
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub span: Span,
}

#[expect(dead_code)]
pub struct Scanner<'a> {
    pub(crate) input: &'a str,
    pub(crate) chars: Vec<char>,
    pub(crate) pos: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn match_str(&mut self, s: &str) -> bool {
        if self.pos + s.len() > self.chars.len() {
            return false;
        }
        let substr: String = self.chars[self.pos..self.pos + s.len()].iter().collect();
        if substr == s {
            for _ in 0..s.len() {
                self.advance();
            }
            true
        } else {
            false
        }
    }

    pub fn scan_token(&mut self) -> Token {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let kind = match self.peek() {
            Some('<') => {
                self.advance();
                if self.match_str("/") {
                    TokenKind::CloseTagOpen
                } else if self.match_str("!--") {
                    TokenKind::CommentOpen
                } else if self.match_str("!") {
                    TokenKind::DoctypeOpen
                } else {
                    TokenKind::OpenTag
                }
            }
            Some('>') => {
                self.advance();
                TokenKind::CloseTag
            }
            Some('/') if self.peek_next() == Some('>') => {
                self.advance();
                self.advance();
                TokenKind::SelfClose
            }
            Some('{') => {
                self.advance();
                TokenKind::ExprOpen
            }
            Some('}') => {
                self.advance();
                TokenKind::ExprClose
            }
            Some('=') => {
                self.advance();
                TokenKind::Equals
            }
            Some('"') => {
                self.advance();
                TokenKind::DoubleQuote
            }
            Some('\'') => {
                self.advance();
                TokenKind::SingleQuote
            }
            Some(ch) if ch.is_whitespace() => {
                self.advance();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                TokenKind::Whitespace
            }
            Some(ch) if is_ident_start(ch) => {
                self.advance();
                while let Some(c) = self.peek() {
                    if is_ident_char(c) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                TokenKind::Word
            }
            Some(_) => {
                self.advance();
                TokenKind::Text
            }
            None => TokenKind::EOF,
        };

        let end_pos = self.pos;
        let value = self.chars[start_pos..end_pos].iter().collect();
        Token {
            kind,
            value,
            span: Span::new(start_pos, end_pos, start_line, start_col),
        }
    }

    pub fn scan_raw_text(&mut self, end_tag: &str) -> Token {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        while self.pos + end_tag.len() <= self.chars.len() {
            let substr: String = self.chars[self.pos..self.pos + end_tag.len()]
                .iter()
                .collect();
            if substr.to_lowercase() == end_tag.to_lowercase() {
                break;
            }
            self.advance();
        }

        let end_pos = self.pos;
        let value = self.chars[start_pos..end_pos].iter().collect();
        Token {
            kind: TokenKind::Text,
            value,
            span: Span::new(start_pos, end_pos, start_line, start_col),
        }
    }
}

pub(crate) fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == ':' || c == '-'
}

pub(crate) fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ':' || c == '-' || c == '.'
}
