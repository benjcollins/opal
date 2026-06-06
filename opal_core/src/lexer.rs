use std::{collections::HashMap, sync::LazyLock};

use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token};

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

pub struct Span {
    start: usize,
    end: usize,
}

static KEYWORDS: LazyLock<HashMap<&'static str, Keyword>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for keyword in Keyword::iter() {
        map.insert(keyword.into(), keyword);
    }
    map
});

static SYMBOLS: LazyLock<HashMap<&'static str, Symbol>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for symbol in Symbol::iter() {
        map.insert(symbol.to_str(), symbol);
    }
    map
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

    pub fn next_token(&mut self) -> Option<(Token, Span)> {
        let mut start;

        let token = 'outer: loop {
            start = self.position;

            let ch = self.peek_char()?;

            if ch.is_whitespace() {
                self.advance_char();
                continue;
            }

            if is_ident_start(ch) {
                self.advance_char();
                while self.peek_char().is_some_and(is_ident_continue) {
                    self.advance_char();
                }
                let ident = &self.input[start..self.position];
                if let Some(keyword) = KEYWORDS.get(ident) {
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
                break Token::Int(value);
            }

            for symbol in SYMBOLS.iter() {
                if self.input[self.position..].starts_with(symbol.0) {
                    self.position += symbol.0.len();
                    break 'outer Token::Symbol(*symbol.1);
                }
            }
        };

        let end = self.position;

        Some((token, Span { start, end }))
    }
}
