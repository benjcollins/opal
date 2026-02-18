use std::{borrow::Cow, fmt};

use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone)]
pub enum Token<'s> {
    Ident(&'s str),
    Int(i64),
    Float(f64),
    Keyword(Keyword),
    Symbol(Symbol),
    String(Cow<'s, str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    Keyword(Keyword),
    Symbol(Symbol),
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Hash, Display)]
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
    While,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Hash)]
pub enum Symbol {
    RightArrow,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Comma,
    DoubleEquals,
    Equals,
    Plus,
    Star,
    Percent,
    Slash,
    Minus,
    Colon,
    BangEquals,
    LessEquals,
    Less,
    GreaterEquals,
    Greater,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    PercentEquals,
    DoubleAmpersand,
    DoublePipe,
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
            Symbol::Equals => "=",
            Symbol::Plus => "+",
            Symbol::Star => "*",
            Symbol::Percent => "%",
            Symbol::Slash => "/",
            Symbol::Minus => "-",
            Symbol::Colon => ":",
            Symbol::RightArrow => "->",
            Symbol::DoubleEquals => "==",
            Symbol::Less => "<",
            Symbol::LessEquals => "<=",
            Symbol::Greater => ">",
            Symbol::GreaterEquals => ">=",
            Symbol::BangEquals => "!=",
            Symbol::PlusEquals => "+=",
            Symbol::MinusEquals => "-=",
            Symbol::StarEquals => "*=",
            Symbol::SlashEquals => "/=",
            Symbol::PercentEquals => "%=",
            Symbol::DoubleAmpersand => "&&",
            Symbol::DoublePipe => "||",
        }
    }
}

impl<'s> Token<'s> {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Ident(_) => TokenKind::Ident,
            Token::Int(_) => TokenKind::Int,
            Token::Float(_) => TokenKind::Float,
            &Token::Keyword(keyword) => TokenKind::Keyword(keyword),
            &Token::Symbol(symbol) => TokenKind::Symbol(symbol),
            Token::String(_) => TokenKind::String,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Int => write!(f, "integer literal"),
            TokenKind::Float => write!(f, "float literal"),
            TokenKind::Keyword(keyword) => write!(f, "'{}'", keyword),
            TokenKind::Symbol(symbol) => write!(f, "'{}'", symbol.as_str()),
            TokenKind::String => write!(f, "string literal"),
        }
    }
}

pub trait TokenMatcher: Clone {
    type Contents<'s>;

    fn matches<'s>(&self, token: Token<'s>) -> Result<Self::Contents<'s>, Token<'s>>;
    fn kind(&self) -> TokenKind;
}

impl TokenMatcher for Symbol {
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

impl TokenMatcher for Keyword {
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

impl TokenMatcher for Ident {
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

impl TokenMatcher for Int {
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

impl TokenMatcher for Float {
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
