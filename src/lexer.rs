use std::str::FromStr;

use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token};

pub struct Lexer<'src> {
    pub source: &'src str,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Span {
        Span { start, end }
    }
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Lexer<'src> {
        Lexer {
            source,
            offset: 0,
        }
    }
    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }
    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }
    pub fn next_token(&mut self) -> Option<(Token<'src>, Span)> {
        let mut start;

        let token = 'outer: loop {

            let ch = self.peek()?;
            start = self.offset;

            if ch.is_whitespace() {
                self.next();
                continue;
            }

            if ch.is_alphabetic() || ch == '_' {
                while self
                    .peek()
                    .map_or(false, |ch| ch.is_alphanumeric() || ch == '_')
                {
                    self.next();
                }
                let ident = &self.source[start..self.offset];
                break Keyword::from_str(ident)
                    .map(|keyword| Token::Keyword(keyword))
                    .unwrap_or(Token::Ident(ident));
            }

            if ch.is_numeric() {
                let mut value = 0;
                while let Some(digit) = self.peek().and_then(|ch| ch.to_digit(10)) {
                    value = (value * 10) + digit as i64;
                    self.next();
                }
                if self.peek().map_or(false, |ch| ch == '.') {
                    self.next();
                    let mut value = value as f64;
                    let mut div = 1.0;
                    while let Some(digit) = self.peek().and_then(|ch| ch.to_digit(10)) {
                        div *= 0.1;
                        value += digit as f64 * div;
                        self.next();
                    }
                    break Token::Float(value);
                } else {
                    break Token::Int(value);
                }
            }

            for symbol in Symbol::iter() {
                if self.source[self.offset..].starts_with(symbol.as_str()) {
                    self.offset += symbol.as_str().len();
                    break 'outer Token::Symbol(symbol);
                }
            }

            panic!("unexpected character '{}'", ch);
        };
        let end = self.offset as u32;
        let span = Span { start: start as u32, end };
        Some((token, span))
    }
}
