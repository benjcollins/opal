use std::{borrow::Cow, collections::HashMap};

use derive_more::From;

#[derive(Debug, Clone)]
pub enum Type {
    Unit,
    Bool,
    Void,
    Int,
    Float,
    Fun(Vec<MetaType>, Box<MetaType>),
}

#[derive(Debug, Clone, From)]
pub enum MetaType {
    Type(Type),
    Meta(Meta),
    NumericMeta(NumericMeta),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Int,
    Float,
}

#[derive(From)]
pub enum NumericMetaType {
    Type(NumericType),
    Meta(NumericMeta),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Meta(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumericMeta(u32);

pub struct TypeContext {
    type_map: HashMap<Meta, MetaType>,
    numeric_type_map: HashMap<NumericMeta, NumericMetaType>,
    next_meta: u32,
}

pub struct TypeError;

impl TypeContext {
    pub fn new() -> TypeContext {
        TypeContext {
            type_map: HashMap::new(),
            numeric_type_map: HashMap::new(),
            next_meta: 0,
        }
    }
    pub fn fresh_meta(&mut self) -> Meta {
        let meta = Meta(self.next_meta);
        self.next_meta += 1;
        meta
    }
    pub fn fresh_numeric_meta(&mut self) -> NumericMeta {
        let meta = NumericMeta(self.next_meta);
        self.next_meta += 1;
        meta
    }
    fn unify_meta_types(
        &self,
        a: &MetaType,
        b: &MetaType,
        substs: &mut HashMap<Meta, MetaType>,
        numeric_substs: &mut HashMap<NumericMeta, NumericMetaType>,
    ) -> Result<(), TypeError> {
        match (a, b) {
            (MetaType::Type(a), MetaType::Type(b)) => {
                self.unify_types(a, b, substs, numeric_substs)?;
            }
            (MetaType::Type(ty), MetaType::Meta(meta))
            | (MetaType::Meta(meta), MetaType::Type(ty)) => {
                substs.insert(*meta, ty.clone().into());
            }
            (MetaType::Meta(a), MetaType::Meta(b)) => {
                substs.insert(*a, (*b).into());
            }
            (MetaType::Type(ty), MetaType::NumericMeta(numeric_meta))
            | (MetaType::NumericMeta(numeric_meta), MetaType::Type(ty)) => {
                let numeric_ty = match ty {
                    Type::Int => NumericType::Int,
                    Type::Float => NumericType::Float,
                    _ => return Err(TypeError),
                };
                numeric_substs.insert(*numeric_meta, numeric_ty.into());
            }
            (MetaType::Meta(meta), MetaType::NumericMeta(numeric_meta))
            | (MetaType::NumericMeta(numeric_meta), MetaType::Meta(meta)) => {
                substs.insert(*meta, (*numeric_meta).into());
            }
            (MetaType::NumericMeta(a), MetaType::NumericMeta(b)) => {
                numeric_substs.insert(*a, (*b).into());
            }
        }
        Ok(())
    }
    fn unify_types(
        &self,
        a: &Type,
        b: &Type,
        substs: &mut HashMap<Meta, MetaType>,
        numeric_substs: &mut HashMap<NumericMeta, NumericMetaType>,
    ) -> Result<(), TypeError> {
        match (a, b) {
            (Type::Unit, Type::Unit)
            | (Type::Bool, Type::Bool)
            | (Type::Void, Type::Void)
            | (Type::Int, Type::Int)
            | (Type::Float, Type::Float) => (),
            (Type::Fun(a_params, a_returns), Type::Fun(b_params, b_returns)) => {
                if a_params.len() != b_params.len() {
                    return Err(TypeError);
                }
                for i in 0..a_params.len() {
                    self.unify_meta_types(&a_params[i], &b_params[i], substs, numeric_substs)?;
                }
                self.unify_meta_types(a_returns, b_returns, substs, numeric_substs)?;
            }
            _ => return Err(TypeError),
        };
        Ok(())
    }
    pub fn unify(&mut self, a: &MetaType, b: &MetaType) -> Result<(), TypeError> {
        let mut substs = HashMap::new();
        let mut numeric_substs = HashMap::new();
        self.unify_meta_types(a, b, &mut substs, &mut numeric_substs)?;
        for (meta, ty) in substs {
            self.type_map.insert(meta, ty);
        }
        Ok(())
    }
    pub fn get_type<'a>(&'a self, ty: &'a MetaType) -> Option<Cow<'a, Type>> {
        match ty {
            MetaType::Type(ty) => Some(Cow::Borrowed(ty)),
            MetaType::Meta(meta) => self.type_map.get(meta).and_then(|ty| self.get_type(ty)),
            MetaType::NumericMeta(meta) => self
                .numeric_type_map
                .get(meta)
                .and_then(|ty| self.get_numeric_type(ty))
                .map(|ty| {
                    Cow::Owned(match ty {
                        NumericType::Int => Type::Int,
                        NumericType::Float => Type::Float,
                    })
                }),
        }
    }
    pub fn get_numeric_type(&self, ty: &NumericMetaType) -> Option<NumericType> {
        match ty {
            NumericMetaType::Type(ty) => Some(*ty),
            NumericMetaType::Meta(meta) => self
                .numeric_type_map
                .get(meta)
                .and_then(|ty| self.get_numeric_type(ty)),
        }
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
