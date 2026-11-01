//! VHDL lexer.
//!
//! Produces a token stream from VHDL source. The tokenizer is hand-written
//! and does not depend on an external grammar, and it performs error
//! tolerant tokenization (comments and strings are handled even if the
//! source is malformed).

/// Token kinds produced by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Keyword such as `signal` or `package`.
    Keyword,
    /// Identifier such as a signal name.
    Identifier,
    /// Literal such as a number, string, character or bit string.
    Literal,
    /// Punctuation such as `(` or `)`.
    Punctuation,
    /// Comment.
    Comment,
    /// Whitespace/newline.
    Whitespace,
}

impl TokenKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword",
            TokenKind::Identifier => "identifier",
            TokenKind::Literal => "literal",
            TokenKind::Punctuation => "punctuation",
            TokenKind::Comment => "comment",
            TokenKind::Whitespace => "whitespace",
        }
    }
}

/// A single lexed token with its kind, text, and source offsets.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(kind: TokenKind, value: String, start: usize, end: usize) -> Self {
        Self {
            kind,
            value,
            start,
            end,
        }
    }
}

static KEYWORDS: &[&str] = &[
    "alias",
    "package",
    "file",
    "entity",
    "architecture",
    "type",
    "subtype",
    "attribute",
    "to",
    "downto",
    "signal",
    "variable",
    "record",
    "array",
    "others",
    "process",
    "component",
    "shared",
    "constant",
    "port",
    "generic",
    "generate",
    "range",
    "map",
    "in",
    "inout",
    "of",
    "out",
    "configuration",
    "pure",
    "impure",
    "is",
    "begin",
    "end",
    "context",
    "wait",
    "until",
    "after",
    "report",
    "open",
    "exit",
    "assert",
    "next",
    "null",
    "force",
    "property",
    "release",
    "sequence",
    "transport",
    "unaffected",
    "select",
    "severity",
    "register",
    "reject",
    "postponed",
    "on",
    "new",
    "literal",
    "linkage",
    "inertial",
    "guarded",
    "group",
    "disconnect",
    "bus",
    "buffer",
    "body",
    "all",
    "block",
    "access",
    "function",
    "procedure",
    "return",
    "for",
    "loop",
    "while",
    "if",
    "elsif",
    "else",
    "case",
    "then",
    "when",
    "library",
    "use",
];

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_hex_digit(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F' | 'z' | 'Z')
}

/// Tokenizes the given source into a token stream.
pub fn tokenize(source: &str) -> Vec<Token> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];

        match c {
            ' ' | '\t' | '\r' | '\n' => {
                tokens.push(Token::new(TokenKind::Whitespace, String::new(), i, i + 1));
                i += 1;
            }
            '-' if chars.get(i + 1) == Some(&'-') => {
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                tokens.push(Token::new(
                    TokenKind::Comment,
                    source[start..i].to_string(),
                    start,
                    i,
                ));
            }
            '"' => {
                let start = i;
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        if i + 1 < chars.len() {
                            s.push(chars[i + 1]);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                i += 1; // closing quote
                tokens.push(Token::new(TokenKind::Literal, s, start, i));
            }
            '\'' => {
                let start = i;
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '\'' {
                    if chars[i] == '\\' {
                        if i + 1 < chars.len() {
                            s.push(chars[i + 1]);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                i += 1; // closing quote
                tokens.push(Token::new(TokenKind::Literal, s, start, i));
            }
            'x' | 'X' if matches!(chars.get(i + 1), Some(ch) if is_hex_digit(*ch)) => {
                let start = i;
                i += 2;
                let mut s = String::new();
                while i < chars.len() && is_hex_digit(chars[i]) {
                    s.push(chars[i]);
                    i += 1;
                }
                i += 1; // closing quote
                tokens.push(Token::new(TokenKind::Literal, s, start, i));
            }
            '0'..='9' => {
                let start = i;
                let mut j = i;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if j < chars.len() && chars[j] == '.' {
                    let mut k = j + 1;
                    while k < chars.len() && chars[k].is_ascii_digit() {
                        k += 1;
                    }
                    j = k;
                }
                if j < chars.len() && (chars[j] == 'e' || chars[j] == 'E') {
                    let mut n = j + 1;
                    if n < chars.len() && chars[n].is_ascii_digit() {
                        while n < chars.len() && chars[n].is_ascii_digit() {
                            n += 1;
                        }
                    }
                    j = n;
                }
                let value = source[start..j].to_string();
                i = j;
                tokens.push(Token::new(TokenKind::Literal, value, start, i));
            }
            _ => {
                // Identifier/digit run: consume all consecutive ident/digit
                // characters as one token (keyword or identifier).
                if is_ident_char(chars[i]) || chars[i].is_ascii_digit() {
                    let start = i;
                    while i < chars.len() && (is_ident_char(chars[i]) || chars[i].is_ascii_digit())
                    {
                        i += 1;
                    }
                    let value = source[start..i].to_string();
                    let kind = KEYWORDS
                        .iter()
                        .any(|kw| kw.eq_ignore_ascii_case(&value))
                        .then_some(TokenKind::Keyword)
                        .unwrap_or(TokenKind::Identifier);
                    tokens.push(Token::new(kind, value, start, i));
                } else {
                    // Non-ident/non-digit character (punctuation such as
                    // `(`, `)`, `;`, `=`, etc.). Emit it as a punctuation
                    // token and advance the index by one so the loop always
                    // makes progress.
                    let start = i;
                    let c = chars[i];
                    tokens.push(Token::new(
                        TokenKind::Punctuation,
                        c.to_string(),
                        start,
                        start + 1,
                    ));
                    i += 1;
                }
            }
        }
    }

    tokens
}

impl Token {
    pub fn kind_str(&self) -> &'static str {
        self.kind.as_str()
    }
}
