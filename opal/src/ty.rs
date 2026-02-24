use crate::ast::{Expr, Ident, VarUse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Bool,
    Unit,
    Void,
    Numeric(NumericType),
    Array(Box<Type>),
    Fun(FunSig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowedType<'a> {
    Bool,
    Unit,
    Void,
    Numeric(NumericType),
    Array(&'a BorrowedType<'a>),
    Fun(&'a [BorrowedType<'a>], &'a BorrowedType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunSig {
    pub params: Vec<Type>,
    pub returns: Box<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Int,
    Float,
}

impl TryFrom<&Expr> for Type {
    type Error = ();

    fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
        Ok(match expr {
            Expr::Var(VarUse(Ident(ident))) => match ident.as_str() {
                "Int" => Type::Numeric(NumericType::Int),
                "Float" => Type::Numeric(NumericType::Float),
                "Bool" => Type::Bool,
                "Unit" => Type::Unit,
                "Void" => Type::Void,
                _ => return Err(()),
            },
            Expr::Index(ty, param) => {
                let Expr::Var(VarUse(Ident(name))) = ty.as_ref() else {
                    return Err(());
                };
                if name.as_str() != "Array" {
                    return Err(());
                }
                let param = param.as_ref().try_into()?;
                Type::Array(Box::new(param))
            }
            _ => return Err(()),
        })
    }
}

impl FunSig {
    pub fn new(params: Vec<Type>, returns: Type) -> FunSig {
        FunSig {
            params,
            returns: Box::new(returns),
        }
    }
}

impl From<NumericType> for Type {
    fn from(value: NumericType) -> Self {
        Type::Numeric(value)
    }
}

impl Type {
    pub fn as_numeric_type(&self) -> Option<NumericType> {
        match *self {
            Type::Numeric(ty) => Some(ty),
            _ => None,
        }
    }
}

impl<'a> From<&BorrowedType<'a>> for Type {
    fn from(value: &BorrowedType<'a>) -> Self {
        match *value {
            BorrowedType::Numeric(ty) => Type::Numeric(ty),
            BorrowedType::Bool => Type::Bool,
            BorrowedType::Unit => Type::Unit,
            BorrowedType::Void => Type::Void,
            BorrowedType::Array(ty) => Type::Array(Box::new(ty.into())),
            BorrowedType::Fun(params, returns) => Type::Fun(FunSig {
                params: Vec::from_iter(params.into_iter().map(|ty| ty.into())),
                returns: Box::new(returns.into()),
            }),
        }
    }
}
