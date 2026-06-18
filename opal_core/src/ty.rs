use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Type {
    Bool,
    String,
    Unit,
    Void,
    Int,
    Float,
    Fun(Vec<TypeId>, TypeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

pub struct Context {
    next_type_id: u32,
    type_map: HashMap<TypeId, Type>,
}

impl Context {
    pub fn replace(&mut self, before: TypeId, after: TypeId) {
        for (_, ty) in self.type_map.iter_mut() {
            ty.replace(before, after);
        }
    }
    pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<(), ()> {
        if a == b {
            return Ok(());
        }
        match (self.type_map.get(&a), self.type_map.get(&b)) {
            (None, None) => self.replace(a, b),
            (None, Some(_)) => self.replace(a, b),
            (Some(_), None) => self.replace(b, a),
            (Some(a_ty), Some(b_ty)) => {
                self.unify_types(a_ty, b_ty)?;
                self.replace(a, b)
            }
        }
        Ok(())
    }
    pub fn unify_types(&mut self, a: &Type, b: &Type) -> Result<(), ()> {
        match (a, b) {
            (Type::Bool, Type::Bool) => Ok(()),
            (Type::Unit, Type::Unit) => Ok(()),
            (Type::Int, Type::Int) => Ok(()),
            (Type::Float, Type::Float) => Ok(()),
            (Type::Void, Type::Void) => Ok(()),
            (Type::Fun(a_params, a_returns), Type::Fun(b_params, b_returns)) => {
                if a_params.len() != b_params.len() {
                    return Err(());
                }
                for i in 0..a_params.len() {
                    self.unify(a_params[i], b_params[i])?;
                }
                self.unify(*a_returns, *b_returns)?;
                Ok(())
            }
            _ => Err(()),
        }
    }
}

impl Type {
    pub fn replace(&mut self, before: TypeId, after: TypeId) {
        match self {
            Type::Bool | Type::String | Type::Unit | Type::Void => (),
            Type::Fun(params, returns) => {
                for param in params {
                    if *param == before {
                        *param = after;
                    }
                }
                if *returns == before {
                    *returns = after;
                }
            }
        }
    }
}
