use std::sync::atomic::{AtomicPtr, Ordering};

use crate::value::{Value, ValueTag};

#[derive(Debug)]
pub struct List {
    pub(super) value_tag: ValueTag,
    pub(super) elements: Vec<AtomicPtr<()>>,
}

impl List {
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: usize) -> Value<'_> {
        unsafe { Value::from_raw_parts(self.value_tag, self.elements[index].load(Ordering::Relaxed)) }
    }
    pub fn set(&self, index: usize, value: Value<'_>) {
        let (tag, data) = value.to_raw_parts();
        if tag != self.value_tag {
            panic!("cannot set invalid type");
        }
        self.elements[index].store(data, Ordering::Relaxed);
    }
    pub fn iter(&self) -> ListIter<'_> {
        ListIter {
            index: 0,
            tag: self.value_tag,
            elements: &self.elements,
        }
    }
}

pub struct ListIter<'h> {
    index: usize,
    tag: ValueTag,
    elements: &'h [AtomicPtr<()>],
}

impl<'h> IntoIterator for &'h List {
    type Item = Value<'h>;

    type IntoIter = ListIter<'h>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'h> Iterator for ListIter<'h> {
    type Item = Value<'h>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.elements.len() {
            return None;
        }
        let value = unsafe { Value::from_raw_parts(self.tag, self.elements[self.index].load(Ordering::Relaxed)) };
        self.index += 1;
        Some(value)
    }
}
