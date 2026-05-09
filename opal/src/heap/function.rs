use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{
    heap::object::ObjectTrait,
    instr::Instr,
    value::{Value, ValueTag},
};

#[derive(Debug)]
pub struct Function {
    pub frame_size: u8,
    pub bytecode: Box<[Instr]>,
    pub(super) constants_tags: Box<[ValueTag]>,
    pub(super) constants_data: Box<[AtomicPtr<()>]>,
}

impl ObjectTrait for Function {
    type Item = ();
}

impl Function {
    pub fn constants(&self) -> Constants<'_> {
        Constants {
            index: 0,
            tags: &self.constants_tags,
            data: &self.constants_data,
        }
    }
    pub fn len(&self) -> usize {
        self.constants_tags.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn set_constant(&self, index: usize, value: Value<'_>) {
        let (tag, data) = value.to_raw_parts();
        if tag != self.constants_tags[index] {
            panic!(
                "cannot set invalid type, was: {:?}, expecting: {:?}",
                tag, self.constants_tags[index]
            );
        }
        self.constants_data[index].store(data, Ordering::Relaxed);
    }
    pub fn get_constant(&self, index: usize) -> Value<'_> {
        unsafe {
            Value::from_raw_parts(
                self.constants_tags[index],
                self.constants_data[index].load(Ordering::Relaxed),
            )
        }
    }
}

pub struct Constants<'h> {
    index: usize,
    tags: &'h [ValueTag],
    data: &'h [AtomicPtr<()>],
}

impl<'h> Iterator for Constants<'h> {
    type Item = Value<'h>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tags.len() {
            return None;
        }
        let tag = self.tags[self.index];
        let data = self.data[self.index].load(Ordering::Relaxed);
        self.index += 1;
        unsafe { Some(Value::from_raw_parts(tag, data)) }
    }
}
