use std::{collections::HashMap, fs};

use opal_core::{ast, parser::Parser, ty::Type};

// fn convert_type(ty: &ast::Type) -> Type {
//     match &*ty.name {
//         "Int" => Type::Numeric(NumericType::Int),
//         "Float" => Type::Numeric(NumericType::Float),
//         "Bool" => Type::Bool,
//         "Unit" => Type::Unit,
//         _ => panic!(),
//     }
// }

fn main() {
    let input = fs::read_to_string("examples/example.opal").unwrap();
    let mut parser = Parser::new(&input);
    let file = parser.parse_file().unwrap();
    // let mut globals = HashMap::new();

    // for decl in &file.decls {
    //     match decl {
    //         ast::Decl::Fun {
    //             name, params, returns, ..
    //         } => {
    //             let mut param_tys = Vec::new();
    //             for param in params {
    //                 param_tys.push(convert_type(&param.ty));
    //             }
    //             let return_ty = if let Some(returns) = returns {
    //                 convert_type(&returns.ty)
    //             } else {
    //                 Type::Unit
    //             };
    //             globals.insert(name.clone(), Type::Fun(param_tys, Box::new(return_ty)));
    //         }
    //     }
    // }

    // for decl in &file.decls {
    //     match decl {
    //         ast::Decl::Fun {
    //             params, body, returns, ..
    //         } => {
    //             let return_ty = if let Some(returns) = returns {
    //                 convert_type(&returns.ty)
    //             } else {
    //                 Type::Unit
    //             };
    //             let mut analyzer = Analyzer::new(&globals, return_ty);
    //             for param in params {
    //                 analyzer.decl_local_var(param.name.clone(), convert_type(&param.ty));
    //             }
    //             let (body, fallthrough) = analyzer.analyze_block(body).unwrap();
    //             println!("{:?}", body);
    //             println!("{:?}", analyzer.is_fallthrough(&fallthrough))
    //         }
    //     }
    // }
}
