use strum::{EnumIter, IntoStaticStr};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    Keyword(Keyword),
    Symbol(Symbol),
    Invalid(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Var,
    Fun,
    True,
    False,
    Module,
    Return,
    NoReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Symbol {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Comma,
    Semicolon,
    Equals,
    Slash,
    Asterisk,
    Minus,
    Plus,
    Colon,
    Arrow,
}

impl Symbol {
    pub fn to_str(&self) -> &'static str {
        match self {
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenBrace => "{",
            Symbol::CloseBrace => "}",
            Symbol::Comma => ",",
            Symbol::Equals => "=",
            Symbol::Semicolon => ";",
            Symbol::Slash => "/",
            Symbol::Asterisk => "*",
            Symbol::Minus => "-",
            Symbol::Plus => "+",
            Symbol::Colon => ":",
            Symbol::Arrow => "->",
        }
    }
}
