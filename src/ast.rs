use crate::intern::InternedStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub InternedStr);

impl Ident {
    pub fn new(s: &str) -> Ident {
        Ident(InternedStr::intern(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var(pub Ident);

impl Var {
    pub fn ident(&self) -> &Ident {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Paren(Box<Expr>),
    Var(Var),
    Infix {
        left: Box<Expr>,
        op: InfixOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDef { var: Var, expr: Expr },
    Assign { var: Var, expr: Expr },
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Type(pub Ident);

#[derive(Debug)]
pub struct FunDef {
    pub name: Ident,
    pub params: Vec<(Var, Type)>,
    pub returns: Option<Type>,
    pub block: Block,
}

#[derive(Debug)]
pub enum ModuleItem {
    FunDef(FunDef),
}

#[derive(Debug)]
pub struct Module {
    pub name: Ident,
    pub items: Vec<ModuleItem>,
}
