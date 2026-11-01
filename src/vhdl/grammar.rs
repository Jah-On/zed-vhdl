//! VHDL grammar.
//!
//! Uses a self-contained, hand-written recursive descent parser (see
//! `parser.rs`) instead of the tree-sitter grammar. This avoids external
//! grammar dependencies and gives finer control over error recovery.

use crate::vhdl::ast::Node;
use crate::vhdl::parser::ParseResult;

/// Parses the given VHDL source into an AST using the custom parser.
pub fn parse(source: &str) -> ParseResult {
    crate::vhdl::parser::parse(source)
}

/// Returns the AST root node for the parsed source, if one was built.
pub fn tree(source: &str) -> Option<Node> {
    parse(source).tree
}

/// Recovered parse errors, when any occurred.
pub fn errors(source: &str) -> Vec<String> {
    parse(source).errors
}
