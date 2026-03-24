use std::{convert::Infallible, fmt, marker::PhantomData};

use crate::{
    heap::{self, Object},
    lower::CompiledFun,
    runtime::NativeFun,
    ty::{BorrowedType, NumericType},
    vm::RuntimeError,
};

#[derive(Debug, Clone, Copy)]
pub enum Value<'m, 's> {
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
    NativeFun(NativeFun),
    CompiledFun(&'s CompiledFun<'s>),
    Array(Object<'m, heap::Array<'s>>),
}

#[derive(Debug, Clone, Copy)]
pub enum StaticValue<'s> {
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
    NativeFun(NativeFun),
    CompiledFun(&'s CompiledFun<'s>),
}

impl<'m, 's> From<StaticValue<'s>> for Value<'m, 's> {
    fn from(value: StaticValue<'s>) -> Self {
        match value {
            StaticValue::Int(value) => Value::Int(value),
            StaticValue::Float(value) => Value::Float(value),
            StaticValue::Bool(value) => Value::Bool(value),
            StaticValue::Unit => Value::Unit,
            StaticValue::NativeFun(fun) => Value::NativeFun(fun),
            StaticValue::CompiledFun(fun) => Value::CompiledFun(fun),
        }
    }
}

pub fn value_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Unit, Value::Unit) => true,
        _ => panic!(),
    }
}

pub struct Array<'m, 's, T> {
    object: Object<'m, heap::Array<'s>>,
    phantom: PhantomData<T>,
}

pub trait ValueConv<'m, 's> {
    const TYPE: BorrowedType<'static>;

    fn into_value(self) -> Value<'m, 's>;
    fn from_value(value: Value<'m, 's>) -> Self;
}

pub trait NativeFunResult<'m, 's> {
    type Output: ValueConv<'m, 's>;
    fn map(self) -> Result<Value<'m, 's>, RuntimeError>;
}

impl<'m, 's> Value<'m, 's> {
    pub fn as_int(self) -> i64 {
        match self {
            Value::Int(value) => value,
            _ => panic!("{}", self),
        }
    }
    pub fn as_float(self) -> f64 {
        match self {
            Value::Float(value) => value,
            _ => panic!(),
        }
    }
    pub fn as_bool(self) -> bool {
        match self {
            Value::Bool(value) => value,
            _ => panic!(),
        }
    }
    pub fn as_array(self) -> Object<'m, heap::Array<'s>> {
        match self {
            Value::Array(value) => value,
            _ => panic!(),
        }
    }
}

impl<'m, 's, T: ValueConv<'m, 's>> NativeFunResult<'m, 's> for Result<T, RuntimeError> {
    type Output = T;
    fn map(self) -> Result<Value<'m, 's>, RuntimeError> {
        self.map(|t| t.into_value())
    }
}

impl<'m, 's> ValueConv<'m, 's> for i64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Int);
    fn into_value(self) -> Value<'m, 's> {
        Value::Int(self)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        value.as_int()
    }
}

impl<'m, 's> ValueConv<'m, 's> for bool {
    const TYPE: BorrowedType<'static> = BorrowedType::Bool;
    fn into_value(self) -> Value<'m, 's> {
        Value::Bool(self)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        value.as_bool()
    }
}

impl<'m, 's> ValueConv<'m, 's> for f64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Float);
    fn into_value(self) -> Value<'m, 's> {
        Value::Float(self)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        value.as_float()
    }
}

impl<'m, 's> ValueConv<'m, 's> for () {
    const TYPE: BorrowedType<'static> = BorrowedType::Unit;
    fn into_value(self) -> Value<'m, 's> {
        Value::Unit
    }
    fn from_value(_: Value<'m, 's>) -> Self {}
}

impl<'m, 's> ValueConv<'m, 's> for Infallible {
    const TYPE: BorrowedType<'static> = BorrowedType::Void;
    fn into_value(self) -> Value<'m, 's> {
        match self {}
    }
    fn from_value(_: Value<'m, 's>) -> Self {
        unreachable!()
    }
}

impl<'m, 's, T: ValueConv<'m, 's>> ValueConv<'m, 's> for Array<'m, 's, T> {
    const TYPE: BorrowedType<'static> = BorrowedType::Array(&T::TYPE);
    fn into_value(self) -> Value<'m, 's> {
        Value::Array(self.object)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        Array {
            object: value.as_array(),
            phantom: PhantomData,
        }
    }
}

impl<'m, 's, T: ValueConv<'m, 's>> Array<'m, 's, T> {
    pub fn len(&self) -> i64 {
        self.object.len() as i64
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: i64) -> T {
        let p = self.object.get(index as usize);
        T::from_value(p)
    }
    pub fn set(&self, index: i64, value: T) {
        self.object.set(index as usize, value.into_value());
    }
}

impl<'m, 's> fmt::Display for Value<'m, 's> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Unit => write!(f, "()"),
            Value::NativeFun(_) => write!(f, "fnptr"),
            Value::CompiledFun(fun) => write!(f, "{}", fun.name),
            Value::Array(array) => {
                write!(f, "[")?;
                for i in 0..array.len() {
                    write!(f, "{}", array.get(i))?;
                    if i != array.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}
