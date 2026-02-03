use crate::intern::InternedStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub InternedStr);

impl Ident {
    pub fn new(s: &str) -> Ident {
        Ident(InternedStr::intern(s))
    }
}

#[derive(Debug, Clone)]
pub struct VarDef {
    pub mutable: bool,
    pub ident: Ident,
}

#[derive(Debug, Clone)]
pub struct VarUse(pub Ident);

impl VarUse {
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
    Call(Ident, Vec<Expr>),
    Paren(Box<Expr>),
    Var(VarUse),
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
    Let { var: VarDef, expr: Expr },
    Assign { var: VarUse, expr: Expr },
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Type(pub Ident);

#[derive(Debug)]
pub struct Fun {
    pub name: Ident,
    pub params: Vec<(VarDef, Type)>,
    pub returns: Option<Type>,
    pub block: Block,
}

#[derive(Debug)]
pub enum ModuleItem {
    Fun(Fun),
}

#[derive(Debug)]
pub struct Module {
    pub name: Ident,
    pub items: Vec<ModuleItem>,
}
