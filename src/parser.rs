use crate::{
    ast::{AstNode, Block, Decl, Expr, Ident, InfixOp, Stmt, Type},
    interner::InternedStr,
    lexer::{Lexer, Span},
    token::{self, Keyword, Symbol, Token, TokenKind, TokenTrait},
};

pub struct Parser<'s> {
    lexer: Lexer<'s>,
    pub token: Option<(Token<'s>, Span)>,
    prev_token_end: Option<u32>,
    next_id: u32,
    pub expected: Vec<TokenKind>,
}

impl<'s> Parser<'s> {
    pub fn new(source: &'s str) -> Parser<'s> {
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token();
        Parser {
            lexer,
            prev_token_end: None,
            token,
            next_id: 0,
            expected: Vec::new(),
        }
    }
    pub fn prev_token_end(&self) -> u32 {
        self.prev_token_end.unwrap()
    }
    pub fn next_token_start(&self) -> u32 {
        let (_, span) = self.token.as_ref().unwrap();
        span.start
    }
    fn new_node<T>(&mut self, span: Span, node: T) -> AstNode<T> {
        let id = self.next_id;
        self.next_id += 1;
        AstNode {
            id,
            span,
            node: Box::new(node),
        }
    }
    fn wrap_node<T, E>(
        &mut self,
        f: impl Fn(&mut Parser) -> Result<T, E>,
    ) -> Result<AstNode<T>, E> {
        let start = self.next_token_start();
        let node = f(self)?;
        let end = self.prev_token_end();
        Ok(self.new_node(Span::new(start, end), node))
    }
    fn expect<T>(&mut self, is_token: impl TokenTrait<Contents<'s> = T>) -> Result<T, ()> {
        match self.consume(is_token) {
            Some(token) => Ok(token),
            None => Err(()),
        }
    }
    fn consume<T>(&mut self, is_token: impl TokenTrait<Contents<'s> = T>) -> Option<T> {
        let Some((token, span)) = self.token.take() else {
            return None;
        };
        match is_token.matches(token) {
            Ok(item) => {
                self.prev_token_end = Some(span.end);
                self.token = self.lexer.next_token();
                self.expected.truncate(0);
                Some(item)
            }
            Err(token) => {
                self.expected.push(is_token.kind());
                self.token = Some((token, span));
                None
            }
        }
    }
}

pub fn parse_ident(parser: &mut Parser) -> Result<Ident, ()> {
    let start = parser.next_token_start();
    let ident = parser.expect(token::Ident)?;
    let end = parser.prev_token_end();
    Ok(Ident {
        str: InternedStr::intern(ident),
        span: Span::new(start, end),
    })
}

pub fn parse_value(parser: &mut Parser) -> Result<AstNode<Expr>, ()> {
    parser.wrap_node(|parser| {
        if let Some(value) = parser.consume(token::Int) {
            return Ok(Expr::Int(value));
        }
        if let Some(value) = parser.consume(token::Float) {
            return Ok(Expr::Float(value));
        }
        if parser.consume(Symbol::OpenParen).is_some() {
            let expr = parse_expr(parser)?;
            parser.expect(Symbol::CloseParen)?;
            return Ok(Expr::Paren(expr));
        }
        Err(())
    })
}

pub fn parse_expr(parser: &mut Parser) -> Result<AstNode<Expr>, ()> {
    const INFIX_OPS: &[(Symbol, InfixOp)] = &[
        (Symbol::Plus, InfixOp::Add),
        (Symbol::Minus, InfixOp::Subtract),
        (Symbol::Star, InfixOp::Multiply),
        (Symbol::Slash, InfixOp::Divide),
        (Symbol::Percent, InfixOp::Mod),
    ];

    let start = parser.next_token_start();
    let mut left = parse_value(parser)?;
    loop {
        for (symbol, infix_op) in INFIX_OPS {
            if let Some(_) = parser.consume(*symbol) {
                let right = parse_value(parser)?;
                let end = parser.prev_token_end();
                left = parser.new_node(
                    Span::new(start, end),
                    Expr::Infix {
                        left,
                        op: *infix_op,
                        right,
                    },
                );
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_stmt(parser: &mut Parser) -> Result<AstNode<Stmt>, ()> {
    parser.wrap_node(|parser| {
        if let Some(_) = parser.consume(Keyword::Var) {
            let name = parse_ident(parser)?;
            let expr = parser
                .consume(Symbol::Equals)
                .map(|_| parse_expr(parser))
                .transpose()?;
            parser.expect(Symbol::Semicolon)?;
            Ok(Stmt::VarDecl { name, expr })
        } else {
            let name = parse_ident(parser)?;
            parser.expect(Symbol::Equals)?;
            let expr = parse_expr(parser)?;
            Ok(Stmt::Assign { name, expr })
        }
    })
}

pub fn parse_block(parser: &mut Parser) -> Result<AstNode<Block>, ()> {
    parser.expect(Symbol::OpenBrace)?;
    parser.wrap_node(|parser| {
        let mut stmts = vec![];
        while parser.consume(Symbol::CloseBrace).is_none() {
            stmts.push(parse_stmt(parser)?);
        }
        Ok(Block { stmts })
    })
}

pub fn parse_separated<T, SEP: TokenTrait, TERM: TokenTrait>(
    parser: &mut Parser,
    sep: SEP,
    term: TERM,
    parse_fn: impl Fn(&mut Parser) -> Result<T, ()>,
) -> Result<Vec<T>, ()> {
    let mut vec = vec![];
    while parser.consume(term.clone()).is_none() {
        vec.push(parse_fn(parser)?);
        if parser.consume(sep.clone()).is_none() {
            parser.expect(term)?;
            break;
        }
    }
    Ok(vec)
}

pub fn parse_type(parser: &mut Parser) -> Result<AstNode<Type>, ()> {
    parser.wrap_node(|parser| {
        let name = parse_ident(parser)?;
        Ok(Type::Name(name))
    })
}

pub fn parse_decl(parser: &mut Parser) -> Result<AstNode<Decl>, ()> {
    parser.wrap_node(|parser| {
        if let Some(_) = parser.consume(Keyword::Fun) {
            let name = parse_ident(parser)?;
            parser.expect(Symbol::OpenParen)?;
            let params = parse_separated(parser, Symbol::Comma, Symbol::CloseParen, |parser| {
                let name = parse_ident(parser)?;
                parser.expect(Symbol::Colon)?;
                let ty = parse_type(parser)?;
                Ok((name, ty))
            })?;
            let returns = parser
                .consume(Symbol::RightArrow)
                .map(|_| parse_type(parser))
                .transpose()?;
            let block = parse_block(parser)?;
            Ok(Decl::Func {
                name,
                params,
                returns,
                block,
            })
        } else {
            Err(())
        }
    })
}
