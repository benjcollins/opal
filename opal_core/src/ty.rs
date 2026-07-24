use std::{borrow::Cow, collections::HashMap, convert::Infallible};

use derive_more::From;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<T> {
    Bool,
    Int,
    Float,
    String,
    Fun(Vec<T>, Box<T>),
}

pub struct FunType<T> {
    params: Vec<T>,
    returns: Returns<T>,
}

pub enum Returns<T> {
    NoReturn,
    None,
    Type(T),
}

#[derive(Debug, Clone, From)]
pub enum MetaType {
    Type(Type<MetaType>),
    Meta(Meta),
    NumericMeta(NumericMeta),
}

#[derive(Debug, Clone, From, PartialEq, Eq)]
pub struct CanonType(Type<CanonType>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Int,
    Float,
}

#[derive(Debug, Clone, From)]
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

impl<T> Type<T> {
    pub fn try_map<U, E>(&self, f: impl Fn(&T) -> Result<U, E> + Copy) -> Result<Type<U>, E> {
        match self {
            Type::Bool => Ok(Type::Bool),
            Type::Int => Ok(Type::Int),
            Type::Float => Ok(Type::Float),
            Type::String => Ok(Type::String),
            Type::Fun(params, returns) => {
                let params = params.iter().map(f).collect::<Result<Vec<_>, _>>()?;
                let returns = f(returns)?;
                Ok(Type::Fun(params, Box::new(returns)))
            }
        }
    }
    pub fn map<U>(&self, f: impl Fn(&T) -> U + Copy) -> Type<U> {
        match self.try_map::<_, Infallible>(|ty| Ok(f(ty))) {
            Ok(ty) => ty,
            Err(inf) => match inf {},
        }
    }
}

fn unify_types<A, B>(
    a: &Type<A>,
    b: &Type<B>,
    mut unify_fn: impl FnMut(&A, &B) -> Result<(), TypeError>,
) -> Result<(), TypeError> {
    match (a, b) {
        (Type::Bool, Type::Bool)
        | (Type::Int, Type::Int)
        | (Type::Float, Type::Float)
        | (Type::String, Type::String) => (),
        (Type::Fun(a_params, a_returns), Type::Fun(b_params, b_returns)) => {
            if a_params.len() != b_params.len() {
                return Err(TypeError);
            }
            for i in 0..a_params.len() {
                unify_fn(&a_params[i], &b_params[i])?;
            }
            unify_fn(a_returns, b_returns)?;
        }
        _ => return Err(TypeError),
    };
    Ok(())
}

impl From<&CanonType> for MetaType {
    fn from(value: &CanonType) -> Self {
        MetaType::Type(value.0.map(|ty| ty.into()))
    }
}

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
    fn unify_meta_type_with_canon_type(
        &self,
        a: &MetaType,
        b: &CanonType,
        substs: &mut HashMap<Meta, MetaType>,
        numeric_substs: &mut HashMap<NumericMeta, NumericMetaType>,
    ) -> Result<(), TypeError> {
        match a {
            MetaType::Type(ty) => {
                unify_types(ty, &b.0, |a, b| {
                    self.unify_meta_type_with_canon_type(a, b, substs, numeric_substs)
                })?;
            }
            MetaType::Meta(meta) => {
                substs.insert(*meta, b.into());
            }
            MetaType::NumericMeta(meta) => {
                let numeric_ty = match b.0 {
                    Type::Int => NumericType::Int,
                    Type::Float => NumericType::Float,
                    _ => return Err(TypeError),
                };
                numeric_substs.insert(*meta, numeric_ty.into());
            }
        };
        Ok(())
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
                unify_types(a, b, |a, b| {
                    self.unify_meta_types(a, b, substs, numeric_substs)
                })?;
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
    pub fn unify(&mut self, a: &MetaType, b: &MetaType) -> Result<(), TypeError> {
        let mut substs = HashMap::new();
        let mut numeric_substs = HashMap::new();
        self.unify_meta_types(a, b, &mut substs, &mut numeric_substs)?;
        for (meta, ty) in substs {
            self.type_map.insert(meta, ty);
        }
        for (meta, ty) in numeric_substs {
            self.numeric_type_map.insert(meta, ty);
        }
        Ok(())
    }
    pub fn unify_with_canon(&mut self, a: &MetaType, b: &CanonType) -> Result<(), TypeError> {
        let mut substs = HashMap::new();
        let mut numeric_substs = HashMap::new();
        self.unify_meta_type_with_canon_type(a, b, &mut substs, &mut numeric_substs)?;
        for (meta, ty) in substs {
            self.type_map.insert(meta, ty);
        }
        for (meta, ty) in numeric_substs {
            self.numeric_type_map.insert(meta, ty);
        }
        Ok(())
    }
    pub fn get_type<'a>(&'a self, ty: &'a MetaType) -> Option<Cow<'a, Type<MetaType>>> {
        match ty {
            MetaType::Type(ty) => Some(Cow::Borrowed(ty)),
            MetaType::Meta(meta) => self.type_map.get(meta).and_then(|ty| self.get_type(ty)),
            MetaType::NumericMeta(meta) => self
                .numeric_type_map
                .get(meta)
                .and_then(|ty| self.get_numeric_type(ty))
                .map(|ty| match ty {
                    NumericType::Int => Type::Int,
                    NumericType::Float => Type::Float,
                })
                .map(Cow::Owned),
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
    pub fn get_canon_type(&self, ty: &MetaType) -> Option<CanonType> {
        match ty {
            MetaType::Type(ty) => ty
                .try_map(|ty| self.get_canon_type(ty).ok_or(()))
                .map(CanonType)
                .ok(),
            MetaType::Meta(meta) => self
                .type_map
                .get(meta)
                .and_then(|ty| self.get_canon_type(ty)),
            MetaType::NumericMeta(meta) => self
                .numeric_type_map
                .get(meta)
                .and_then(|ty| self.get_numeric_type(ty))
                .map(|ty| match ty {
                    NumericType::Int => Type::Int,
                    NumericType::Float => Type::Float,
                })
                .map(CanonType),
        }
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
