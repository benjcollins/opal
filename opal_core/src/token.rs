use strum::{EnumIter, IntoStaticStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Ident(String),
    Int(i64),
    Str(String),
    Keyword(Keyword),
    Symbol(Symbol),
    Invalid(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Var,
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
    Equals,
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
        }
    }
}
