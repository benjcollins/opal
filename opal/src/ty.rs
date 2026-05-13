use crate::{
    ast::{Expr, Ident},
    intern::InternedStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Bool,
    Unit,
    Void,
    Str,
    Numeric(NumericType),
    List(Box<Type>),
    Fun(FunSig),
    Generic(InternedStr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowedType<'a> {
    Bool,
    Unit,
    Void,
    Str,
    Numeric(NumericType),
    Array(&'a BorrowedType<'a>),
    Fun(&'a [BorrowedType<'a>], &'a BorrowedType<'a>),
    Generic(&'static str),
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
            Expr::Var(Ident { str, .. }) => match str.as_str() {
                "Int" => Type::Numeric(NumericType::Int),
                "Float" => Type::Numeric(NumericType::Float),
                "Bool" => Type::Bool,
                "Unit" => Type::Unit,
                "Void" => Type::Void,
                _ => Type::Generic(str.clone()),
            },
            Expr::FunType { params, returns, .. } => {
                let params = params
                    .items()
                    .map(|param| param.try_into())
                    .collect::<Result<Vec<Type>, _>>()?;
                let returns = if let Some((_, returns)) = returns {
                    Box::new(returns.as_ref().try_into()?)
                } else {
                    Box::new(Type::Unit)
                };
                Type::Fun(FunSig { params, returns })
            }
            Expr::Index { expr, index, .. } => {
                let Expr::Var(Ident { str, .. }) = expr.as_ref() else {
                    return Err(());
                };
                if str.as_str() != "List" {
                    return Err(());
                }
                let param = index.as_ref().try_into()?;
                Type::List(Box::new(param))
            }
            _ => return Err(()),
        })
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
            BorrowedType::Str => Type::Str,
            BorrowedType::Array(ty) => Type::List(Box::new(ty.into())),
            BorrowedType::Fun(params, returns) => Type::Fun(FunSig {
                params: Vec::from_iter(params.iter().map(|ty| ty.into())),
                returns: Box::new(returns.into()),
            }),
            BorrowedType::Generic(name) => Type::Generic(InternedStr::new(name)),
        }
    }
}
