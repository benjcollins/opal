use crate::{
    ast::{
        ArithOp, Block, CompOp, Else, Expr, Fun, Ident, If, InfixOp, Lit, Module, ModuleItem, Stmt, Type, VarDef,
        VarUse,
    },
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
        let expr = parse_expr(parser, 0)?;
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
        return parse_value_ident(parser, Ident::new(ident));
    }
    Err(())
}

pub fn parse_value_ident(parser: &mut Parser, ident: Ident) -> Result<Expr, ()> {
    if parser.consume2(Symbol::OpenParen) {
        let args = parse_separated(parser, Symbol::Comma, Symbol::CloseParen, |parser| {
            parse_expr(parser, 0)
        })?;
        return Ok(Expr::Call(ident, args));
    } else {
        return Ok(Expr::Var(VarUse(ident)));
    }
}

type Prec = u8;

pub fn parse_expr(parser: &mut Parser, prec: Prec) -> Result<Expr, ()> {
    let left = parse_value(parser)?;
    parse_infix(parser, left, prec)
}

pub fn parse_infix(parser: &mut Parser, mut left: Expr, prec: Prec) -> Result<Expr, ()> {
    const INFIX_OPS: &[(Symbol, InfixOp, Prec)] = &[
        (Symbol::Star, InfixOp::Arith(ArithOp::Multiply), 3),
        (Symbol::Slash, InfixOp::Arith(ArithOp::Divide), 3),
        //
        (Symbol::Percent, InfixOp::Arith(ArithOp::Modulus), 3),
        (Symbol::Plus, InfixOp::Arith(ArithOp::Add), 2),
        (Symbol::Minus, InfixOp::Arith(ArithOp::Subtract), 2),
        //
        (Symbol::DoubleEquals, InfixOp::Comp(CompOp::Equal), 1),
        (Symbol::BangEquals, InfixOp::Comp(CompOp::NotEqual), 1),
        (Symbol::Less, InfixOp::Comp(CompOp::Less), 1),
        (Symbol::Greater, InfixOp::Comp(CompOp::Greater), 1),
        (Symbol::LessEquals, InfixOp::Comp(CompOp::LessEqual), 1),
        (Symbol::GreaterEquals, InfixOp::Comp(CompOp::GreaterEqual), 1),
    ];
    'outer: loop {
        for (symbol, infix_op, op_prec) in INFIX_OPS {
            if *op_prec > prec && parser.consume2(*symbol) {
                let right = parse_expr(parser, *op_prec)?;
                left = Expr::Infix {
                    left: Box::new(left),
                    op: *infix_op,
                    right: Box::new(right),
                };
                continue 'outer;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_var_def(parser: &mut Parser) -> Result<VarDef, ()> {
    let mutable = parser.consume2(Keyword::Mut);
    let ident = Ident::new(parser.expect(token::Ident)?);
    Ok(VarDef { mutable, ident })
}

pub fn parse_if(parser: &mut Parser) -> Result<If, ()> {
    parser.expect(Symbol::OpenParen)?;
    let cond = parse_expr(parser, 0)?;
    parser.expect(Symbol::CloseParen)?;
    let if_block = parse_block(parser)?;
    let else_ = if parser.consume2(Keyword::Else) {
        if parser.consume2(Keyword::If) {
            Else::If(Box::new(parse_if(parser)?))
        } else {
            Else::Block(parse_block(parser)?)
        }
    } else {
        Else::Nothing
    };
    Ok(If { cond, if_block, else_ })
}

pub fn parse_stmt(parser: &mut Parser) -> Result<Stmt, ()> {
    if parser.consume2(Keyword::Let) {
        let var = parse_var_def(parser)?;
        parser.expect(Symbol::Equals)?;
        let expr = parse_expr(parser, 0)?;
        parser.expect(Symbol::Semicolon)?;
        return Ok(Stmt::Let { var, expr });
    }
    if parser.consume2(Keyword::Return) {
        let expr = (!parser.consume2(Symbol::Semicolon))
            .then(|| {
                let expr = parse_expr(parser, 0)?;
                parser.expect(Symbol::Semicolon)?;
                Ok(expr)
            })
            .transpose()?;
        return Ok(Stmt::Return(expr));
    }
    if let Some(ident) = parser.consume(token::Ident) {
        if parser.consume2(Symbol::Equals) {
            let var = VarUse(Ident::new(ident));
            let expr = parse_expr(parser, 0)?;
            parser.expect(Symbol::Semicolon)?;
            return Ok(Stmt::Assign { var, expr });
        };

        const ARITH_ASSIGN_OPS: &[(ArithOp, Symbol)] = &[
            (ArithOp::Add, Symbol::PlusEquals),
            (ArithOp::Subtract, Symbol::MinusEquals),
            (ArithOp::Multiply, Symbol::StarEquals),
            (ArithOp::Divide, Symbol::SlashEquals),
            (ArithOp::Modulus, Symbol::PercentEquals),
        ];

        for &(op, symbol) in ARITH_ASSIGN_OPS {
            if parser.consume2(symbol) {
                let var = VarUse(Ident::new(ident));
                let expr = parse_expr(parser, 0)?;
                parser.expect(Symbol::Semicolon)?;
                return Ok(Stmt::AssignArith { var, op, expr });
            }
        }

        let left = parse_value_ident(parser, Ident::new(ident))?;
        let expr = parse_infix(parser, left, 0)?;
        parser.expect(Symbol::Semicolon)?;
        return Ok(Stmt::Expr(expr));
    }
    if parser.consume2(Keyword::If) {
        return Ok(Stmt::If(parse_if(parser)?));
    }
    if parser.consume2(Keyword::While) {
        parser.expect(Symbol::OpenParen)?;
        let cond = parse_expr(parser, 0)?;
        parser.expect(Symbol::CloseParen)?;
        let block = parse_block(parser)?;
        return Ok(Stmt::While { cond, block });
    }
    return Ok(Stmt::Expr(parse_expr(parser, 0)?));
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
            let var = parse_var_def(parser)?;
            parser.expect(Symbol::Colon)?;
            let ty = parse_type(parser)?;
            Ok((var, ty))
        })?;
        let returns = parser
            .consume2(Symbol::RightArrow)
            .then(|| parse_type(parser))
            .transpose()?;
        let block = parse_block(parser)?;
        Ok(ModuleItem::Fun(Fun {
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
