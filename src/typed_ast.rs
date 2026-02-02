use crate::{
    ast::{Ident, InfixOp, Lit},
    infer::{NumericType, Type},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

#[derive(Debug, Clone)]
pub struct TypedVar {
    pub ident: Ident,
    pub id: VarId,
}

#[derive(Debug, Clone)]
pub enum TypedExpr {
    Lit(Lit),
    Var(TypedVar),
    Infix {
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        op: InfixOp,
        ty: NumericType,
    },
}

#[derive(Debug, Clone)]
pub enum TypedStmt {
    VarDecl { var: TypedVar, expr: TypedExpr },
    Assign { var: TypedVar, expr: TypedExpr },
}

#[derive(Debug)]
pub struct TypedBlock {
    pub stmts: Vec<TypedStmt>,
}

#[derive(Debug)]
pub struct TypedFunDef {
    pub name: Ident,
    pub params: Vec<(TypedVar, Type)>,
    pub returns: Option<Type>,
    pub block: TypedBlock,
}
