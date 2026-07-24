use crate::{
    ast,
    lexer::{Lexer, Span},
    token::{Keyword, Symbol, Token},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Option<(Token, Span)>,
}

#[derive(Debug)]
pub struct ParseError {
    pub span: Option<Span>,
    pub message: &'static str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        Self { token, lexer }
    }
    fn peek_token(&mut self) -> Option<&Token> {
        self.token.as_ref().map(|(token, _)| token)
    }
    fn advance_token(&mut self) -> Span {
        if let Some((_, span)) = self.token {
            self.token = self.lexer.next_token();
            span
        } else {
            panic!("end of input reached");
        }
    }
    fn parse_error(&self, message: &'static str) -> ParseError {
        ParseError {
            span: self.token.as_ref().map(|(_, span)| *span),
            message,
        }
    }
    pub fn parse_value(&mut self) -> Result<ast::Expr, ParseError> {
        match self.peek_token() {
            Some(Token::Int(value)) => {
                let value = *value;
                let span = self.advance_token();
                Ok(ast::Expr::Int { value, span })
            }
            Some(Token::String(value)) => {
                let value = value.clone();
                let span = self.advance_token();
                Ok(ast::Expr::String { value, span })
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                let span = self.advance_token();
                Ok(ast::Expr::Var { name, span })
            }
            Some(Token::Keyword(Keyword::True)) => {
                let span = self.advance_token();
                Ok(ast::Expr::Bool { value: true, span })
            }
            Some(Token::Keyword(Keyword::False)) => {
                let span = self.advance_token();
                Ok(ast::Expr::Bool { value: false, span })
            }
            Some(Token::Float(value)) => {
                let value = *value;
                let span = self.advance_token();
                Ok(ast::Expr::Float { value, span })
            }
            Some(Token::Symbol(Symbol::OpenParen)) => {
                let open_paren = self.advance_token();
                let expr = self.parse_expr(0)?;
                if self.peek_token() == Some(&Token::Symbol(Symbol::CloseParen)) {
                    let close_paren = self.advance_token();
                    Ok(ast::Expr::Parens {
                        open_paren,
                        expr: Box::new(expr),
                        close_paren,
                    })
                } else {
                    Err(self.parse_error("expected closing parenthesis"))
                }
            }
            _ => Err(self.parse_error("expected an expression")),
        }
    }
    pub fn parse_expr(&mut self, prec: u8) -> Result<ast::Expr, ParseError> {
        const INFIX_PRECEDENCE: &[(Symbol, ast::InfixOp, u8)] = &[
            (Symbol::Plus, ast::InfixOp::Add, 2),
            (Symbol::Minus, ast::InfixOp::Sub, 2),
            (Symbol::Asterisk, ast::InfixOp::Mul, 3),
            (Symbol::Slash, ast::InfixOp::Div, 3),
        ];

        let mut left = self.parse_value()?;

        'outer: loop {
            if self.peek_token() == Some(&Token::Symbol(Symbol::OpenParen)) && prec < 1 {
                let open_paren = self.advance_token();
                let mut args = Vec::new();
                while self.peek_token() != Some(&Token::Symbol(Symbol::CloseParen)) {
                    let expr = self.parse_expr(0)?;
                    if self.peek_token() == Some(&Token::Symbol(Symbol::Comma)) {
                        let comma = self.advance_token();
                        args.push((expr, Some(comma)));
                    } else {
                        args.push((expr, None));
                        break;
                    }
                }
                if self.peek_token() != Some(&Token::Symbol(Symbol::CloseParen)) {
                    return Err(self.parse_error("expected closing parenthesis"));
                }
                let close_paren = self.advance_token();
                left = ast::Expr::Call {
                    fun: Box::new(left),
                    open_paren,
                    args,
                    close_paren,
                };
                continue;
            }
            for (symbol, op, op_prec) in INFIX_PRECEDENCE.iter().copied() {
                if self.peek_token() == Some(&Token::Symbol(symbol)) && op_prec > prec {
                    let op_span = self.advance_token();
                    let right = self.parse_expr(op_prec)?;
                    left = ast::Expr::Infix {
                        left: Box::new(left),
                        op,
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
    pub fn parse_type(&mut self) -> Result<ast::Type, ParseError> {
        if let Some(Token::Ident(name)) = self.peek_token() {
            let name = name.clone();
            let span = self.advance_token();
            Ok(ast::Type { name, span })
        } else {
            Err(self.parse_error("expected a type"))
        }
    }
    pub fn parse_stmt(&mut self) -> Result<ast::Stmt, ParseError> {
        match self.peek_token() {
            Some(Token::Keyword(Keyword::Var)) => {
                let var = self.advance_token();
                if let Some(Token::Ident(name)) = self.peek_token() {
                    let name = name.clone();
                    let name_span = self.advance_token();
                    if self.peek_token() != Some(&Token::Symbol(Symbol::Equals)) {
                        return Err(self.parse_error("expected equals sign"));
                    }
                    let equals = self.advance_token();
                    let value = self.parse_expr(0)?;
                    if self.peek_token() != Some(&Token::Symbol(Symbol::Semicolon)) {
                        return Err(self.parse_error("expected semicolon"));
                    }
                    let semicolon = self.advance_token();
                    Ok(ast::Stmt::VarDecl {
                        var,
                        name,
                        name_span,
                        equals,
                        value,
                        semicolon,
                    })
                } else {
                    Err(self.parse_error("expected variable name"))
                }
            }
            Some(Token::Keyword(Keyword::Return)) => {
                let return_ = self.advance_token();
                let expr = if self.peek_token() == Some(&Token::Symbol(Symbol::Semicolon)) {
                    None
                } else {
                    let expr = self.parse_expr(0)?;
                    if self.peek_token() != Some(&Token::Symbol(Symbol::Semicolon)) {
                        return Err(self.parse_error("expected a semicolon"));
                    }
                    Some(expr)
                };
                let semicolon = self.advance_token();
                Ok(ast::Stmt::Return {
                    return_,
                    expr,
                    semicolon,
                })
            }
            _ => {
                let expr = self.parse_expr(0)?;
                if self.peek_token() != Some(&Token::Symbol(Symbol::Semicolon)) {
                    return Err(self.parse_error("expected semicolon"));
                };
                let semicolon = self.advance_token();
                Ok(ast::Stmt::Expr { expr, semicolon })
            }
        }
    }
    pub fn parse_block(&mut self) -> Result<ast::Block, ParseError> {
        if self.peek_token() != Some(&Token::Symbol(Symbol::OpenBrace)) {
            return Err(self.parse_error("expected opening brace"));
        }
        let open_brace = self.advance_token();
        let mut stmts = Vec::new();
        while self.peek_token() != Some(&Token::Symbol(Symbol::CloseBrace)) {
            stmts.push(self.parse_stmt()?);
        }
        let close_brace = self.advance_token();
        Ok(ast::Block {
            open_brace,
            stmts,
            close_brace,
        })
    }
    pub fn parse_param(&mut self) -> Result<ast::Param, ParseError> {
        let Some(Token::Ident(name)) = self.peek_token() else {
            return Err(self.parse_error("expected parameter name"));
        };
        let name = name.clone();
        let name_span = self.advance_token();
        if self.peek_token() != Some(&Token::Symbol(Symbol::Colon)) {
            return Err(self.parse_error("expected colon"));
        }
        let colon = self.advance_token();
        let ty = self.parse_type()?;
        let comma = if self.peek_token() == Some(&Token::Symbol(Symbol::Comma)) {
            Some(self.advance_token())
        } else {
            None
        };
        Ok(ast::Param {
            name,
            name_span,
            colon,
            ty,
            comma,
        })
    }
    pub fn parse_returns(&mut self) -> Result<ast::Returns, ParseError> {
        match self.peek_token() {
            Some(Token::Keyword(Keyword::NoReturn)) => {
                Ok(ast::Returns::NoReturn(self.advance_token()))
            }
            Some(Token::Symbol(Symbol::Arrow)) => {
                let arrow = self.advance_token();
                let ty = self.parse_type()?;
                Ok(ast::Returns::Type { arrow, ty })
            }
            _ => Ok(ast::Returns::None),
        }
    }
    pub fn parse_decl(&mut self) -> Result<ast::Decl, ParseError> {
        match self.peek_token() {
            Some(Token::Keyword(Keyword::Fun)) => {
                let func = self.advance_token();
                let Some(Token::Ident(name)) = self.peek_token() else {
                    return Err(self.parse_error("expected function name"));
                };
                let name = name.clone();
                let name_span = self.advance_token();
                if self.peek_token() != Some(&Token::Symbol(Symbol::OpenParen)) {
                    return Err(self.parse_error("expected opening parenthesis"));
                }
                let open_paren = self.advance_token();
                let mut params = Vec::new();
                while self.peek_token() != Some(&Token::Symbol(Symbol::CloseParen)) {
                    let param = self.parse_param()?;
                    let has_comma = param.comma.is_some();
                    params.push(param);
                    if !has_comma {
                        break;
                    }
                }
                let close_paren = self.advance_token();
                let returns = self.parse_returns()?;
                let body = self.parse_block()?;
                Ok(ast::Decl::Fun {
                    fun: func,
                    name,
                    name_span,
                    open_paren,
                    params,
                    close_paren,
                    returns,
                    body,
                })
            }
            _ => Err(self.parse_error("expected a declaration")),
        }
    }
    pub fn parse_file(&mut self) -> Result<ast::File, ParseError> {
        if self.peek_token() != Some(&Token::Keyword(Keyword::Module)) {
            return Err(self.parse_error("expected module declaration"));
        }
        let module_keyword = self.advance_token();
        let Some(Token::Ident(module_name)) = self.peek_token() else {
            return Err(self.parse_error("expected module name"));
        };
        let module_name = module_name.clone();
        let module_name_span = self.advance_token();
        if self.peek_token() != Some(&Token::Symbol(Symbol::Semicolon)) {
            return Err(self.parse_error("expected semicolon"));
        }
        let semicolon = self.advance_token();
        let mut decls = Vec::new();
        while self.peek_token().is_some() {
            decls.push(self.parse_decl()?);
        }
        Ok(ast::File {
            decls,
            module_keyword,
            module_name,
            module_name_span,
            semicolon,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use insta::assert_debug_snapshot;

    use super::{ParseError, Parser};

    fn parse_successful<T>(
        input: &str,
        parse_fn: impl FnOnce(&mut Parser) -> Result<T, ParseError>,
    ) -> T {
        let mut parser = Parser::new(input);
        parse_fn(&mut parser).unwrap()
    }

    #[test]
    fn test_infix_expr() {
        assert_debug_snapshot!(parse_successful("5 + 2", |parser| parser.parse_expr(0)));
    }

    #[test]
    fn test_call_expr() {
        assert_debug_snapshot!(parse_successful("foo(1, 2)", |parser| parser.parse_expr(0)));
    }

    #[test]
    fn test_paren_expr() {
        assert_debug_snapshot!(parse_successful("(1 + 2)", |parser| parser.parse_expr(0)));
    }

    #[test]
    fn test_unit_expr() {
        assert_debug_snapshot!(parse_successful("()", |parser| parser.parse_expr(0)));
    }

    #[test]
    fn test_precedence_expr() {
        assert_debug_snapshot!(parse_successful("1 + 2 * 3", |parser| parser.parse_expr(0)));
    }

    #[test]
    fn test_nested_call_expr() {
        assert_debug_snapshot!(parse_successful("foo(bar(1), 2)", |parser| parser.parse_expr(0)));
    }

    #[test]
    fn test_var_decl_stmt() {
        assert_debug_snapshot!(parse_successful("var x = 3;", |parser| parser.parse_stmt()));
    }

    #[test]
    fn test_expr_stmt() {
        assert_debug_snapshot!(parse_successful("foo(1);", |parser| parser.parse_stmt()));
    }

    #[test]
    fn test_block() {
        assert_debug_snapshot!(parse_successful("{ var x = 3; }", |parser| parser.parse_block()));
    }

    #[test]
    fn test_fun_decl() {
        assert_debug_snapshot!(parse_successful("fun main() { var x = 3; }", |parser| {
            parser.parse_decl()
        }));
    }

    #[test]
    fn test_module_decl() {
        assert_debug_snapshot!(parse_successful("module main;", |parser| parser.parse_file()));
    }

    #[test]
    fn test_module_with_decl() {
        assert_debug_snapshot!(parse_successful(
            "module main; fun main() { var x = 3; }",
            |parser| { parser.parse_file() }
        ));
    }

    fn parse_failure<T: Debug>(
        input: &str,
        parse_fn: impl FnOnce(&mut Parser) -> Result<T, ParseError>,
        expected_message: &str,
    ) {
        let error_start = input
            .find("<ERROR>")
            .expect("expected <ERROR> marker in input");
        let input = input.replace("<ERROR>", "");
        let mut parser = Parser::new(&input);
        let error = parse_fn(&mut parser).unwrap_err();
        assert_eq!(error.message, expected_message);
        if error_start == input.len() {
            assert!(error.span.is_none());
        } else {
            assert_eq!(error.span.unwrap().start, error_start);
        }
    }

    #[test]
    fn test_missing_closing_paren_error() {
        parse_failure(
            "(1 + 2<ERROR>",
            |parser| parser.parse_expr(0),
            "expected closing parenthesis",
        );
    }

    #[test]
    fn test_expected_expression_error() {
        parse_failure(
            "<ERROR>;",
            |parser| parser.parse_expr(0),
            "expected an expression",
        );
    }

    #[test]
    fn test_expected_type_error() {
        parse_failure(
            "fun main(x: <ERROR>) {}",
            |parser| parser.parse_decl(),
            "expected a type",
        );
    }

    #[test]
    fn test_expected_equals_sign_error() {
        parse_failure(
            "var x <ERROR>3;",
            |parser| parser.parse_stmt(),
            "expected equals sign",
        );
    }

    #[test]
    fn test_expected_semicolon_error() {
        parse_failure(
            "foo(1)<ERROR>",
            |parser| parser.parse_stmt(),
            "expected semicolon",
        );
    }

    #[test]
    fn test_expected_variable_name_error() {
        parse_failure(
            "var <ERROR>= 3;",
            |parser| parser.parse_stmt(),
            "expected variable name",
        );
    }

    #[test]
    fn test_expected_opening_brace_error() {
        parse_failure(
            "fun main()<ERROR>",
            |parser| parser.parse_decl(),
            "expected opening brace",
        );
    }

    #[test]
    fn test_expected_parameter_name_error() {
        parse_failure(
            "fun main(<ERROR>: i32) {}",
            |parser| parser.parse_decl(),
            "expected parameter name",
        );
    }

    #[test]
    fn test_expected_colon_error() {
        parse_failure(
            "fun main(x <ERROR>i32) {}",
            |parser| parser.parse_decl(),
            "expected colon",
        );
    }

    #[test]
    fn test_expected_function_name_error() {
        parse_failure(
            "fun <ERROR>() {}",
            |parser| parser.parse_decl(),
            "expected function name",
        );
    }

    #[test]
    fn test_expected_opening_parenthesis_error() {
        parse_failure(
            "fun main <ERROR>{}",
            |parser| parser.parse_decl(),
            "expected opening parenthesis",
        );
    }

    #[test]
    fn test_expected_declaration_error() {
        parse_failure(
            "<ERROR>let x = 3;",
            |parser| parser.parse_decl(),
            "expected a declaration",
        );
    }

    #[test]
    fn test_expected_module_declaration_error() {
        parse_failure(
            "<ERROR>fun main() {}",
            |parser| parser.parse_file(),
            "expected module declaration",
        );
    }

    #[test]
    fn test_expected_module_name_error() {
        parse_failure(
            "module <ERROR>;",
            |parser| parser.parse_file(),
            "expected module name",
        );
    }

    #[test]
    fn test_expected_module_semicolon_error() {
        parse_failure(
            "module main<ERROR>",
            |parser| parser.parse_file(),
            "expected semicolon",
        );
    }
}
