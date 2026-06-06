use strum::{EnumIter, IntoStaticStr};

pub enum Token {
    Ident(String),
    Int(i64),
    Str(String),
    Keyword(Keyword),
    Symbol(Symbol),
    Invalid(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
pub enum Keyword {
    Let,
    Fun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Symbol {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Comma,
    Semicolon,
}

impl Symbol {
    pub fn to_str(&self) -> &'static str {
        match self {
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenBrace => "{",
            Symbol::CloseBrace => "}",
            Symbol::Comma => ",",
            Symbol::Semicolon => ";",
        }
    }
}
