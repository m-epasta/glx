use compiler::ast::html::*;

pub struct Formatter {
    indent_size: usize,
}

impl Formatter {
    pub fn new(indent_size: usize) -> Self {
        Self { indent_size }
    }

    pub fn format(&self, nodes: &mut [Node]) -> String {
        let mut visitor = FormatVisitor::new(self.indent_size);
        for node in nodes {
            visitor.visit_node(node);
        }
        visitor.output.trim().to_string()
    }
}

struct FormatVisitor {
    output: String,
    indent_level: usize,
    indent_size: usize,
}

impl FormatVisitor {
    fn new(indent_size: usize) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size,
        }
    }

    fn push_indent(&mut self) {
        self.output
            .push_str(&" ".repeat(self.indent_level * self.indent_size));
    }
}

impl Visitor for FormatVisitor {
    fn visit_node(&mut self, node: &mut Node) {
        match node {
            Node::Element(el) => self.visit_element(el),
            Node::Text(text) => self.visit_text(text),
            Node::Expression(expr) => self.visit_expression(expr),
            Node::Comment(comment) => self.visit_comment(comment),
            Node::Doctype(doctype) => self.visit_doctype(doctype),
            Node::Whitespace(_) => {} // Structural formatter ignores existing whitespace
            Node::Fragment(f) => self.visit_fragment(f),
        }
    }

    fn visit_element(&mut self, element: &mut Element) {
        self.push_indent();
        self.output.push('<');
        self.output.push_str(&element.name);

        let mut last_was_attribute = false;
        for attr_node in &mut element.attributes {
            match attr_node {
                AttributeNode::Attribute(attr) => {
                    if last_was_attribute {
                        self.output.push(' ');
                    }
                    self.visit_attribute(attr);
                    last_was_attribute = true;
                }
                AttributeNode::Whitespace(ws) => {
                    self.output.push_str(&ws.value);
                    last_was_attribute = false;
                }
                AttributeNode::Comment(comment) => {
                    self.output.push(' ');
                    self.visit_comment(comment);
                    last_was_attribute = false;
                }
            }
        }

        if element.self_closing {
            self.output.push_str(" />\n");
        } else {
            self.output.push_str(">\n");
            self.indent_level += 1;
            for child in &mut element.children {
                self.visit_node(child);
            }
            self.indent_level -= 1;
            self.push_indent();
            self.output.push_str("</");
            self.output.push_str(&element.name);
            self.output.push_str(">\n");
        }
    }

    fn visit_static_attribute(&mut self, attribute: &mut Attribute) {
        if let Attribute::Static {
            key,
            value,
            quote,
            is_boolean,
            ..
        } = attribute
        {
            self.output.push_str(key);
            if !*is_boolean {
                let q = match quote {
                    QuoteKind::Double => "\"",
                    QuoteKind::Single => "'",
                    QuoteKind::None => "",
                };
                self.output.push('=');
                self.output.push_str(q);
                self.output.push_str(value);
                self.output.push_str(q);
            }
        }
    }

    fn visit_dynamic_attribute(&mut self, attribute: &mut Attribute) {
        if let Attribute::Dynamic { key, expr, .. } = attribute {
            self.output.push_str(key);
            self.output.push_str("={");
            self.output.push_str(expr);
            self.output.push('}');
        }
    }

    fn visit_spread_attribute(&mut self, attribute: &mut Attribute) {
        if let Attribute::Spread { expr, .. } = attribute {
            self.output.push_str("{...");
            self.output.push_str(expr);
            self.output.push('}');
        }
    }

    fn visit_shorthand_attribute(&mut self, attribute: &mut Attribute) {
        if let Attribute::Shorthand { key, .. } = attribute {
            self.output.push('{');
            self.output.push_str(key);
            self.output.push('}');
        }
    }

    fn visit_directive_attribute(&mut self, attribute: &mut Attribute) {
        if let Attribute::Directive {
            name, value, quote, ..
        } = attribute
        {
            self.output.push_str(name);
            if let Some(v) = value {
                let q = match quote {
                    QuoteKind::Double => "\"",
                    QuoteKind::Single => "'",
                    QuoteKind::None => "",
                };
                self.output.push('=');
                self.output.push_str(q);
                self.output.push_str(v);
                self.output.push_str(q);
            }
        }
    }

