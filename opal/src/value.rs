use std::{convert::Infallible, fmt, marker::PhantomData, mem, ptr};

use strum::{EnumIs, FromRepr};

use crate::{
    heap::{self, Object},
    lower::CompiledFun,
    runtime::NativeFun,
    ty::{BorrowedType, NumericType},
    vm::RuntimeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs, FromRepr)]
#[repr(u8)]
pub enum ValueTag {
    Undefined,
    Int,
    Float,
    Bool,
    Unit,
    Array,
    HostFun,
    Fun,
}

#[derive(Debug, Clone, Copy)]
pub struct Value<'s, 'm> {
    tag: ValueTag,
    data: *mut (),
    _phantom: PhantomData<(&'s (), &'m ())>,
}

#[derive(Debug, Clone, Copy)]
pub struct StaticValue<'s> {
    tag: ValueTag,
    data: *mut (),
    _phantom: PhantomData<&'s ()>,
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

#[cfg(target_pointer_width = "64")]
type Int = i64;

#[cfg(target_pointer_width = "32")]
type Int = i32;

#[cfg(target_pointer_width = "64")]
type Float = f64;

#[cfg(target_pointer_width = "32")]
type Float = f32;

impl<'m, 's> TryFrom<Value<'m, 's>> for StaticValue<'s> {
    type Error = ();

    fn try_from(value: Value<'m, 's>) -> Result<Self, ()> {
        if value.tag == ValueTag::Array {
            return Err(());
        }
        Ok(StaticValue {
            tag: value.tag,
            data: value.data,
            _phantom: PhantomData,
        })
    }
}

impl<'m, 's> From<StaticValue<'s>> for Value<'m, 's> {
    fn from(value: StaticValue<'s>) -> Self {
        Value {
            tag: value.tag,
            data: value.data,
            _phantom: PhantomData,
        }
    }
}

impl<'m, 's> Value<'m, 's> {
    pub unsafe fn new(tag: ValueTag, data: *mut ()) -> Value<'m, 's> {
        assert!(tag != ValueTag::Undefined);
        Value {
            tag,
            data,
            _phantom: PhantomData,
        }
    }
    pub fn data(&self) -> *mut () {
        self.data
    }
    pub fn tag(&self) -> ValueTag {
        self.tag
    }
    pub fn int(value: Int) -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::Int, ptr::without_provenance_mut(value as usize)) }
    }
    pub fn float(value: Float) -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::Float, ptr::without_provenance_mut(value as usize)) }
    }
    pub fn bool(value: bool) -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::Bool, ptr::without_provenance_mut(if value { 1 } else { 0 })) }
    }
    pub fn unit() -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::Unit, ptr::without_provenance_mut(0)) }
    }
    pub fn array(array: Object<'m, heap::Array<'m>>) -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::Array, array.as_ptr()) }
    }
    pub fn fun(fun: &CompiledFun) -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::Fun, ptr::from_ref(fun).cast::<()>().cast_mut()) }
    }
    pub fn host_fun(fun: NativeFun) -> Value<'m, 's> {
        unsafe { Value::new(ValueTag::HostFun, fun as *mut ()) }
    }
    pub fn as_int(self) -> Int {
        assert!(self.tag.is_int());
        self.data.addr() as Int
    }
    pub fn as_float(self) -> Float {
        assert!(self.tag.is_float());
        self.data.addr() as Float
    }
    pub fn as_bool(self) -> bool {
        assert!(self.tag.is_bool());
        self.data.addr() == 1
    }
    pub fn as_array(self) -> Object<'m, heap::Array<'s>> {
        assert!(self.tag.is_array());
        unsafe { Object::from_ptr(self.data.cast()) }
    }
    pub fn as_host_fun(&self) -> NativeFun {
        assert!(self.tag.is_host_fun());
        unsafe { mem::transmute::<*mut (), NativeFun>(self.data) }
    }
    pub fn as_fun(&self) -> &'s CompiledFun {
        assert!(self.tag.is_host_fun());
        unsafe { self.data.cast::<CompiledFun<'s>>().as_ref_unchecked() }
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
        Value::int(self)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        value.as_int()
    }
}

impl<'m, 's> ValueConv<'m, 's> for bool {
    const TYPE: BorrowedType<'static> = BorrowedType::Bool;
    fn into_value(self) -> Value<'m, 's> {
        Value::bool(self)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        value.as_bool()
    }
}

impl<'m, 's> ValueConv<'m, 's> for f64 {
    const TYPE: BorrowedType<'static> = BorrowedType::Numeric(NumericType::Float);
    fn into_value(self) -> Value<'m, 's> {
        Value::float(self)
    }
    fn from_value(value: Value<'m, 's>) -> Self {
        value.as_float()
    }
}

impl<'m, 's> ValueConv<'m, 's> for () {
    const TYPE: BorrowedType<'static> = BorrowedType::Unit;
    fn into_value(self) -> Value<'m, 's> {
        Value::unit()
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
        Value::array(self.object)
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
        match self.tag {
            ValueTag::Int => write!(f, "{}", self.data.addr() as Int),
            ValueTag::Float => write!(f, "{}", self.data.addr() as Float),
            ValueTag::Bool => write!(f, "{}", self.data.addr() == 1),
            ValueTag::Unit => write!(f, "()"),
            ValueTag::HostFun => write!(f, "host_fun"),
            ValueTag::Fun => write!(f, "fun"),
            ValueTag::Array => {
                let array = self.as_array();
                write!(f, "[")?;
                for i in 0..array.len() {
                    write!(f, "{}", array.get(i))?;
                    if i != array.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            ValueTag::Undefined => panic!("something has gone wrong"),
        }
    }
}
