use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Unit,
    Void,
    Array(Box<Type>),
    Fun(FunSig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowedType<'a> {
    Int,
    Float,
    Bool,
    Unit,
    Void,
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

impl TryFrom<&ast::Type> for Type {
    type Error = ();

    fn try_from(ty: &ast::Type) -> Result<Self, Self::Error> {
        Ok(match ty.0.0.as_str() {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "Unit" => Type::Unit,
            "Void" => Type::Void,
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
        match value {
            NumericType::Int => Type::Int,
            NumericType::Float => Type::Float,
        }
    }
}

impl Type {
    pub fn as_numeric_type(&self) -> Option<NumericType> {
        match self {
            Type::Int => Some(NumericType::Int),
            Type::Float => Some(NumericType::Float),
            _ => None,
        }
    }
}

impl<'a> From<&BorrowedType<'a>> for Type {
    fn from(value: &BorrowedType<'a>) -> Self {
        match *value {
            BorrowedType::Int => Type::Int,
            BorrowedType::Float => Type::Float,
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