    fn visit_event_attribute(&mut self, attribute: &mut Attribute) {
        if let Attribute::Event {
            name,
            handler,
            quote,
            ..
        } = attribute
        {
            self.output.push_str(name);
            self.output.push('=');
            match quote {
                QuoteKind::Double => {
                    self.output.push('"');
                    self.output.push_str(handler);
                    self.output.push('"');
                }
                QuoteKind::Single => {
                    self.output.push('\'');
                    self.output.push_str(handler);
                    self.output.push('\'');
                }
                QuoteKind::None => {
                    self.output.push('{');
                    self.output.push_str(handler);
                    self.output.push('}');
                }
            }
        }
    }

    fn visit_text(&mut self, text: &mut Text) {
        let trimmed = text.value.trim();
        if !trimmed.is_empty() {
            self.push_indent();
            self.output.push_str(trimmed);
            self.output.push('\n');
        }
    }

    fn visit_expression(&mut self, expression: &mut Expression) {
        self.push_indent();
        self.output.push('{');
        self.output.push_str(&expression.content);
        self.output.push_str("}\n");
    }

    fn visit_comment(&mut self, comment: &mut Comment) {
        self.push_indent();
        self.output.push_str("<!-- ");
        self.output.push_str(comment.content.trim());
        self.output.push_str(" -->\n");
    }

    fn visit_doctype(&mut self, doctype: &mut Doctype) {
        self.push_indent();
        self.output.push_str("<!DOCTYPE ");
        self.output.push_str(doctype.value.trim());
        self.output.push_str(">\n");
    }

    fn visit_fragment(&mut self, fragment: &mut Fragment) {
        self.push_indent();
        self.output.push_str("<>\n");
        self.indent_level += 1;
        for child in &mut fragment.children {
            self.visit_node(child);
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("</>\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compiler::ast::html::Parser;

    #[test]
    fn test_format_basic() {
        let input = "<div><p>Hello  </p>   </div>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty(), "Errors: {:?}", errors);

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);

        let expected = "<div>\n  <p>\n    Hello\n  </p>\n</div>";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_fragments() {
        let input = "<><div>Hello</div></>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty());

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);

        let expected = "<>\n  <div>\n    Hello\n  </div>\n</>";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_syntax_preservation() {
        let input = "<div class='foo' checked id=bar></div>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty());

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);

        // Formatter keeps single quotes and boolean attribute style
        let expected = "<div class='foo' checked id=bar>\n</div>";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_complex_expression() {
        let input = "<div>{ \" { brace in string } \" }</div>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty());

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);

        let expected = "<div>\n  { \" { brace in string } \" }\n</div>";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_events() {
        let input = "<button onclick=\"handleClick()\" onmouseover={handleHover}>Click</button>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty());

        // Verify that they are indeed Event attributes
        if let Node::Element(el) = &nodes[0] {
            // el.attributes[0] is Whitespace(" ")
            assert!(matches!(
                el.attributes[1],
                AttributeNode::Attribute(Attribute::Event { .. })
            ));
            // el.attributes[2] is Whitespace(" ")
            assert!(matches!(
                el.attributes[3],
                AttributeNode::Attribute(Attribute::Event { .. })
            ));
        }

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);
        
        let expected = "<button onclick=\"handleClick()\" onmouseover={handleHover}>\n  Click\n</button>";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_script_raw_text() {
        let input = "<script>if (a < b && c > d) console.log('safe');</script>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty());

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);

        // Formatter currently adds a newline after <script> and indent
        let expected = "<script>\n  if (a < b && c > d) console.log('safe');\n</script>";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_lossless_attributes() {
        let input = "<div   class=\"foo\"  \n     id=\"bar\" ></div>";
        let mut parser = Parser::new(input);
        let (mut nodes, errors) = parser.parse();
        assert!(errors.is_empty());

        // Verify AST has whitespace nodes
        if let Node::Element(el) = &nodes[0] {
            assert!(matches!(el.attributes[0], AttributeNode::Whitespace(_)));
            assert!(matches!(el.attributes[2], AttributeNode::Whitespace(_)));
        }

        let formatter = Formatter::new(2);
        let output = formatter.format(&mut nodes);

        // The formatter preserves the whitespace if it's in the AST
        let expected = "<div   class=\"foo\"  \n     id=\"bar\" >\n</div>";
        assert_eq!(output, expected);
    }
}
