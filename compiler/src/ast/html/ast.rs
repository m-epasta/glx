//! AST definitions for HTML/GLX nodes.

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Box<Element>),
    Text(Text),
    Expression(Expression),
    Comment(Comment),
    Doctype(Doctype),
    Fragment(Fragment),
    Whitespace(Whitespace),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementKind {
    Html,
    Component,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeNode {
    Attribute(Attribute),
    Whitespace(Whitespace),
    Comment(Comment),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub kind: ElementKind,
    pub attributes: Vec<AttributeNode>,
    pub children: Vec<Node>,
    pub self_closing: bool,
    pub opening_span: Span,
    pub closing_span: Option<Span>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub content: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub content: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Doctype {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Whitespace {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteKind {
    Double,
    Single,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    Static {
        key: String,
        value: String,
        quote: QuoteKind,
        is_boolean: bool,
        key_span: Span,
        value_span: Option<Span>,
        span: Span,
    },
    Dynamic {
        key: String,
        expr: String,
        key_span: Span,
        value_span: Span, // For {expr}
        span: Span,
    },
    Spread {
        expr: String,
        span: Span,
    },
    Shorthand {
        key: String,
        span: Span,
    },
    Directive {
        name: String,
        value: Option<String>,
        kind: DirectiveKind,
        quote: QuoteKind,
        key_span: Span,
        value_span: Option<Span>,
        span: Span,
    },
    Event {
        name: String,
        handler: String,
        quote: QuoteKind,
        key_span: Span,
        handler_span: Span,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectiveKind {
    Client,
    SetHtml,
    DefineVars,
    Other,
}

impl From<&str> for DirectiveKind {
    fn from(s: &str) -> Self {
        match s {
            "client" => Self::Client,
            "set:html" => Self::SetHtml,
            "define:vars" => Self::DefineVars,
            _ => Self::Other,
        }
    }
}

pub trait Visitor {
    fn visit_node(&mut self, node: &mut Node) {
        match node {
            Node::Element(el) => self.visit_element(el),
            Node::Text(text) => self.visit_text(text),
            Node::Expression(expr) => self.visit_expression(expr),
            Node::Comment(comment) => self.visit_comment(comment),
            Node::Doctype(doctype) => self.visit_doctype(doctype),
            Node::Fragment(fragment) => self.visit_fragment(fragment),
            Node::Whitespace(whitespace) => self.visit_whitespace(whitespace),
        }
    }

    fn visit_element(&mut self, element: &mut Element) {
        for node in &mut element.attributes {
            self.visit_attribute_node(node);
        }
        for child in &mut element.children {
            self.visit_node(child);
        }
    }

    fn visit_attribute_node(&mut self, node: &mut AttributeNode) {
        match node {
            AttributeNode::Attribute(attr) => self.visit_attribute(attr),
            AttributeNode::Whitespace(ws) => self.visit_whitespace(ws),
            AttributeNode::Comment(comment) => self.visit_comment(comment),
        }
    }

    fn visit_attribute(&mut self, attribute: &mut Attribute) {
        match attribute {
            Attribute::Static { .. } => self.visit_static_attribute(attribute),
            Attribute::Dynamic { .. } => self.visit_dynamic_attribute(attribute),
            Attribute::Spread { .. } => self.visit_spread_attribute(attribute),
            Attribute::Shorthand { .. } => self.visit_shorthand_attribute(attribute),
            Attribute::Directive { .. } => self.visit_directive_attribute(attribute),
            Attribute::Event { .. } => self.visit_event_attribute(attribute),
        }
    }

    fn visit_static_attribute(&mut self, _attribute: &mut Attribute) {}
    fn visit_dynamic_attribute(&mut self, _attribute: &mut Attribute) {}
    fn visit_spread_attribute(&mut self, _attribute: &mut Attribute) {}
    fn visit_shorthand_attribute(&mut self, _attribute: &mut Attribute) {}
    fn visit_directive_attribute(&mut self, _attribute: &mut Attribute) {}
    fn visit_event_attribute(&mut self, _attribute: &mut Attribute) {}
    fn visit_text(&mut self, _text: &mut Text) {}
    fn visit_expression(&mut self, _expression: &mut Expression) {}
    fn visit_comment(&mut self, _comment: &mut Comment) {}
    fn visit_doctype(&mut self, _doctype: &mut Doctype) {}
    fn visit_fragment(&mut self, fragment: &mut Fragment) {
        for child in &mut fragment.children {
            self.visit_node(child);
        }
    }
    fn visit_whitespace(&mut self, _whitespace: &mut Whitespace) {}
}
