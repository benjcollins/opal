use std::{cmp::Reverse, collections::HashMap, sync::LazyLock};

use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token};

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

static KEYWORD_MAP: LazyLock<HashMap<&'static str, Keyword>> =
    LazyLock::new(|| HashMap::from_iter(Keyword::iter().map(|keyword| (keyword.into(), keyword))));

static SYMBOLS_SORTED: LazyLock<Vec<Symbol>> = LazyLock::new(|| {
    let mut symbols = Vec::from_iter(Symbol::iter());
    symbols.sort_by_key(|symbol| Reverse(symbol.to_str()));
    symbols
});

impl Span {
    pub fn to(&self, other: &Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    pub fn advance_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.position += ch.len_utf8();
        } else {
            panic!("end of input reached");
        }
    }

    pub fn consume_str(&mut self, expected: &str) -> bool {
        if self.input[self.position..].starts_with(expected) {
            self.position += expected.len();
            true
        } else {
            false
        }
    }

    pub fn consume_block_comment(&mut self) {
        while !self.consume_str("*/") && self.peek_char().is_some() {
            if self.consume_str("/*") {
                self.consume_block_comment();
            }
            self.advance_char();
        }
    }

    pub fn consume_line_comment(&mut self) {}

    pub fn next_token(&mut self) -> Option<(Token, Span)> {
        let mut start;

        let token = 'outer: loop {
            start = self.position;

            let ch = self.peek_char()?;

            if ch.is_whitespace() {
                self.advance_char();
                continue;
            }

            if self.consume_str("//") {
                while self.peek_char().is_some_and(|ch| ch != '\n') {
                    self.advance_char();
                }
                continue;
            }

            if self.consume_str("/*") {
                self.consume_block_comment();
                continue;
            }

            if ch == '"' {
                self.advance_char();
                let mut value = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch == '"' {
                        self.advance_char();
                        break;
                    } else if ch == '\\' {
                        self.advance_char();
                        if let Some(escaped) = self.peek_char() {
                            value.push(match escaped {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                '\\' => '\\',
                                '"' => '"',
                                other => other,
                            });
                            self.advance_char();
                        } else {
                            break;
                        }
                    } else {
                        value.push(ch);
                        self.advance_char();
                    }
                }
                break Token::String(value);
            }

            if is_ident_start(ch) {
                self.advance_char();
                while self.peek_char().is_some_and(is_ident_continue) {
                    self.advance_char();
                }
                let ident = &self.input[start..self.position];
                if let Some(keyword) = KEYWORD_MAP.get(ident) {
                    break Token::Keyword(*keyword);
                } else {
                    break Token::Ident(ident.to_string());
                }
            }

            if ch.is_ascii_digit() {
                let mut value = 0;
                while let Some(digit) = self.peek_char().and_then(|ch| ch.to_digit(10)) {
                    self.advance_char();
                    value = value * 10 + digit as i64;
                }
                if self.peek_char() == Some('.') {
                    self.advance_char();
                    let mut fraction = 0.0;
                    let mut divisor = 1.0;
                    while let Some(digit) = self.peek_char().and_then(|ch| ch.to_digit(10)) {
                        self.advance_char();
                        fraction = fraction * 10.0 + digit as f64;
                        divisor *= 10.0;
                    }
                    break Token::Float(value as f64 + fraction / divisor);
                }
                break Token::Int(value);
            }

            for symbol in SYMBOLS_SORTED.iter().copied() {
                if self.consume_str(symbol.to_str()) {
                    break 'outer Token::Symbol(symbol);
                }
            }

            self.advance_char();
            break Token::Invalid(ch);
        };

        let end = self.position;

        Some((token, Span { start, end }))
    }
}

