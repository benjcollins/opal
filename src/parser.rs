use crate::{
    ast::{Block, Expr, FunDef, Ident, InfixOp, Lit, Module, ModuleItem, Stmt, Type, Var},
    lexer::{Lexer, Span},
    token::{self, Float, Int, Keyword, Symbol, Token, TokenKind, TokenType},
};

pub struct Parser<'s> {
    pub lexer: Lexer<'s>,
    pub token: Option<(Token<'s>, Span)>,
    pub expected: Vec<TokenKind>,
}

impl<'s> Parser<'s> {
    pub fn new(source: &'s str) -> Parser<'s> {
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token();
        Parser {
            lexer,
            token,
            expected: Vec::new(),
        }
    }
    fn expect<T>(&mut self, token: impl TokenType<Contents<'s> = T>) -> Result<T, ()> {
        match self.consume(token) {
            Some(token) => Ok(token),
            None => Err(()),
        }
    }
    fn consume<T>(&mut self, token_type: impl TokenType<Contents<'s> = T>) -> Option<T> {
        let Some((token, span)) = self.token.take() else {
            return None;
        };
        match token_type.matches(token) {
            Ok(item) => {
                self.token = self.lexer.next_token();
                self.expected.truncate(0);
                Some(item)
            }
            Err(token) => {
                self.expected.push(token_type.kind());
                self.token = Some((token, span));
                None
            }
        }
    }
    fn consume2(&mut self, token: impl TokenType) -> bool {
        self.consume(token).is_some()
    }
}

pub fn parse_value(parser: &mut Parser) -> Result<Expr, ()> {
    if let Some(value) = parser.consume(Int) {
        return Ok(Expr::Lit(Lit::Int(value)));
    }
    if let Some(value) = parser.consume(Float) {
        return Ok(Expr::Lit(Lit::Float(value)));
    }
    if parser.consume2(Symbol::OpenParen) {
        let expr = parse_expr(parser)?;
        parser.expect(Symbol::CloseParen)?;
        return Ok(Expr::Paren(Box::new(expr)));
    }
    if parser.consume2(Keyword::True) {
        return Ok(Expr::Lit(Lit::Bool(true)));
    }
    if parser.consume2(Keyword::False) {
        return Ok(Expr::Lit(Lit::Bool(false)));
    }
    if let Some(ident) = parser.consume(token::Ident) {
        return Ok(Expr::Var(Var(Ident::new(ident))));
    }
    Err(())
}

pub fn parse_expr(parser: &mut Parser) -> Result<Expr, ()> {
    const INFIX_OPS: &[(Symbol, InfixOp)] = &[
        (Symbol::Plus, InfixOp::Add),
        (Symbol::Minus, InfixOp::Subtract),
        (Symbol::Star, InfixOp::Multiply),
        (Symbol::Slash, InfixOp::Divide),
        (Symbol::Percent, InfixOp::Mod),
    ];

    let mut left = parse_value(parser)?;
    loop {
        for (symbol, infix_op) in INFIX_OPS {
            if parser.consume2(*symbol) {
                let right = parse_value(parser)?;
                left = Expr::Infix {
                    left: Box::new(left),
                    op: *infix_op,
                    right: Box::new(right),
                };
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_stmt(parser: &mut Parser) -> Result<Stmt, ()> {
    if parser.consume2(Keyword::Let) {
        let ident = Ident::new(parser.expect(token::Ident)?);
        parser.expect(Symbol::Equals)?;
        let expr = parse_expr(parser)?;
        parser.expect(Symbol::Semicolon)?;
        Ok(Stmt::VarDef {
            var: Var(ident),
            expr,
        })
    } else {
        let name = Var(Ident::new(parser.expect(token::Ident)?));
        parser.expect(Symbol::Equals)?;
        let expr = parse_expr(parser)?;
        Ok(Stmt::Assign { var: name, expr })
    }
}

pub fn parse_block(parser: &mut Parser) -> Result<Block, ()> {
    parser.expect(Symbol::OpenBrace)?;
    let mut stmts = vec![];
    while !parser.consume2(Symbol::CloseBrace) {
        stmts.push(parse_stmt(parser)?);
    }
    Ok(Block { stmts })
}

pub fn parse_separated<T, SEP: TokenType, TERM: TokenType>(
    parser: &mut Parser,
    sep: SEP,
    term: TERM,
    parse_fn: impl Fn(&mut Parser) -> Result<T, ()>,
) -> Result<Vec<T>, ()> {
    let mut vec = vec![];
    while !parser.consume2(term.clone()) {
        vec.push(parse_fn(parser)?);
        if !parser.consume2(sep.clone()) {
            parser.expect(term)?;
            break;
        }
    }
    Ok(vec)
}

pub fn parse_type(parser: &mut Parser) -> Result<Type, ()> {
    let ident = Ident::new(parser.expect(token::Ident)?);
    Ok(Type(ident))
}

pub fn parse_module_item(parser: &mut Parser) -> Result<ModuleItem, ()> {
    if parser.consume2(Keyword::Fun) {
        let name = Ident::new(parser.expect(token::Ident)?);
        parser.expect(Symbol::OpenParen)?;
        let params = parse_separated(parser, Symbol::Comma, Symbol::CloseParen, |parser| {
            let var = Var(Ident::new(parser.expect(token::Ident)?));
            parser.expect(Symbol::Colon)?;
            let ty = parse_type(parser)?;
            Ok((var, ty))
        })?;
        let returns = parser
            .consume2(Symbol::RightArrow)
            .then(|| parse_type(parser))
            .transpose()?;
        let block = parse_block(parser)?;
        Ok(ModuleItem::FunDef(FunDef {
            name,
            params,
            returns,
            block,
        }))
    } else {
        Err(())
    }
}

pub fn parse_module(parser: &mut Parser) -> Result<Module, ()> {
    parser.expect(Keyword::Module)?;
    let name = Ident::new(parser.expect(token::Ident)?);
    parser.expect(Symbol::Semicolon)?;
    let mut items = vec![];
    while parser.token.is_some() {
        items.push(parse_module_item(parser)?);
    }
    Ok(Module { name, items })
}
