use std::{convert::Infallible, fmt, marker::PhantomData, mem, ptr};

use strum::{EnumIs, FromRepr};

use crate::{
    heap::{function::Function, list, object::Object},
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
    List(&'h Object<list::List>),

    HostFun(HostFun),
    VMFun(&'h Object<Function>),
    UnpatchedFun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs, FromRepr)]
#[repr(u8)]
pub enum ValueTag {
    Int,
    Float,
    Bool,
    Unit,
    List,
    Fun,
    Void,
}

pub struct List<'h, T> {
    object: &'h Object<list::List>,
    phantom: PhantomData<T>,
}

pub trait ValueConv<'h> {
    const TYPE: BorrowedType<'static>;

    fn into(self) -> Value<'h>;
    fn from(value: Value<'h>) -> Self;
}

pub trait HostFunResult<'h> {
    type Output: ValueConv<'h>;
    fn map(self) -> Result<Value<'h>, RuntimeError>;
}

impl<'h> Value<'h> {
    pub(crate) fn to_raw_parts(self) -> (ValueTag, *mut ()) {
        match self {
            Value::Int(value) => (ValueTag::Int, ptr::without_provenance_mut(value as usize)),
            Value::Float(value) => (ValueTag::Float, ptr::without_provenance_mut(value.to_bits() as usize)),
            Value::Bool(value) => (ValueTag::Bool, ptr::without_provenance_mut(if value { 1 } else { 0 })),
            Value::Unit => (ValueTag::Unit, ptr::null_mut()),
            Value::List(object) => (ValueTag::List, ptr::from_ref(object).cast::<()>().cast_mut()),

            Value::HostFun(value) => (ValueTag::Fun, (value as *mut ()).map_addr(|addr| addr | 1)),
            Value::VMFun(object) => (ValueTag::Fun, ptr::from_ref(object).cast::<()>().cast_mut()),
            Value::UnpatchedFun => (ValueTag::Fun, ptr::null_mut()),
        }
    }
    pub(crate) unsafe fn from_raw_parts(tag: ValueTag, data: *mut ()) -> Value<'h> {
        match tag {
            ValueTag::Int => Value::Int(data.addr() as i64),
            ValueTag::Float => Value::Float(f64::from_bits(data.addr() as u64)),
            ValueTag::Bool => Value::Bool(data.addr() == 1),
            ValueTag::Unit => Value::Unit,
            ValueTag::List => Value::List(unsafe { data.cast::<Object<list::List>>().as_ref_unchecked() }),
            ValueTag::Fun => {
                if data.addr() == 0 {
                    Value::UnpatchedFun
                } else if data.addr() & 1 != 0 {
                    Value::HostFun(unsafe { mem::transmute::<*mut (), HostFun>(data.map_addr(|addr| addr ^ 1)) })
                } else {
                    Value::VMFun(unsafe { data.cast::<Object<Function>>().as_ref_unchecked() })
                }
            }
            ValueTag::Void => panic!(),
        }
    }
}

impl<'h> fmt::Display for Value<'h> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Unit => write!(f, "()"),
            Value::List(list) => {
                write!(f, "[")?;
                for i in 0..list.len() {
                    write!(f, "{}", list.get(i))?;
                    if i + 1 != list.len() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Value::HostFun(_) => write!(f, "<host fun>"),
            Value::VMFun(_) => write!(f, "<vm fun>"),
            Value::UnpatchedFun => write!(f, "<null fun>"),
        }
    }
}

impl<'h, T: ValueConv<'h>> HostFunResult<'h> for Result<T, RuntimeError> {
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

impl<'h, T: ValueConv<'h>> ValueConv<'h> for List<'h, T> {
    const TYPE: BorrowedType<'static> = BorrowedType::Array(&T::TYPE);
    fn into(self) -> Value<'h> {
        Value::List(self.object)
    }
    fn from(value: Value<'h>) -> Self {
        let Value::List(object) = value else {
            panic!("{:?}", value)
        };
        List {
            object,
            phantom: PhantomData,
        }
    }
}

impl<'h, T: ValueConv<'h>> List<'h, T> {
    pub fn len(&self) -> usize {
        self.object.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: i64) -> T {
        let p = self.object.get(index as usize);
        T::from(p)
    }
    pub fn set(&self, index: i64, value: T) {
        self.object.set(index as usize, value.into());
    }
}
