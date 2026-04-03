//! Parser for building the HTML/GLX AST from tokens.

use super::ast::*;
use super::scanner::*;
use commons::error::ParseError;

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current_token: Token,
    peek_token: Token,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut scanner = Scanner::new(input);
        let current_token = scanner.scan_token();
        let peek_token = scanner.scan_token();
        Self {
            scanner,
            current_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    fn advance(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.scanner.scan_token());
    }

    pub fn parse(&mut self) -> (Vec<Node>, Vec<ParseError>) {
        let mut nodes = Vec::new();
        while self.current_token.kind != TokenKind::EOF {
            match self.parse_node() {
                Ok(node) => nodes.push(node),
                Err(err) => {
                    self.errors.push(err);
                    self.sync();
                }
            }
        }
        (nodes, std::mem::take(&mut self.errors))
    }

    fn sync(&mut self) {
        while self.current_token.kind != TokenKind::EOF {
            match self.current_token.kind {
                TokenKind::OpenTag
                | TokenKind::CloseTagOpen
                | TokenKind::CommentOpen
                | TokenKind::DoctypeOpen
                | TokenKind::ExprOpen => {
                    return;
                }
                _ => self.advance(),
            }
        }
    }

    fn parse_node(&mut self) -> Result<Node, ParseError> {
        match self.current_token.kind {
            TokenKind::OpenTag => {
                if self.peek_token.kind == TokenKind::CloseTag {
                    self.parse_fragment()
                } else {
                    self.parse_element()
                }
            }
            TokenKind::CommentOpen => self.parse_comment(),
            TokenKind::DoctypeOpen => self.parse_doctype(),
            TokenKind::ExprOpen => self.parse_expression(),
            TokenKind::Whitespace => self.parse_whitespace(),
            TokenKind::Text
            | TokenKind::Word
            | TokenKind::DoubleQuote
            | TokenKind::SingleQuote
            | TokenKind::Equals => self.parse_text(),
            _ => {
                // Default to text if we encounter something unexpected
                self.parse_text()
            }
        }
    }

    fn parse_fragment(&mut self) -> Result<Node, ParseError> {
        let (start_pos, start_line, start_col) = (
            self.current_token.span.start,
            self.current_token.span.line,
            self.current_token.span.column,
        );
        self.advance(); // consume '<'
        self.advance(); // consume '>'

        let mut children = Vec::new();
        while self.current_token.kind != TokenKind::CloseTagOpen
            && self.current_token.kind != TokenKind::EOF
        {
            children.push(self.parse_node()?);
        }

        if self.current_token.kind == TokenKind::CloseTagOpen {
            self.advance(); // consume '</'
            if self.current_token.kind == TokenKind::CloseTag {
                self.advance(); // consume '>'
            }
        }

        let end_span = self.current_token.span.clone();
        Ok(Node::Fragment(Fragment {
            children,
            span: Span::new(start_pos, end_span.start, start_line, start_col),
        }))
    }

    fn parse_whitespace(&mut self) -> Result<Node, ParseError> {
        let span = self.current_token.span.clone();
        let value = self.current_token.value.clone();
        self.advance();
        Ok(Node::Whitespace(Whitespace { value, span }))
    }

    fn parse_element(&mut self) -> Result<Node, ParseError> {
        let (start_pos, start_line, start_col) = (
            self.current_token.span.start,
            self.current_token.span.line,
            self.current_token.span.column,
        );
        let opening_start = self.current_token.span.clone();
        self.advance(); // consume '<'

        if self.current_token.kind != TokenKind::Word {
            return Err(ParseError::InvalidFile {
                msg: format!(
                    "Expected tag name after '<' at {}:{}",
                    self.current_token.span.line, self.current_token.span.column
                ),
            });
        }

        let name = self.current_token.value.clone();
        self.advance(); // consume tag name

        let kind = if name.chars().next().is_some_and(|c| c.is_uppercase()) {
            ElementKind::Component
        } else {
            ElementKind::Html
        };

        let mut attributes = Vec::new();
        while self.current_token.kind != TokenKind::CloseTag
            && self.current_token.kind != TokenKind::SelfClose
            && self.current_token.kind != TokenKind::EOF
        {
            match self.current_token.kind {
                TokenKind::Whitespace => {
                    attributes.push(AttributeNode::Whitespace(Whitespace {
                        value: self.current_token.value.clone(),
                        span: self.current_token.span.clone(),
                    }));
                    self.advance();
                }
                TokenKind::CommentOpen => {
                    attributes.push(AttributeNode::Comment(self.parse_comment_struct()?));
                }
                _ => {
                    attributes.push(AttributeNode::Attribute(self.parse_attribute()?));
                }
            }
        }

        let mut closing_span = None;
        let self_closing = if self.current_token.kind == TokenKind::SelfClose {
            let opening_end = self.current_token.span.clone();
            self.advance(); // consume '/>'
            let opening_span = Span::new(
                opening_start.start,
                opening_end.end,
                opening_start.line,
                opening_start.column,
            );

            let end_pos = self.current_token.span.start;
            return Ok(Node::Element(Box::new(Element {
                name,
                kind,
                attributes,
                children: Vec::new(),
                self_closing: true,
                opening_span,
                closing_span: None,
                span: Span::new(start_pos, end_pos, start_line, start_col),
            })));
        } else if self.current_token.kind == TokenKind::CloseTag {
            let opening_end = self.current_token.span.clone();
            self.advance(); // consume '>'
            let opening_span = Span::new(
                opening_start.start,
                opening_end.end,
                opening_start.line,
                opening_start.column,
            );
            (false, opening_span)
        } else {
            // Error case, but we try to recover
            (
                false,
                Span::new(
                    opening_start.start,
                    self.current_token.span.start,
                    opening_start.line,
                    opening_start.column,
                ),
            )
        };

        let (self_closing, opening_span) = self_closing;

        let mut children = Vec::new();
        if !self_closing && !is_void_element(&name) {
            let lower_name = name.to_lowercase();
            if lower_name == "script" || lower_name == "style" {
                // We need to scan raw text until the closing tag.
                self.scanner.pos = self.current_token.span.start;
                self.scanner.line = self.current_token.span.line;
                self.scanner.column = self.current_token.span.column;

                let end_tag = format!("</{}", name);
                let text_token = self.scanner.scan_raw_text(&end_tag);
                if !text_token.value.is_empty() {
                    children.push(Node::Text(Text {
                        value: text_token.value,
                        span: text_token.span,
                    }));
                }
                // Resync tokens
                self.current_token = self.scanner.scan_token(); // Should be CloseTagOpen
                self.peek_token = self.scanner.scan_token(); // Should be tag name or CloseTag
            } else {
                while self.current_token.kind != TokenKind::CloseTagOpen
                    && self.current_token.kind != TokenKind::EOF
                {
                    children.push(self.parse_node()?);
                }
            }

            if self.current_token.kind == TokenKind::CloseTagOpen {
                let cs_start = self.current_token.span.clone();
                self.advance(); // consume '</'
                if self.current_token.kind == TokenKind::Word {
                    self.advance();
                }
                if self.current_token.kind == TokenKind::CloseTag {
                    let cs_end = self.current_token.span.clone();
                    closing_span = Some(Span::new(
                        cs_start.start,
                        cs_end.end,
                        cs_start.line,
                        cs_start.column,
                    ));
                    self.advance();
                }
            }
        }

        let end_pos = self.current_token.span.start;
        Ok(Node::Element(Box::new(Element {
            name,
            kind,
            attributes,
            children,
            self_closing,
            opening_span,
            closing_span,
            span: Span::new(start_pos, end_pos, start_line, start_col),
        })))
    }

    fn parse_attribute(&mut self) -> Result<Attribute, ParseError> {
        if self.current_token.kind == TokenKind::ExprOpen {
            if self.peek_token.value == "..." {
                return self.parse_spread_attribute();
            } else {
                return self.parse_shorthand_attribute();
            }
        }

        let start_span = self.current_token.span.clone();
        let key_span = start_span.clone();
        let key = self.current_token.value.clone();
        self.advance();

        if self.current_token.kind == TokenKind::Equals {
            self.advance(); // consume '='

            if self.current_token.kind == TokenKind::DoubleQuote
                || self.current_token.kind == TokenKind::SingleQuote
            {
                let quote_token = self.current_token.kind.clone();
                let quote_kind = if quote_token == TokenKind::DoubleQuote {
                    QuoteKind::Double
                } else {
                    QuoteKind::Single
                };
                let v_start_span = self.current_token.span.clone();
                self.advance(); // consume opening quote
                let mut value = String::new();
                while self.current_token.kind != quote_token
                    && self.current_token.kind != TokenKind::EOF
                {
                    value.push_str(&self.current_token.value);
                    self.advance();
                }
                let v_end_span = self.current_token.span.clone();
                self.advance(); // consume closing quote

                let value_span = Span::new(
                    v_start_span.start,
                    v_end_span.end,
                    v_start_span.line,
                    v_start_span.column,
                );
                let end_span = self.current_token.span.clone();
                if key.starts_with("on") && !key.contains(':') {
                    Ok(Attribute::Event {
                        name: key,
                        handler: value,
                        quote: quote_kind,
                        key_span,
                        handler_span: value_span,
                        span: Span::new(
                            start_span.start,
                            end_span.start,
                            start_span.line,
                            start_span.column,
                        ),
                    })
                } else if key.contains(':') {
                    Ok(Attribute::Directive {
                        name: key.clone(),
                        value: Some(value),
                        kind: DirectiveKind::from(key.as_str()),
                        quote: quote_kind,
                        key_span,
                        value_span: Some(value_span),
                        span: Span::new(
                            start_span.start,
                            end_span.start,
                            start_span.line,
                            start_span.column,
                        ),
                    })
                } else {
                    Ok(Attribute::Static {
                        key,
                        value,
                        quote: quote_kind,
                        is_boolean: false,
                        key_span,
                        value_span: Some(value_span),
                        span: Span::new(
                            start_span.start,
                            end_span.start,
                            start_span.line,
                            start_span.column,
                        ),
                    })
                }
            } else if self.current_token.kind == TokenKind::ExprOpen {
                let v_start_span = self.current_token.span.clone();
                let expr = self.parse_expression_content()?;
                let v_end_span = self.current_token.span.clone();
                let value_span = Span::new(
                    v_start_span.start,
                    v_end_span.start,
                    v_start_span.line,
                    v_start_span.column,
                );
                let end_span = self.current_token.span.clone();
                if key.starts_with("on") && !key.contains(':') {
                    Ok(Attribute::Event {
                        name: key,
                        handler: expr,
                        quote: QuoteKind::None,
                        key_span,
                        handler_span: value_span,
                        span: Span::new(
                            start_span.start,
                            end_span.start,
                            start_span.line,
                            start_span.column,
                        ),
                    })
                } else {
                    Ok(Attribute::Dynamic {
                        key,
                        expr,
                        key_span,
                        value_span,
                        span: Span::new(
                            start_span.start,
                            end_span.start,
                            start_span.line,
                            start_span.column,
                        ),
                    })
                }
            } else {
                // Bare value attribute
                let v_start_span = self.current_token.span.clone();
                let value = self.current_token.value.clone();
                self.advance();
                let v_end_span = self.current_token.span.clone();
                let value_span = Some(Span::new(
                    v_start_span.start,
                    v_end_span.start,
                    v_start_span.line,
                    v_start_span.column,
                ));
                let end_span = self.current_token.span.clone();
                Ok(Attribute::Static {
                    key,
                    value,
                    quote: QuoteKind::None,
                    is_boolean: false,
                    key_span,
                    value_span,
                    span: Span::new(
                        start_span.start,
                        end_span.start,
                        start_span.line,
                        start_span.column,
                    ),
                })
            }
        } else {
            // Boolean attribute
            let end_span = self.current_token.span.clone();
            if key.contains(':') {
                Ok(Attribute::Directive {
                    name: key.clone(),
                    value: None,
                    kind: DirectiveKind::from(key.as_str()),
                    quote: QuoteKind::None,
                    key_span,
                    value_span: None,
                    span: Span::new(
                        start_span.start,
                        end_span.start,
                        start_span.line,
                        start_span.column,
                    ),
                })
            } else {
                Ok(Attribute::Static {
                    key,
                    value: "true".to_string(),
                    quote: QuoteKind::None,
                    is_boolean: true,
                    key_span,
                    value_span: None,
                    span: Span::new(
                        start_span.start,
                        end_span.start,
                        start_span.line,
                        start_span.column,
                    ),
                })
            }
        }
    }

    fn parse_shorthand_attribute(&mut self) -> Result<Attribute, ParseError> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume '{'
        if self.current_token.kind != TokenKind::Word {
            return Err(ParseError::InvalidFile {
                msg: format!(
                    "Expected identifier in shorthand attribute at {}:{}",
                    self.current_token.span.line, self.current_token.span.column
                ),
            });
        }
        let key = self.current_token.value.clone();
        self.advance();
        if self.current_token.kind == TokenKind::ExprClose {
            self.advance();
        }
        let end_span = self.current_token.span.clone();
        Ok(Attribute::Shorthand {
            key,
            span: Span::new(
                start_span.start,
                end_span.start,
                start_span.line,
                start_span.column,
            ),
        })
    }

    fn parse_spread_attribute(&mut self) -> Result<Attribute, ParseError> {
        let start_span = self.current_token.span.clone();
        let full_expr = self.parse_expression_content()?;
        let expr = full_expr
            .strip_prefix("...")
            .unwrap_or(&full_expr)
            .to_string();
        let end_span = self.current_token.span.clone();
        Ok(Attribute::Spread {
            expr,
            span: Span::new(
                start_span.start,
                end_span.start,
                start_span.line,
                start_span.column,
            ),
        })
    }

    fn parse_expression(&mut self) -> Result<Node, ParseError> {
        let start_span = self.current_token.span.clone();
        let content = self.parse_expression_content()?;
        let end_span = self.current_token.span.clone();
        Ok(Node::Expression(Expression {
            content,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                start_span.column,
            ),
        }))
    }

    fn parse_expression_content(&mut self) -> Result<String, ParseError> {
        self.advance(); // consume '{'
        let mut content = String::new();
        let mut depth = 1;

        while depth > 0 && self.current_token.kind != TokenKind::EOF {
            let kind = self.current_token.kind.clone();
            let value = self.current_token.value.clone();

            match kind {
                TokenKind::ExprOpen => {
                    depth += 1;
                    content.push_str(&value);
                    self.advance();
                }
                TokenKind::ExprClose => {
                    depth -= 1;
                    if depth > 0 {
                        content.push_str(&value);
                        self.advance();
                    }
                }
                TokenKind::DoubleQuote | TokenKind::SingleQuote => {
                    content.push_str(&value);
                    let _quote = value;
                    self.advance();
                    while self.current_token.kind != kind
                        && self.current_token.kind != TokenKind::EOF
                    {
                        if self.current_token.value == "\\" {
                            content.push_str(&self.current_token.value);
                            self.advance();
                            if self.current_token.kind != TokenKind::EOF {
                                content.push_str(&self.current_token.value);
                                self.advance();
                            }
                        } else {
                            content.push_str(&self.current_token.value);
                            self.advance();
                        }
                    }
                    if self.current_token.kind == kind {
                        content.push_str(&self.current_token.value);
                        self.advance();
                    }
                }
                _ if value == "`" => {
                    // Template literal
                    content.push_str(&value);
                    self.advance();
                    while self.current_token.value != "`"
                        && self.current_token.kind != TokenKind::EOF
                    {
                        if self.current_token.value == "$"
                            && self.peek_token.kind == TokenKind::ExprOpen
                        {
                            content.push_str(&self.current_token.value); // push $
                            self.advance();
                            content.push_str(&self.current_token.value); // push {
                            content.push_str(&self.parse_expression_content()?);
                            content.push('}');
                        } else if self.current_token.value == "\\" {
                            content.push_str(&self.current_token.value);
                            self.advance();
                            if self.current_token.kind != TokenKind::EOF {
                                content.push_str(&self.current_token.value);
                                self.advance();
                            }
                        } else {
                            content.push_str(&self.current_token.value);
                            self.advance();
                        }
                    }
                    if self.current_token.value == "`" {
                        content.push_str(&self.current_token.value);
                        self.advance();
                    }
                }
                _ => {
                    content.push_str(&value);
                    self.advance();
                }
            }
        }

        if self.current_token.kind == TokenKind::ExprClose {
            self.advance();
        }

        Ok(content)
    }

    fn parse_comment(&mut self) -> Result<Node, ParseError> {
        Ok(Node::Comment(self.parse_comment_struct()?))
    }

    fn parse_comment_struct(&mut self) -> Result<Comment, ParseError> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume '<!--'
        let mut content = String::new();
        while self.current_token.kind != TokenKind::CommentClose
            && self.current_token.kind != TokenKind::EOF
        {
            content.push_str(&self.current_token.value);
            self.advance();
        }
        if self.current_token.kind == TokenKind::CommentClose {
            self.advance();
        }
        let end_span = self.current_token.span.clone();
        Ok(Comment {
            content,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                start_span.column,
            ),
        })
    }

    fn parse_doctype(&mut self) -> Result<Node, ParseError> {
        Ok(Node::Doctype(self.parse_doctype_struct()?))
    }

    fn parse_doctype_struct(&mut self) -> Result<Doctype, ParseError> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume '<!' or '<!DOCTYPE'
        let mut value = String::new();
        while self.current_token.kind != TokenKind::CloseTag
            && self.current_token.kind != TokenKind::EOF
        {
            value.push_str(&self.current_token.value);
            self.advance();
        }
        if self.current_token.kind == TokenKind::CloseTag {
            self.advance();
        }
        let end_span = self.current_token.span.clone();
        Ok(Doctype {
            value,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                start_span.column,
            ),
        })
    }

    fn parse_text(&mut self) -> Result<Node, ParseError> {
        let start_span = self.current_token.span.clone();
        let mut value = String::new();

        while self.current_token.kind != TokenKind::OpenTag
            && self.current_token.kind != TokenKind::CloseTagOpen
            && self.current_token.kind != TokenKind::ExprOpen
            && self.current_token.kind != TokenKind::Whitespace
            && self.current_token.kind != TokenKind::EOF
        {
            value.push_str(&self.current_token.value);
            self.advance();
        }

        let end_span = self.current_token.span.clone();
        Ok(Node::Text(Text {
            value,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                start_span.column,
            ),
        }))
    }
}

pub(crate) fn is_void_element(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