impl Iterator for Lexer<'_> {
    type Item = (Token, Span);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        token::{Keyword, Symbol, Token},
    };

    fn check(input: &str, expected_tokens: &[(Token, Span)]) {
        let tokens: Vec<_> = Lexer::new(input).collect();
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn example_test() {
        check("52", &[(Token::Int(52), Span { start: 0, end: 2 })]);
    }

    // Token type tests
    #[test]
    fn test_int_token() {
        check("42", &[(Token::Int(42), Span { start: 0, end: 2 })]);
    }

    #[test]
    fn test_float_token() {
        check("3.74", &[(Token::Float(3.74), Span { start: 0, end: 4 })]);
    }

    #[test]
    fn test_string_token() {
        check(
            "\"hello\"",
            &[(Token::String("hello".to_string()), Span { start: 0, end: 7 })],
        );
    }

    #[test]
    fn test_string_with_escapes() {
        check(
            "\"hello\\nworld\"",
            &[(Token::String("hello\nworld".to_string()), Span { start: 0, end: 14 })],
        );
    }

    #[test]
    fn test_ident_token() {
        check(
            "myvar",
            &[(Token::Ident("myvar".to_string()), Span { start: 0, end: 5 })],
        );
    }

    #[test]
    fn test_ident_with_underscore() {
        check(
            "_private",
            &[(Token::Ident("_private".to_string()), Span { start: 0, end: 8 })],
        );
    }

    // Keyword tests
    #[test]
    fn test_keyword_var() {
        check("var", &[(Token::Keyword(Keyword::Var), Span { start: 0, end: 3 })]);
    }

    #[test]
    fn test_keyword_func() {
        check("fun", &[(Token::Keyword(Keyword::Fun), Span { start: 0, end: 3 })]);
    }

    #[test]
    fn test_keyword_true() {
        check("true", &[(Token::Keyword(Keyword::True), Span { start: 0, end: 4 })]);
    }

    #[test]
    fn test_keyword_false() {
        check("false", &[(Token::Keyword(Keyword::False), Span { start: 0, end: 5 })]);
    }

    #[test]
    fn test_keyword_module() {
        check(
            "module",
            &[(Token::Keyword(Keyword::Module), Span { start: 0, end: 6 })],
        );
    }

    // Symbol tests
    #[test]
    fn test_open_paren() {
        check("(", &[(Token::Symbol(Symbol::OpenParen), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_close_paren() {
        check(")", &[(Token::Symbol(Symbol::CloseParen), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_open_brace() {
        check("{", &[(Token::Symbol(Symbol::OpenBrace), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_close_brace() {
        check("}", &[(Token::Symbol(Symbol::CloseBrace), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_comma() {
        check(",", &[(Token::Symbol(Symbol::Comma), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_semicolon() {
        check(";", &[(Token::Symbol(Symbol::Semicolon), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_equals() {
        check("=", &[(Token::Symbol(Symbol::Equals), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_plus() {
        check("+", &[(Token::Symbol(Symbol::Plus), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_minus() {
        check("-", &[(Token::Symbol(Symbol::Minus), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_asterisk() {
        check("*", &[(Token::Symbol(Symbol::Asterisk), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_slash() {
        check("/", &[(Token::Symbol(Symbol::Slash), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_colon() {
        check(":", &[(Token::Symbol(Symbol::Colon), Span { start: 0, end: 1 })]);
    }

    #[test]
    fn test_arrow() {
        check("->", &[(Token::Symbol(Symbol::Arrow), Span { start: 0, end: 2 })]);
    }

    // Comment tests
    #[test]
    fn test_line_comment() {
        check(
            "// this is a comment\n42",
            &[(Token::Int(42), Span { start: 21, end: 23 })],
        );
    }

    #[test]
    fn test_line_comment_at_end() {
        check("42 // comment", &[(Token::Int(42), Span { start: 0, end: 2 })]);
    }

    #[test]
    fn test_block_comment() {
        check("/* comment */ 42", &[(Token::Int(42), Span { start: 14, end: 16 })]);
    }

    #[test]
    fn test_nested_block_comment() {
        check(
            "/* outer /* inner */ outer */ 42",
            &[(Token::Int(42), Span { start: 30, end: 32 })],
        );
    }

    #[test]
    fn test_multiline_block_comment() {
        check(
            "/* line1\nline2\nline3 */ 42",
            &[(Token::Int(42), Span { start: 24, end: 26 })],
        );
    }

    // Whitespace tests
    #[test]
    fn test_spaces_ignored() {
        check(
            "42   100",
            &[
                (Token::Int(42), Span { start: 0, end: 2 }),
                (Token::Int(100), Span { start: 5, end: 8 }),
            ],
        );
    }

    #[test]
    fn test_tabs_ignored() {
        check(
            "42\t100",
            &[
                (Token::Int(42), Span { start: 0, end: 2 }),
                (Token::Int(100), Span { start: 3, end: 6 }),
            ],
        );
    }

    #[test]
    fn test_newlines_ignored() {
        check(
            "42\n100",
            &[
                (Token::Int(42), Span { start: 0, end: 2 }),
                (Token::Int(100), Span { start: 3, end: 6 }),
            ],
        );
    }

    #[test]
    fn test_mixed_whitespace() {
        check(
            "42  \t\n  100",
            &[
                (Token::Int(42), Span { start: 0, end: 2 }),
                (Token::Int(100), Span { start: 8, end: 11 }),
            ],
        );
    }

    // Complex tests
    #[test]
    fn test_expression_with_multiple_tokens() {
        check(
            "x + 42",
            &[
                (Token::Ident("x".to_string()), Span { start: 0, end: 1 }),
                (Token::Symbol(Symbol::Plus), Span { start: 2, end: 3 }),
                (Token::Int(42), Span { start: 4, end: 6 }),
            ],
        );
    }
}
