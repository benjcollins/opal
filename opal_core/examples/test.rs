use std::{collections::HashMap, fs};

use opal_core::{
    analyzer::Analyzer,
    ast,
    codegen::Codegen,
    ir::{Expr, Stmt},
    parser::Parser,
    ty::{Type, TypeContext},
};

fn convert_type<T>(ty: &ast::Type) -> Type<T> {
    match &*ty.name {
        "Int" => Type::Int,
        "Float" => Type::Float,
        "Bool" => Type::Bool,
        "Unit" => Type::Unit,
        "Void" => Type::Void,
        _ => panic!(),
    }
}

fn main() {
    let input = fs::read_to_string("examples/example.opal").unwrap();
    let mut parser = Parser::new(&input);
    let file = parser.parse_file().unwrap();
    let mut globals = HashMap::new();

    for decl in &file.decls {
        match decl {
            ast::Decl::Fun {
                name,
                params,
                returns,
                ..
            } => {
                let mut param_tys = Vec::new();
                for param in params {
                    param_tys.push(convert_type(&param.ty).into());
                }
                let return_ty = if let Some(returns) = returns {
                    convert_type(&returns.ty)
                } else {
                    Type::Unit
                };
                globals.insert(
                    name.clone(),
                    Type::Fun(param_tys, Box::new(return_ty.into())).into(),
                );
            }
        }
    }

    for decl in &file.decls {
        match decl {
            ast::Decl::Fun {
                params,
                body,
                returns,
                ..
            } => {
                let return_ty = if let Some(returns) = returns {
                    convert_type(&returns.ty)
                } else {
                    Type::Unit
                };
                let mut type_context = TypeContext::new();
                let mut analyzer =
                    Analyzer::new(&globals, return_ty.clone().into(), &mut type_context);
                for param in params {
                    analyzer.decl_local_var(param.name.clone(), convert_type(&param.ty).into());
                }
                let (mut body, fallthrough) = analyzer.analyze_block(body).unwrap();
                if analyzer.is_fallthrough(&fallthrough) {
                    if return_ty == Type::Unit {
                        body.stmts.push(Stmt::Return(Expr::Unit));
                    } else {
                        panic!("missing return!");
                    }
                }
                let mut codegen = Codegen::new(&type_context);

                codegen.gen_block(&body);
            }
        }
    }
}
