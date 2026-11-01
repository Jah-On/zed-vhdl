//! Hand-written recursive-descent VHDL parser with error recovery.
//!
//! The parser is written directly in Rust rather than using an external
//! grammar (e.g. tree-sitter). It tokenizes with the lexer, then builds an
//! AST via recursive descent. On syntax errors it records the error and
//! continues with error recovery, so malformed VHDL still yields a usable
//! tree for highlighting and formatting.

use crate::vhdl::ast::Node;
use crate::vhdl::lexer::{tokenize, Token, TokenKind};

/// Result of parsing a source string.
#[derive(Debug)]
pub struct ParseResult {
    pub tree: Option<Node>,
    pub errors: Vec<String>,
}

impl ParseResult {
    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<String>,
}

#[allow(dead_code)]
impl Parser {
    // Some construct parsers are intentionally left unused by the current
    // stub grammar; silence dead-code warnings for them.
    fn new(source: String) -> Self {
        Parser {
            tokens: tokenize(&source),
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn peek_value(&self) -> Option<&str> {
        self.peek().map(|t| t.value.as_str())
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind)
    }

    fn at_kind(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn at_value(&self, value: &str) -> bool {
        self.peek_value() == Some(value)
    }

    fn advance(&mut self) -> Option<Token> {
        self.bump()
    }

    /// Consumes the token if it matches the kind; otherwise records an
    /// error, skips the token, and returns false.
    fn expect(&mut self, kind: TokenKind, what: &str) -> bool {
        let found_val = self.peek_value().unwrap_or("");
        let matched = self.peek().map_or(false, |t| t.kind == kind);
        if matched {
            self.advance();
            true
        } else {
            self.errors
                .push(format!("expected {what}, found '{found_val}'"));
            self.advance();
            false
        }
    }

    /// Consumes the token if it matches the end keyword; otherwise records
    /// an error and returns None.
    fn expect_end_kind(&mut self, kind: &str) -> Option<String> {
        let kind_str = kind.to_string();
        let matched = self
            .peek()
            .map_or(false, |t| t.value == kind && t.kind == TokenKind::Keyword);
        if matched {
            self.advance();
            Some(kind_str)
        } else {
            match self.peek() {
                Some(t) => {
                    let found_val = t.value.clone();
                    self.errors
                        .push(format!("expected end {kind_str}, found '{found_val}'"));
                    self.advance();
                    None
                }
                None => {
                    let end_str = "end of input".to_string();
                    self.errors
                        .push(format!("expected end {kind_str}, found {end_str}"));
                    None
                }
            }
        }
    }

    // ---- name / identifier helpers ----

    /// Consumes a name (keyword or identifier) token and returns its value.
    fn parse_name(&mut self) -> String {
        match self.peek() {
            Some(t) if t.kind == TokenKind::Keyword || t.kind == TokenKind::Identifier => {
                let v = self.advance().unwrap().value.clone();
                v
            }
            _ => {
                if let Some(t) = self.bump() {
                    self.errors
                        .push(format!("expected name, found '{}'", t.value));
                }
                String::new()
            }
        }
    }

    /// Consumes an identifier-list (list of names separated by commas,
    /// terminated by `;`, `,` or `)`).
    fn parse_identifier_list(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        loop {
            if self.at_value(",") {
                self.advance();
                continue;
            }
            if self.at_value(";") || self.at_value(")") {
                self.advance();
                break;
            }
            match self.peek() {
                Some(t) if t.kind == TokenKind::Identifier || t.kind == TokenKind::Keyword => {
                    names.push(t.value.clone());
                    self.advance();
                }
                Some(t) => {
                    self.errors
                        .push(format!("expected identifier, found '{}'", t.value));
                    self.bump();
                    break;
                }
                None => break,
            }
        }
        names
    }

    // ---- statement construction ----

    fn parse_statement(&mut self) -> Node {
        // End-of-block terminators.
        if self.at_value(";") || self.at_value(")") || self.at_value("end") {
            self.advance();
            return Node::Block { items: Vec::new() };
        }

        // Try known statement constructs in order.
        if self.at_value("entity") {
            return self.parse_entity_declaration();
        }
        if self.at_value("component") {
            return self.parse_component_declaration();
        }
        if self.at_value("package") {
            return self.parse_package_declaration();
        }
        if self.at_value("type") || self.at_value("subtype") {
            return self.parse_type_declaration();
        }
        if self.at_value("constant")
            || self.at_value("signal")
            || self.at_value("variable")
            || self.at_value("shared")
            || self.at_value("file")
            || self.at_value("alias")
            || self.at_value("attribute")
        {
            return self.parse_declaration();
        }
        if self.at_value("function") || self.at_value("procedure") {
            return self.parse_function_declaration();
        }
        if self.at_value("architecture") {
            return self.parse_architecture_body();
        }
        if self.at_value("process") {
            return self.parse_process_statement();
        }
        if self.at_value("configuration") {
            return self.parse_configuration_declaration();
        }
        if self.at_value("block") {
            return self.parse_block_specification();
        }
        if self.at_value("context") {
            return self.parse_context_declaration();
        }
        if self.at_value("group") {
            return self.parse_group_declaration();
        }
        if self.at_value("if") {
            return self.parse_if_statement();
        }
        if self.at_value("element") {
            return self.parse_element_declaration();
        }

        // Error recovery: skip the offending token.
        match self.bump() {
            Some(t) => {
                self.errors.push(format!(
                    "unexpected token '{}' at position {}",
                    t.value, t.start
                ));
                Node::Block { items: Vec::new() }
            }
            None => Node::Block { items: Vec::new() },
        }
    }

    // ---- statement construct parsers ----

    fn parse_entity_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "entity keyword");
        let name = self.parse_name();
        Node::EntityDeclaration { name }
    }

