use crate::intern::InternedStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub InternedStr);

impl Ident {
    pub fn new(s: &str) -> Ident {
        Ident(InternedStr::new(s))
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

#[derive(Debug, Clone)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Call(Box<Expr>, Vec<Expr>),
    Paren(Box<Expr>),
    Var(VarUse),
    ArrayElements(Vec<Expr>),
    ArrayDefaultLength(Box<Expr>, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Prefix(PrefixOp, Box<Expr>),
    FunType(Vec<Expr>, Option<Box<Expr>>),
    Infix {
        left: Box<Expr>,
        op: InfixOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp {
    Arith(ArithOp),
    Comp(CompOp),
    Equality(EqualityOp),
    Logical(LogicalOp),
    Bitwise(BitwiseOp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixOp {
    Negative,
    Positive,
    LogicalNot,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseOp {
    And,
    Or,
    XOr,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityOp {
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompOp {
    Greater,
    Less,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Bitwise(BitwiseOp),
    Arith(ArithOp),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let { var: VarDef, ty: Option<Expr>, expr: Expr },
    Assign { dst: Expr, op: Option<AssignOp>, src: Expr },
    Break,
    Continue,
    Return(Option<Expr>),
    Expr(Expr),
    If(If),
    While { cond: Expr, block: Block },
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expr,
    pub if_block: Block,
    pub else_: Else,
}

#[derive(Debug, Clone)]
pub enum Else {
    If(Box<If>),
    Block(Block),
    Nothing,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Fun {
    pub name: Ident,
    pub params: Vec<(VarDef, Expr)>,
    pub returns: Option<Expr>,
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
