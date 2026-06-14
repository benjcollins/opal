use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum Type<T> {
    Bool,
    String,
    Unit,
    Void,
    Numeric(NumericType),
    Fun(Vec<T>, Box<T>),
}

#[derive(Debug, Clone)]
pub enum NumericType {
    Int,
    Float,
}

pub enum Var<T> {
    Type(Option<T>),
    Rc<RefCell<Option<T>>>,
}

struct InferType(Var<Type<InferType>>);

struct CanonType(Type<CanonType>);

impl<T> Var<T> {
    pub fn unify(&self, other: &Self) -> Result<(), ()> {
        match (self.0.borrow(), other.0.borrow()) {
            (None, None) => {
                Var(Rc::new(RefCell::new(value)))
            }
            (None, Some(_)) => todo!(),
            (Some(_), None) => todo!(),
            (Some(_), Some(_)) => todo!(),
        }
    }
}

impl InferType {
    pub fn unify(&self, other: &InferType) -> Result<(), ()> {
        match (self, other) {
            (Type::Unit, Type::Unit) => Ok(()),
            (Type::String, Type::String) => Ok(()),
            (Type::Void, Type::Void) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),

            (Type::Fun(self_params, self_returns), Type::Fun(other_params, other_returns)) => {
                if self_params.len() != other_params.len() {
                    return Err((self.clone(), other.clone()));
                }
                for i in 0..self_params.len() {
                    self_params[i].unify(&other_params[i])?;
                }
                self_returns.unify(other_returns)
            }

            (Type::Numeric(a), Type::Numeric(b)) => a.unify(b).map_err(|| Type::Numeric(())),
            (Type::Meta(meta_ty), other_ty) | (other_ty, Type::Meta(meta_ty)) => {
                todo!()
            }
            _ => Err(()),
        }
    }
}

// impl NumericType {
//     pub fn unify(&self, other: &NumericType) -> Result<(), (NumericType, NumericType)> {
//         todo!()
//     }
// }