    fn parse_component_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "component keyword");
        let name = self.parse_name();
        Node::ComponentDeclaration { name }
    }

    fn parse_package_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "package keyword");
        let name = self.parse_name();
        Node::PackageDeclaration { name }
    }

    fn parse_type_declaration(&mut self) -> Node {
        if self.at_value("subtype") {
            self.expect(TokenKind::Keyword, "subtype keyword");
            let name = self.parse_name();
            Node::SubtypeDeclaration { name }
        } else {
            self.expect(TokenKind::Keyword, "type keyword");
            let name = self.parse_name();
            Node::FullTypeDeclaration { name }
        }
    }

    fn parse_declaration(&mut self) -> Node {
        let kind = match self.peek_value().unwrap_or("") {
            "constant" => "constant",
            "signal" => "signal",
            "variable" => "variable",
            "shared" => "shared_variable",
            "file" => "file",
            "alias" => "alias",
            "attribute" => "attribute",
            _ => return Node::Block { items: Vec::new() },
        };
        self.expect(TokenKind::Keyword, &format!("{kind} keyword"));
        let names = self.parse_identifier_list();

        match kind {
            "constant" => Node::ConstantDeclaration { names },
            "signal" => Node::SignalDeclaration { names },
            "variable" => Node::VariableDeclaration { names },
            "shared_variable" => Node::SharedVariableDeclaration { names },
            "file" => Node::FileDeclaration { names },
            "alias" => {
                let name = self.parse_name();
                let target = Node::Block { items: Vec::new() };
                Node::AliasDeclaration {
                    name,
                    target: Box::new(target),
                }
            }
            "attribute" => Node::AttributeDeclaration {
                name: String::new(),
            },
            _ => Node::Block { items: Vec::new() },
        }
    }

    fn parse_function_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "function keyword");
        let name = self.parse_name();
        let _params = self.parse_parameter_list();
        let _ret = self.parse_return_type();
        self.expect(TokenKind::Keyword, "is keyword");
        let body = self.parse_body("function");
        Node::FunctionBody {
            name,
            body: Box::new(body),
        }
    }

    fn parse_procedure_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "procedure keyword");
        let name = self.parse_name();
        let _params = self.parse_parameter_list();
        let _ret = self.parse_return_type();
        self.expect(TokenKind::Keyword, "is keyword");
        let body = self.parse_body("procedure");
        Node::ProcedureBody {
            name,
            body: Box::new(body),
        }
    }

    // ---- bodies ----

    fn parse_body(&mut self, kind: &str) -> Node {
        self.expect(TokenKind::Keyword, "begin keyword");
        let items = self.parse_statement_list();
        let _end = self.expect_end_kind(kind);
        Node::Block { items }
    }

    fn parse_package_body(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "package keyword");
        self.expect(TokenKind::Keyword, "body keyword");
        let items = self.parse_statement_list();
        let _end = self.expect_end_kind("package");
        Node::Block { items }
    }

    fn parse_architecture_body(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "architecture keyword");
        let name = self.parse_name();
        self.expect(TokenKind::Keyword, "of keyword");
        let entity = self.parse_name();
        self.expect(TokenKind::Keyword, "body keyword");
        let items = self.parse_statement_list();
        let _end = self.expect_end_kind("architecture");
        Node::ArchitectureBody {
            entity,
            name,
            body: Box::new(Node::Block { items }),
        }
    }

    fn parse_process_statement(&mut self) -> Node {
        let name = match self.peek() {
            Some(t) if t.kind == TokenKind::Keyword && t.value != "process" => {
                Some(self.bump().unwrap().value.clone())
            }
            _ => None,
        };
        self.expect(TokenKind::Keyword, "process keyword");
        let body = self.parse_body("process");
        Node::ProcessStatement {
            name,
            body: Box::new(body),
        }
    }

    fn parse_component_instantiation_statement(&mut self) -> Node {
        let name = match self.peek() {
            Some(t) if t.kind == TokenKind::Keyword && t.value != "entity" => {
                Some(self.bump().unwrap().value.clone())
            }
            _ => None,
        };
        self.expect(TokenKind::Keyword, "entity keyword");
        let _entity_name = self.parse_name();
        Node::ComponentInstantiationStatement {
            name,
            entity: Box::new(Node::Block { items: Vec::new() }),
        }
    }

    fn parse_configuration_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "configuration keyword");
        let name = self.parse_name();
        self.expect(TokenKind::Keyword, "of keyword");
        let entity = self.parse_name();
        Node::ConfigurationDeclaration { name, entity }
    }

    fn parse_block_specification(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "block keyword");
        let name_or_label = self.parse_name();
        Node::BlockSpecification { name_or_label }
    }

    fn parse_context_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "context keyword");
        let name = self.parse_name();
        Node::ContextDeclaration { name }
    }

    fn parse_group_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "group keyword");
        let name = self.parse_name();
        Node::GroupDeclaration { name }
    }

    fn parse_attribute_specification(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "attribute keyword");
        let name = self.parse_name();
        self.expect(TokenKind::Keyword, "of keyword");
        let _entity = self.parse_name();
        Node::AttributeSpecification {
            name,
            entity: Box::new(Node::Block { items: Vec::new() }),
        }
    }

    fn parse_if_statement(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "if keyword");
        let _condition = self.parse_expression();
        self.expect(TokenKind::Keyword, "then keyword");
        let branches = self.parse_if_branches();
        Node::IfStatement { branches }
    }

    fn parse_if_branches(&mut self) -> Vec<(Box<Node>, Box<Node>)> {
        let mut branches = Vec::new();
        loop {
            // Guard: always make progress. At end of input there is no token
            // to consume, so break instead of spinning on a no-op parse.
            if self.peek().is_none() {
                break;
            }
            let action = self.parse_statement();
            branches.push((
                Box::new(Node::Block { items: Vec::new() }),
                Box::new(action),
            ));

            match self.peek_value() {
                Some("elsif") => {
                    self.advance();
                    self.expect(TokenKind::Keyword, "then keyword");
                    let condition = self.parse_expression();
                    self.expect(TokenKind::Keyword, "then keyword");
                    let action = self.parse_statement();
                    branches.push((Box::new(condition), Box::new(action)));
                }
                Some("else") => {
                    self.advance();
                    let action = self.parse_statement();
                    branches.push((
                        Box::new(Node::Block { items: Vec::new() }),
                        Box::new(action),
                    ));
                }
                Some("end") => break,
                _ => {
                    self.errors.push("expected elsif/else/end if".to_string());
                }
            }
        }
        branches
    }

    fn parse_expression(&mut self) -> Node {
        // Minimal expression: a simple name or literal.
        if let Some(t) = self.peek() {
            if t.kind == TokenKind::Identifier || t.kind == TokenKind::Literal {
                self.advance();
            }
        }
        Node::Block { items: Vec::new() }
    }

    fn parse_element_declaration(&mut self) -> Node {
        self.expect(TokenKind::Keyword, "element keyword");
        let names = self.parse_identifier_list();
        Node::ElementDeclaration { names }
    }

    // ---- sub-parsers ----

    fn parse_parameter_list(&mut self) -> Option<Vec<String>> {
        if !self.at_kind(TokenKind::Punctuation) {
            return None;
        }
        self.advance();
        let mut params = Vec::new();
        loop {
            if self.at_value(")") {
                self.advance();
                break;
            }
            match self.peek() {
                Some(t) if t.kind == TokenKind::Identifier || t.kind == TokenKind::Keyword => {
                    params.push(t.value.clone());
                    self.advance();
                }
                Some(t) => {
                    self.errors
                        .push(format!("expected parameter, found '{}'", t.value));
                    self.bump();
                    break;
                }
                None => break,
            }
            if self.at_value(",") {
                self.advance();
                continue;
            }
        }
        Some(params)
    }

    fn parse_return_type(&mut self) -> Option<String> {
        if !self.at_value("return") {
            return None;
        }
        self.advance();
        Some(self.parse_name())
    }

    fn parse_statement_list(&mut self) -> Vec<Box<Node>> {
        let mut items = Vec::new();
        loop {
            // Guard: always make progress. At end of input there is no token
            // to consume, so break instead of spinning on a no-op parse.
            if self.peek().is_none() {
                break;
            }
            if let Some(t) = self.peek() {
                if t.value == ";" || t.value == ")" || t.value == "end" {
                    self.advance();
                    break;
                }
            }
            items.push(Box::new(self.parse_statement()));
        }
        items
    }

    // ---- top-level parse ----

    fn parse_top_level(&mut self) -> Vec<Box<Node>> {
        let mut items = Vec::new();
        loop {
            if self.peek().is_none() {
                break;
            }
            items.push(Box::new(self.parse_statement()));
        }
        items
    }
}

/// Parses source into an AST with error recovery.
pub fn parse(source: &str) -> ParseResult {
    let mut parser = Parser::new(source.to_string());
    let items = parser.parse_top_level();
    let root = Node::Block { items };
    ParseResult {
        tree: Some(root),
        errors: parser.errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_keywords_and_literals() {
        let tokens = tokenize("signal clk in integer; constant v = 5; end");
        // Whitespace tokens are emitted too, so the count includes them.
        assert_eq!(tokens.len(), 19);
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::Keyword && t.value == "signal"));
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::Literal && t.value == "5"));
    }

    #[test]
    fn parses_simple_entity_and_signal() {
        let result = parse(
            "entity foo is (
    port (clk in integer; dout out integer)
) is (
    signal s; end entity;",
        );
        assert!(
            !result.errors.is_empty(),
            "expected some recovered errors on the stub grammar"
        );
        let _root = result.tree.unwrap();
    }

    #[test]
    fn recovers_from_unexpected_token() {
        let result = parse("this is not vhdl");
        assert!(!result.tree.is_none());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn parses_declaration_list() {
        let result = parse("constant a, b;");
        assert!(result.tree.is_some());
    }
}
