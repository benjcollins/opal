use std::{collections::HashMap, fmt, mem::take, path::Path, vec};

use colored::Colorize;

use crate::{
    ast::{
        ArithOp, AssignOp, BitwiseOp, Block, CompOp, Else, EqualityOp, Expr, Fun, Generics, Ident, If, InfixOp, Lit,
        LogicalOp, Module, ModuleItem, PrefixOp, Stmt,
    },
    lexer::{Lexer, Span},
    token::{self, Float, Int, Keyword, Str, Symbol, Token, TokenKind, TokenMatcher},
};

pub struct Parser<'s, 'p> {
    pub path: Option<&'p Path>,
    pub lexer: Lexer<'s>,
    pub token: Option<(Token, Span)>,
    pub expected: Vec<TokenKind>,
    pub errors: Vec<ParseError<'s, 'p>>,
    pub recover: HashMap<TokenKind, u32>,
}

#[derive(Debug)]
pub struct ParseError<'s, 'p> {
    path: Option<&'p Path>,
    token: Option<(Token, Span)>,
    expected: Vec<TokenKind>,
    line: u32,
    column: u32,
    source: &'s str,
}

impl fmt::Display for ParseError<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self
            .path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<source>".to_string());
        write!(
            f,
            "{}: {}:{}:{} ",
            "syntax error".red().bold(),
            path.cyan(),
            self.line.to_string().cyan(),
            self.column.to_string().cyan()
        )?;
        if let Some((_, span)) = &self.token {
            writeln!(
                f,
                "unexpected token {}",
                format!("'{}'", &self.source[span.start..span.end]).yellow()
            )?;
        } else {
            writeln!(f, "unexpected {}", "end of file".yellow())?;
        }
        write!(f, "  expected one of: ")?;
        for (i, expected) in self.expected.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", expected.to_string().green())?;
        }
        Ok(())
    }
}

type Prec = u8;

pub struct Recovered(Option<TokenKind>);

fn line_column(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 1;
    let mut column = 1;
    for c in source[..offset].chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

const INFIX_OPS: &[(Symbol, InfixOp, Prec)] = &[
    (Symbol::Star, InfixOp::Arith(ArithOp::Multiply), 10),
    (Symbol::Slash, InfixOp::Arith(ArithOp::Divide), 10),
    (Symbol::Percent, InfixOp::Arith(ArithOp::Modulus), 10),
    //
    (Symbol::Plus, InfixOp::Arith(ArithOp::Add), 9),
    (Symbol::Minus, InfixOp::Arith(ArithOp::Subtract), 9),
    //
    (Symbol::DoubleLess, InfixOp::Bitwise(BitwiseOp::ShiftLeft), 8),
    (Symbol::DoubleGreater, InfixOp::Bitwise(BitwiseOp::ShiftRight), 8),
    //
    (Symbol::Less, InfixOp::Comp(CompOp::Less), 7),
    (Symbol::Greater, InfixOp::Comp(CompOp::Greater), 7),
    (Symbol::LessEquals, InfixOp::Comp(CompOp::LessEqual), 7),
    (Symbol::GreaterEquals, InfixOp::Comp(CompOp::GreaterEqual), 7),
    //
    (Symbol::DoubleEquals, InfixOp::Equality(EqualityOp::Equal), 6),
    (Symbol::BangEquals, InfixOp::Equality(EqualityOp::NotEqual), 6),
    //
    (Symbol::Ampersand, InfixOp::Bitwise(BitwiseOp::And), 5),
    (Symbol::Caret, InfixOp::Bitwise(BitwiseOp::XOr), 4),
    (Symbol::Pipe, InfixOp::Bitwise(BitwiseOp::Or), 3),
    //
    (Symbol::DoubleAmpersand, InfixOp::Logical(LogicalOp::And), 2),
    (Symbol::DoublePipe, InfixOp::Logical(LogicalOp::Or), 1),
];

const PREFIX_OPS: &[(Symbol, Prec, PrefixOp)] = &[
    (Symbol::Minus, 11, PrefixOp::Negative),
    (Symbol::Plus, 11, PrefixOp::Positive),
    (Symbol::Tilde, 11, PrefixOp::BitwiseNot),
    (Symbol::Bang, 11, PrefixOp::LogicalNot),
];

const POSTFIX_OPS: &[(Symbol, Prec, fn(&mut Parser, Expr, Span) -> Result<Expr, Recovered>)] = &[
    (Symbol::OpenParen, 12, |self_, expr, open_paren| {
        self_.parse_expr_call(expr, open_paren)
    }),
    (Symbol::OpenBracket, 12, |self_, expr, open_bracket| {
        self_.parse_expr_index(expr, open_bracket)
    }),
];

const ASSIGN_OPS: &[(AssignOp, Symbol)] = &[
    (AssignOp::Assign, Symbol::ColonEquals),
    (AssignOp::Arith(ArithOp::Add), Symbol::PlusEquals),
    (AssignOp::Arith(ArithOp::Subtract), Symbol::MinusEquals),
    (AssignOp::Arith(ArithOp::Multiply), Symbol::StarEquals),
    (AssignOp::Arith(ArithOp::Divide), Symbol::SlashEquals),
    (AssignOp::Arith(ArithOp::Modulus), Symbol::PercentEquals),
    //
    (AssignOp::Bitwise(BitwiseOp::And), Symbol::AmpersandEquals),
    (AssignOp::Bitwise(BitwiseOp::Or), Symbol::PipeEquals),
    (AssignOp::Bitwise(BitwiseOp::XOr), Symbol::CaretEquals),
    (AssignOp::Bitwise(BitwiseOp::ShiftLeft), Symbol::DoubleLessEquals),
    (AssignOp::Bitwise(BitwiseOp::ShiftRight), Symbol::DoubleGreaterEquals),
];

#[derive(Debug, Clone)]
pub struct Separated<T>(Vec<(T, Option<Span>)>);

impl<T> Separated<T> {
    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.0.iter().map(|(item, _)| item)
    }
}

