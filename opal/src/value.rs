use std::{convert::Infallible, marker::PhantomData, mem, ptr};

use strum::{EnumIs, FromRepr};

use crate::{
    heap2::{Function, List, Object},
    runtime::HostFun,
    ty::{BorrowedType, NumericType},
    vm::RuntimeError,
};

#[derive(Debug, Clone, Copy)]
pub enum Value<'h> {
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
    HostFun(HostFun),
    List(&'h Object<List>),
    Fun(&'h Object<Function>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs, FromRepr)]
#[repr(u8)]
pub enum ValueTag {
    Int,
    Float,
    Bool,
    Unit,
    List,
    HostFun,
    Fun,
    Void,
}

pub struct TypedList<'h, T> {
    object: &'h Object<List>,
    phantom: PhantomData<T>,
}

pub trait ValueConv<'h> {
    const TYPE: BorrowedType<'static>;

    fn into(self) -> Value<'h>;
    fn from(value: Value<'h>) -> Self;
}

pub trait NativeFunResult<'h> {
    type Output: ValueConv<'h>;
    fn map(self) -> Result<Value<'h>, RuntimeError>;
}

impl<'h> Value<'h> {
    pub fn to_raw_parts(&self) -> (ValueTag, *mut ()) {
        match *self {
            Value::Int(value) => (ValueTag::Int, ptr::without_provenance_mut(value as usize)),
            Value::Float(value) => (ValueTag::Float, ptr::without_provenance_mut(value.to_bits() as usize)),
            Value::Bool(value) => (ValueTag::Bool, ptr::without_provenance_mut(if value { 1 } else { 0 })),
            Value::Unit => (ValueTag::Unit, ptr::without_provenance_mut(0)),
            Value::HostFun(value) => (ValueTag::HostFun, value as *mut ()),
            Value::List(object) => (ValueTag::List, ptr::from_ref(object).cast::<()>().cast_mut()),
            Value::Fun(object) => (ValueTag::Fun, ptr::from_ref(object).cast::<()>().cast_mut()),
        }
    }
    pub unsafe fn from_raw_parts(tag: ValueTag, data: *mut ()) -> Value<'h> {
        match tag {
            ValueTag::Int => Value::Int(data.addr() as i64),
            ValueTag::Float => Value::Float(f64::from_bits(data.addr() as u64)),
            ValueTag::Bool => Value::Bool(data.addr() == 1),
            ValueTag::Unit => Value::Unit,
            ValueTag::List => Value::List(unsafe { data.cast::<Object<List>>().as_ref_unchecked() }),
            ValueTag::HostFun => Value::HostFun(unsafe { mem::transmute::<*mut (), HostFun>(data) }),
            ValueTag::Fun => Value::Fun(unsafe { data.cast::<Object<Function>>().as_ref_unchecked() }),
            ValueTag::Void => panic!(),
        }
    }
}

impl<'h, T: ValueConv<'h>> NativeFunResult<'h> for Result<T, RuntimeError> {
    type Output = T;
    fn map(self) -> Result<Value<'h>, RuntimeError> {
        self.map(|t| t.into())
    }
}

impl<'h> ValueConv<'h> for i64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Int);
    fn into(self) -> Value<'h> {
        Value::Int(self)
    }
    fn from(value: Value<'h>) -> Self {
        let Value::Int(value) = value else { panic!() };
        value
    }
}

impl<'h> ValueConv<'h> for bool {
    const TYPE: BorrowedType<'static> = BorrowedType::Bool;
    fn into(self) -> Value<'h> {
        Value::Bool(self)
    }
    fn from(value: Value<'h>) -> Self {
        let Value::Bool(value) = value else { panic!() };
        value
    }
}

impl<'h> ValueConv<'h> for f64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Float);
    fn into(self) -> Value<'h> {
        Value::Float(self)
    }
    fn from(value: Value<'h>) -> Self {
        let Value::Float(value) = value else { panic!() };
        value
    }
}

impl<'h> ValueConv<'h> for () {
    const TYPE: BorrowedType<'static> = BorrowedType::Unit;
    fn into(self) -> Value<'h> {
        Value::Unit
    }
    fn from(value: Value<'h>) -> Self {
        let Value::Unit = value else { panic!() };
    }
}

impl<'h> ValueConv<'h> for Infallible {
    const TYPE: BorrowedType<'static> = BorrowedType::Void;
    fn into(self) -> Value<'h> {
        match self {}
    }
    fn from(_: Value<'h>) -> Self {
        unreachable!()
    }
}

impl<'h, T: ValueConv<'h>> ValueConv<'h> for TypedList<'h, T> {
    const TYPE: BorrowedType<'static> = BorrowedType::Array(&T::TYPE);
    fn into(self) -> Value<'h> {
        Value::List(self.object)
    }
    fn from(value: Value<'h>) -> Self {
        let Value::List(object) = value else { panic!() };
        TypedList {
            object,
            phantom: PhantomData,
        }
    }
}

impl<'h, T: ValueConv<'h>> TypedList<'h, T> {
    pub fn len(&self) -> usize {
        self.object.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: i64) -> T {
        let p = self.object.get_element(index as usize);
        T::from(p)
    }
    pub fn set(&self, index: i64, value: T) {
        self.object.set_element(index as usize, value.into());
    }
}
