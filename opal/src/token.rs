use std::fmt;

use strum::{Display, EnumIter, EnumString};

use crate::intern::InternedStr;

#[derive(Debug, Clone)]
pub enum Token {
    Ident(InternedStr),
    Int(i64),
    Float(f64),
    Keyword(Keyword),
    Symbol(Symbol),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    Keyword(Keyword),
    Symbol(Symbol),
    Str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Fun,
    Let,
    True,
    False,
    Module,
    Return,
    If,
    Else,
    While,
    Break,
    Continue,
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
    OpenBracket,
    CloseBracket,
    Ampersand,
    Pipe,
    Caret,
    AmpersandEquals,
    PipeEquals,
    CaretEquals,
    ColonEquals,
    DoubleLess,
    DoubleGreater,
    DoubleLessEquals,
    DoubleGreaterEquals,
    Tilde,
    Bang,
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
            Symbol::OpenBracket => "[",
            Symbol::CloseBracket => "]",
            Symbol::Ampersand => "&",
            Symbol::Pipe => "|",
            Symbol::Caret => "^",
            Symbol::AmpersandEquals => "&=",
            Symbol::PipeEquals => "|=",
            Symbol::CaretEquals => "^=",
            Symbol::ColonEquals => ":=",
            Symbol::DoubleLess => "<<",
            Symbol::DoubleGreater => ">>",
            Symbol::DoubleLessEquals => "<<=",
            Symbol::DoubleGreaterEquals => ">>=",
            Symbol::Tilde => "~",
            Symbol::Bang => "!",
        }
    }
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Ident(_) => TokenKind::Ident,
            Token::Int(_) => TokenKind::Int,
            Token::Float(_) => TokenKind::Float,
            &Token::Keyword(keyword) => TokenKind::Keyword(keyword),
            &Token::Symbol(symbol) => TokenKind::Symbol(symbol),
            Token::Str(_) => TokenKind::Str,
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
            TokenKind::Str => write!(f, "string literal"),
        }
    }
}

pub trait TokenMatcher: Clone {
    type Contents;

    fn matches(&self, token: Token) -> Result<Self::Contents, Token>;
    fn kind(&self) -> TokenKind;
}

impl TokenMatcher for Symbol {
    type Contents = ();

    fn matches(&self, token: Token) -> Result<(), Token> {
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
    type Contents = ();

    fn matches(&self, token: Token) -> Result<(), Token> {
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
    type Contents = InternedStr;

    fn matches<'s>(&self, token: Token) -> Result<InternedStr, Token> {
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
    type Contents = i64;

    fn matches<'s>(&self, token: Token) -> Result<i64, Token> {
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
    type Contents = f64;

    fn matches(&self, token: Token) -> Result<f64, Token> {
        match token {
            Token::Float(value) => Ok(value),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Float
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Str;

impl TokenMatcher for Str {
    type Contents = String;

    fn matches(&self, token: Token) -> Result<Self::Contents, Token> {
        match token {
            Token::Str(value) => Ok(value),
            token => Err(token),
        }
    }

    fn kind(&self) -> TokenKind {
        TokenKind::Str
    }
}
