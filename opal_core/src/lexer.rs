use std::{collections::HashMap, sync::LazyLock};

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
    symbols.sort_by_key(|symbol| symbol.to_str());
    symbols
});

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
                while !self.consume_str("*/") && self.peek_char().is_some() {
                    self.advance_char();
                }
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
                break Token::Str(value);
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