impl<'s, 'p> Parser<'s, 'p> {
    pub fn new(source: &'s str, path: Option<&'p Path>) -> Parser<'s, 'p> {
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token();
        Parser {
            path,
            lexer,
            token,
            expected: Vec::new(),
            errors: Vec::new(),
            recover: HashMap::new(),
        }
    }
    fn expect<T>(&mut self, matcher: impl TokenMatcher<Contents = T>) -> Result<(T, Span), Recovered> {
        match self.consume(matcher) {
            Some(token) => Ok(token),
            None => Err(self.error()),
        }
    }
    fn consume<T>(&mut self, matcher: impl TokenMatcher<Contents = T>) -> Option<(T, Span)> {
        let (token, span) = self.token.take()?;
        match matcher.matches(token) {
            Ok(item) => {
                self.token = self.lexer.next_token();
                self.expected.truncate(0);
                Some((item, span))
            }
            Err(token) => {
                self.expected.push(matcher.kind());
                self.token = Some((token, span));
                None
            }
        }
    }
    fn advance(&mut self, token: impl TokenMatcher) -> bool {
        self.consume(token).is_some()
    }
    fn error(&mut self) -> Recovered {
        let span = self.token.as_ref().map(|(_, span)| *span).unwrap_or(Span {
            start: self.lexer.source.len(),
            end: self.lexer.source.len(),
        });
        let (line, column) = line_column(self.lexer.source, span.start);
        self.errors.push(ParseError {
            token: self.token.clone(),
            expected: take(&mut self.expected),
            path: self.path,
            line,
            column,
            source: self.lexer.source,
        });
        loop {
            let Some((token, _)) = &self.token else {
                return Recovered(None);
            };
            let kind = token.kind();
            if self.recover.get(&kind).copied().unwrap_or(0) > 0 {
                self.token = self.lexer.next_token();
                return Recovered(Some(kind));
            };
            self.token = self.lexer.next_token();
        }
    }
    fn recover<T>(
        &mut self,
        token: TokenKind,
        mut parse_fn: impl FnMut(&mut Parser) -> Result<T, Recovered>,
    ) -> Result<Option<T>, Recovered> {
        *self.recover.entry(token).or_insert(0) += 1;
        let res = parse_fn(self);
        *self.recover.get_mut(&token).unwrap() -= 1;
        match res {
            Ok(value) => Ok(Some(value)),
            Err(Recovered(Some(kind))) if token == kind => Ok(None),
            Err(Recovered(r)) => Err(Recovered(r)),
        }
    }
    fn parse_value(&mut self) -> Result<Expr, Recovered> {
        if let Some((value, span)) = self.consume(Int) {
            return Ok(Expr::Lit(Lit::Int { value, span }));
        }
        if let Some((value, span)) = self.consume(Float) {
            return Ok(Expr::Lit(Lit::Float { value, span }));
        }
        if let Some((value, span)) = self.consume(Str) {
            return Ok(Expr::Lit(Lit::Str { value, span }));
        }
        if let Some((_, fun)) = self.consume(Keyword::Fun) {
            let (_, open_paren) = self.expect(Symbol::OpenParen)?;
            let (params, close_paren) =
                self.parse_separated(Symbol::Comma, Symbol::CloseParen, |_self| _self.parse_expr(0))?;
            let returns = if let Some((_, arrow)) = self.consume(Symbol::RightArrow) {
                Some((arrow, Box::new(self.parse_expr(0)?)))
            } else {
                None
            };
            return Ok(Expr::FunType {
                fun,
                open_paren,
                params,
                close_paren,
                returns,
            });
        }
        if let Some((_, open_bracket)) = self.consume(Symbol::OpenBracket) {
            if let Some((_, close_bracket)) = self.consume(Symbol::CloseBracket) {
                return Ok(Expr::ListElements {
                    open_bracket,
                    elements: Separated(vec![]),
                    close_bracket,
                });
            }
            let first = self.parse_expr(0)?;
            if let Some((_, semicolon)) = self.consume(Symbol::Semicolon) {
                let len = self.parse_expr(0)?;
                let (_, close_bracket) = self.expect(Symbol::CloseBracket)?;
                return Ok(Expr::ListDefaultLength {
                    open_bracket,
                    default: Box::new(first),
                    semicolon,
                    length: Box::new(len),
                    close_bracket,
                });
            } else if let Some((_, comma)) = self.consume(Symbol::Comma) {
                let mut elements = vec![(first, Some(comma))];
                let close_bracket = self.parse_separated_into(
                    Symbol::Comma,
                    Symbol::CloseBracket,
                    |self_| self_.parse_expr(0),
                    &mut elements,
                )?;
                return Ok(Expr::ListElements {
                    open_bracket,
                    elements: Separated(elements),
                    close_bracket,
                });
            } else {
                return Err(self.error());
            }
        }
        if let Some((_, open_paren)) = self.consume(Symbol::OpenParen) {
            if let Some((_, close_paren)) = self.consume(Symbol::CloseParen) {
                return Ok(Expr::Lit(Lit::Unit {
                    open_paren,
                    close_paren,
                }));
            }
            let expr = self.parse_expr(0)?;
            let (_, close_paren) = self.expect(Symbol::CloseParen)?;
            return Ok(Expr::Paren {
                open_paren,
                expr: Box::new(expr),
                close_paren,
            });
        }
        if let Some((_, span)) = self.consume(Keyword::True) {
            return Ok(Expr::Lit(Lit::Bool { value: true, span }));
        }
        if let Some((_, span)) = self.consume(Keyword::False) {
            return Ok(Expr::Lit(Lit::Bool { value: false, span }));
        }
        if let Some((str, span)) = self.consume(token::Ident) {
            return Ok(Expr::Var(Ident { str, span }));
        }
        Err(self.error())
    }
    pub fn parse_expr_call(&mut self, expr: Expr, open_paren: Span) -> Result<Expr, Recovered> {
        let (args, close_paren) =
            self.parse_separated(Symbol::Comma, Symbol::CloseParen, |self_| self_.parse_expr(0))?;
        Ok(Expr::Call {
            fun: Box::new(expr),
            open_paren,
            args,
            close_paren,
        })
    }
    pub fn parse_expr_index(&mut self, expr: Expr, open_bracket: Span) -> Result<Expr, Recovered> {
        let index = self.parse_expr(0)?;
        let (_, close_bracket) = self.expect(Symbol::CloseBracket)?;
        Ok(Expr::Index {
            expr: Box::new(expr),
            open_bracket,
            index: Box::new(index),
            close_bracket,
        })
    }
    pub fn parse_expr(&mut self, prec: Prec) -> Result<Expr, Recovered> {
        let mut left = None;
        for (symbol, op_prec, op) in PREFIX_OPS.iter().copied() {
            if let Some((_, op_span)) = self.consume(symbol) {
                let expr = Box::new(self.parse_expr(op_prec)?);
                left = Some(Expr::Prefix { op, op_span, expr });
                break;
            }
        }
        let mut left = if let Some(left) = left {
            left
        } else {
            self.parse_value()?
        };
        'outer: loop {
            for (symbol, op_prec, parse_fn) in POSTFIX_OPS {
                if *op_prec > prec
                    && let Some((_, op_span)) = self.consume(*symbol)
                {
                    left = parse_fn(self, left, op_span)?;
                }
            }
            for (symbol, infix_op, op_prec) in INFIX_OPS {
                if *op_prec > prec
                    && let Some((_, op_span)) = self.consume(*symbol)
                {
                    let right = self.parse_expr(*op_prec)?;
                    left = Expr::Infix {
                        left: Box::new(left),
                        op: *infix_op,
                        op_span,
                        right: Box::new(right),
                    };
                    continue 'outer;
                }
            }
            break;
        }
        Ok(left)
    }
    pub fn parse_if(&mut self) -> Result<If, Recovered> {
        self.expect(Symbol::OpenParen)?;
        let cond = self.parse_expr(0)?;
        self.expect(Symbol::CloseParen)?;
        let if_block = self.parse_block()?;
        let else_ = if self.advance(Keyword::Else) {
            if self.advance(Keyword::If) {
                Else::If(Box::new(self.parse_if()?))
            } else {
                Else::Block(self.parse_block()?)
            }
        } else {
            Else::Nothing
        };
        Ok(If { cond, if_block, else_ })
    }
    pub fn parse_stmt(&mut self) -> Result<Stmt, Recovered> {
        if let Some((_, var)) = self.consume(Keyword::Var) {
            let (str, span) = self.expect(token::Ident)?;
            let name = Ident { str, span };
            let ty = if let Some((_, colon)) = self.consume(Symbol::Colon) {
                Some((colon, self.parse_expr(0)?))
            } else {
                None
            };
            let (_, equals) = self.expect(Symbol::Equals)?;
            let expr = self.parse_expr(0)?;
            let (_, semicolon) = self.expect(Symbol::Semicolon)?;
            return Ok(Stmt::Var {
                var,
                name,
                ty,
                equals,
                expr,
                semicolon,
            });
        }
        if self.advance(Keyword::Return) {
            let expr = if !self.advance(Symbol::Semicolon) {
                let expr = self.parse_expr(0)?;
                self.expect(Symbol::Semicolon)?;
                Some(expr)
            } else {
                None
            };
            return Ok(Stmt::Return(expr));
        }
        if self.advance(Keyword::If) {
            return Ok(Stmt::If(self.parse_if()?));
        }
        if self.advance(Keyword::Break) {
            self.expect(Symbol::Semicolon)?;
            return Ok(Stmt::Break);
        }
        if self.advance(Keyword::Continue) {
            self.expect(Symbol::Semicolon)?;
            return Ok(Stmt::Continue);
        }
        if self.advance(Keyword::While) {
            self.expect(Symbol::OpenParen)?;
            let cond = self.parse_expr(0)?;
            self.expect(Symbol::CloseParen)?;
            let block = self.parse_block()?;
            return Ok(Stmt::While { cond, block });
        }

        let expr = self.parse_expr(0)?;
        for &(op, symbol) in ASSIGN_OPS {
            if let Some((_, op_span)) = self.consume(symbol) {
                let src = self.parse_expr(0)?;
                let (_, semicolon) = self.expect(Symbol::Semicolon)?;
                return Ok(Stmt::Assign {
                    dst: expr,
                    op,
                    op_span,
                    src,
                    semicolon,
                });
            }
        }
        self.expect(Symbol::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }
    pub fn parse_block(&mut self) -> Result<Block, Recovered> {
        self.expect(Symbol::OpenBrace)?;
        let mut stmts = vec![];
        self.recover(TokenKind::Symbol(Symbol::CloseBrace), |self_| {
            while !self_.advance(Symbol::CloseBrace) {
                let stmt = self_.recover(TokenKind::Symbol(Symbol::Semicolon), |self_| self_.parse_stmt())?;
                if let Some(stmt) = stmt {
                    stmts.push(stmt);
                }
            }
            Ok(())
        })?;
        Ok(Block { stmts })
    }
    pub fn parse_separated_into<T, SEP: TokenMatcher, TERM: TokenMatcher>(
        &mut self,
        sep: SEP,
        term: TERM,
        parse_fn: impl Fn(&mut Parser) -> Result<T, Recovered>,
        items: &mut Vec<(T, Option<Span>)>,
    ) -> Result<Span, Recovered> {
        Ok(loop {
            if let Some((_, term)) = self.consume(term.clone()) {
                break term;
            }
            let item = parse_fn(self)?;
            let sep = self.consume(sep.clone()).map(|(_, sep)| sep);
            items.push((item, sep));
            if sep.is_none() {
                let (_, term) = self.expect(term)?;
                break term;
            }
        })
    }
    pub fn parse_separated<T, SEP: TokenMatcher, TERM: TokenMatcher>(
        &mut self,
        sep: SEP,
        term: TERM,
        parse_fn: impl Fn(&mut Parser) -> Result<T, Recovered>,
    ) -> Result<(Separated<T>, Span), Recovered> {
        let mut items = vec![];
        let term = self.parse_separated_into(sep, term, parse_fn, &mut items)?;
        Ok((Separated(items), term))
    }
    pub fn parse_module_item(&mut self) -> Result<ModuleItem, Recovered> {
        if self.advance(Keyword::Fun) {
            let (str, span) = self.expect(token::Ident)?;
            let name = Ident { str, span };
            let generics = if let Some((_, open_bracket)) = self.consume(Symbol::OpenBracket) {
                let (args, close_bracket) = self.parse_separated(Symbol::Comma, Symbol::CloseBracket, |self_| {
                    let (str, span) = self_.expect(token::Ident)?;
                    Ok(Ident { str, span })
                })?;
                Some(Generics {
                    open_bracket,
                    args,
                    close_bracket,
                })
            } else {
                None
            };
            self.expect(Symbol::OpenParen)?;
            let (params, _) = self.parse_separated(Symbol::Comma, Symbol::CloseParen, |self_| {
                let (str, span) = self_.expect(token::Ident)?;
                let name = Ident { str, span };
                self_.expect(Symbol::Colon)?;
                let ty = self_.parse_expr(0)?;
                Ok((name, ty))
            })?;
            let returns = if self.advance(Symbol::RightArrow) {
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            let block = self.parse_block()?;
            Ok(ModuleItem::Fun(Fun {
                name,
                generics,
                params,
                returns,
                block,
            }))
        } else {
            Err(self.error())
        }
    }
    fn parse_module(&mut self) -> Result<Module, Recovered> {
        self.expect(Keyword::Module)?;
        let (str, span) = self.expect(token::Ident)?;
        let name = Ident { str, span };
        self.expect(Symbol::Semicolon)?;

        let mut items = vec![];
        while self.token.is_some() {
            items.push(self.parse_module_item()?);
        }

        Ok(Module { name, items })
    }
}

pub fn parse_module<'s, 'p>(source: &'s str, path: Option<&'p Path>) -> (Option<Module>, Vec<ParseError<'s, 'p>>) {
    let mut self_ = Parser::new(source, path);
    (self_.parse_module().ok(), self_.errors)
}
