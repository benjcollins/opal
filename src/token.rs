use std::borrow::Cow;

use strum::{EnumIter, EnumString};

#[derive(Debug, Clone)]
pub enum Token<'s> {
    Ident(&'s str),
    Int(i64),
    Float(f64),
    Keyword(Keyword),
    Symbol(Symbol),
    String(Cow<'s, str>),
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    Keyword(Keyword),
    Symbol(Symbol),
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Fun,
    Let,
    True,
    False,
    Module,
    Mut,
    Return,
    If,
    Else,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Symbol {
    RightArrow,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Comma,
    DoubleEqual,
    Equal,
    Plus,
    Star,
    Percent,
    Slash,
    Minus,
    Colon,
    NotEqual,
    LessEqual,
    Less,
    GreaterEqual,
    Greater,
}

impl Symbol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenBrace => "{",
            Symbol::CloseBrace => "}",
            Symbol::Semicolon => ";",
            Symbol::Comma => ",",
            Symbol::Equal => "=",
            Symbol::Plus => "+",
            Symbol::Star => "*",
            Symbol::Percent => "%",
            Symbol::Slash => "/",
            Symbol::Minus => "-",
            Symbol::Colon => ":",
            Symbol::RightArrow => "->",
            Symbol::DoubleEqual => "==",
            Symbol::Less => "<",
            Symbol::LessEqual => "<=",
            Symbol::Greater => ">",
            Symbol::GreaterEqual => ">=",
            Symbol::NotEqual => "!=",
        }
    }
}

pub trait TokenType: Clone {
    type Contents<'s>;

    fn matches<'s>(&self, token: Token<'s>) -> Result<Self::Contents<'s>, Token<'s>>;
    fn kind(&self) -> TokenKind;
}

impl TokenType for Symbol {
    type Contents<'s> = ();

    fn matches<'s>(&self, token: Token<'s>) -> Result<(), Token<'s>> {
        match token {
            Token::Symbol(s) if s == *self => Ok(()),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Symbol(*self)
    }
}

impl TokenType for Keyword {
    type Contents<'s> = ();

    fn matches<'s>(&self, token: Token<'s>) -> Result<(), Token<'s>> {
        match token {
            Token::Keyword(k) if k == *self => Ok(()),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Keyword(*self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ident;

impl TokenType for Ident {
    type Contents<'s> = &'s str;

    fn matches<'s>(&self, token: Token<'s>) -> Result<&'s str, Token<'s>> {
        match token {
            Token::Ident(ident) => Ok(ident),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Ident
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Int;

impl TokenType for Int {
    type Contents<'s> = i64;

    fn matches<'s>(&self, token: Token<'s>) -> Result<i64, Token<'s>> {
        match token {
            Token::Int(value) => Ok(value),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Int
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Float;

impl TokenType for Float {
    type Contents<'s> = f64;

    fn matches<'s>(&self, token: Token<'s>) -> Result<f64, Token<'s>> {
        match token {
            Token::Float(value) => Ok(value),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Float
    }
}
