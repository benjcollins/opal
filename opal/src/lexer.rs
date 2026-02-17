use std::{cell::LazyCell, cmp::Reverse, str::FromStr};

use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token};

pub struct Lexer<'src> {
    pub source: &'src str,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

thread_local! {
    static SYMBOLS: LazyCell<&'static [Symbol]> = LazyCell::new(|| {
        let mut symbols = Vec::from_iter(Symbol::iter());
        symbols.sort_by_key(|a| Reverse(a.as_str().len()));
        symbols.leak()
    });
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Lexer<'src> {
        Lexer { source, offset: 0 }
    }
    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }
    fn consume(&mut self, s: &str) -> bool {
        if self.source[self.offset..].starts_with(s) {
            self.offset += s.len();
            true
        } else {
            false
        }
    }
    fn skip_line_comment(&mut self) {
        while !self.consume("\n") {
            if self.consume("/*") {
                self.skip_block_comment();
            }
            self.advance();
        }
    }
    fn skip_block_comment(&mut self) {
        while !self.consume("*/") {
            if self.consume("//") {
                self.skip_line_comment();
            }
            self.advance();
        }
    }
    pub fn next_token(&mut self) -> Option<(Token<'src>, Span)> {
        let mut start;

        let token = 'outer: loop {
            let ch = self.peek()?;
            start = self.offset;

            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            if ch.is_alphabetic() || ch == '_' {
                while self.peek().is_some_and(|ch| ch.is_alphanumeric() || ch == '_') {
                    self.advance();
                }
                let ident = &self.source[start..self.offset];
                break Keyword::from_str(ident)
                    .map(Token::Keyword)
                    .unwrap_or(Token::Ident(ident));
            }

            if ch.is_numeric() {
                let mut value = 0;
                while let Some(digit) = self.peek().and_then(|ch| ch.to_digit(10)) {
                    value = (value * 10) + digit as i64;
                    self.advance();
                }
                if self.peek().is_some_and(|ch| ch == '.') {
                    self.advance();
                    let mut value = value as f64;
                    let mut div = 1.0;
                    while let Some(digit) = self.peek().and_then(|ch| ch.to_digit(10)) {
                        div *= 0.1;
                        value += digit as f64 * div;
                        self.advance();
                    }
                    break Token::Float(value);
                } else {
                    break Token::Int(value);
                }
            }

            if self.consume("//") {
                self.skip_line_comment();
                continue;
            }

            if self.consume("/*") {
                self.skip_block_comment();
                continue;
            }

            if let Some(symbol) = SYMBOLS.with(|symbols| {
                for symbol in symbols.iter() {
                    if self.consume(symbol.as_str()) {
                        return Some(*symbol);
                    }
                }
                None
            }) {
                break 'outer Token::Symbol(symbol);
            }

            panic!("unexpected character '{}'", ch);
        };
        let end = self.offset;
        let span = Span { start, end };
        Some((token, span))
    }
}
