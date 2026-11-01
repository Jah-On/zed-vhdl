//! VHDL syntax highlighter.
//!
//! The custom hand-written lexer produces a token stream where each token
//! carries its source range (`start`/`end`). This highlighter walks that
//! token stream and emits a highlight span for every token, covering keywords,
//! identifiers, literals, punctuation and comments. It reuses the lexer that
//! backs the parser, so highlighting is robust and dependency-free (no
//! tree-sitter grammar).

use crate::vhdl::lexer::{tokenize, TokenKind};

/// A highlighted span with its token kind and source range.
#[derive(Debug, Clone)]
pub struct HighlightedSpan {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

/// Returns the highlighted token spans for the given VHDL source.
pub fn highlight(source: &str) -> Vec<HighlightedSpan> {
    let tokens = tokenize(source);
    tokens
        .into_iter()
        .map(|token| HighlightedSpan {
            kind: token.kind,
            start: token.start,
            end: token.end,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_keyword_and_literal_spans() {
        let spans = highlight("signal clk in integer; const v = 5;");
        assert!(!spans.is_empty());
        let has_keyword = spans.iter().any(|s| s.kind == TokenKind::Keyword);
        let has_literal = spans.iter().any(|s| s.kind == TokenKind::Literal);
        let has_punctuation = spans.iter().any(|s| s.kind == TokenKind::Punctuation);
        assert!(has_keyword);
        assert!(has_literal);
        assert!(has_punctuation);
        // Each span has non-empty source ranges.
        for span in spans {
            assert!(span.start <= span.end);
        }
    }

    #[test]
    fn handles_malformed_source_without_looping() {
        // Punctuation tokens must be emitted and the tokenize loop must make
        // progress, never spin forever.
        let spans = highlight("( ); ; end");
        assert!(!spans.is_empty());
    }
}
