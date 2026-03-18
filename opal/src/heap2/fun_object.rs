use crate::{
    heap2::{HeapObject, ObjectRef},
    instr::Instr,
    value::Value,
};

pub struct FunObjectRef<'h>(ObjectRef<'h>);

impl<'h> HeapObject<'h> for FunObjectRef<'h> {
    fn size(&self) -> usize {
        todo!()
    }
    fn trace(&self) {
        todo!()
    }
}

impl<'h> FunObjectRef<'h> {
    pub fn instrs(&self) -> &[Instr] {
        todo!()
    }
    pub fn consts(&self) -> &[Value<'static>] {
        todo!()
    }
}
