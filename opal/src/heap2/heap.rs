use std::{
    ptr,
    sync::{
        Once,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    },
};

use crate::{
    heap2::object::{Function, List, Object, ObjectHeader, ObjectTag, Stack},
    instr::Instr,
    value::{Value, ValueTag},
};

pub const HEAP_ALIGN: usize = size_of::<usize>();

pub struct Heap {
    pub objects_head: AtomicPtr<ObjectHeader>,
}

impl Heap {
    pub fn init() -> Option<Heap> {
        static ONCE: Once = Once::new();
        let mut first = false;
        ONCE.call_once(|| {
            first = true;
        });
        first.then(|| Heap {
            objects_head: AtomicPtr::null(),
        })
    }
    fn alloc<T>(&self, tag: ObjectTag, body: T) -> &Object<T> {
        unsafe {
            let header = ObjectHeader {
                next: AtomicPtr::null(),
                handle_count: AtomicU32::new(0),
                tag,
                marked: AtomicBool::new(false),
            };
            let object_ptr = Box::into_raw(Box::new(Object { header, body }));
            let header_ptr = object_ptr.cast::<ObjectHeader>();

            let mut objects_head = self.objects_head.load(Ordering::Relaxed);
            loop {
                match self
                    .objects_head
                    .compare_exchange(objects_head, header_ptr, Ordering::Release, Ordering::Relaxed)
                {
                    Ok(_) => break,
                    Err(new_head) => objects_head = new_head,
                }
            }
            (*header_ptr).next.store(objects_head, Ordering::Relaxed);
            object_ptr.as_ref_unchecked()
        }
    }
    pub fn alloc_function(&self, bytecode: &[Instr], constants: &[Value], frame_size: u8) -> &Object<Function> {
        let (constants_tag, constants_data): (Vec<_>, Vec<_>) = constants
            .iter()
            .map(|value| {
                let (tag, data) = value.to_raw_parts();
                (tag, AtomicPtr::new(data))
            })
            .unzip();
        self.alloc(
            ObjectTag::Function,
            Function {
                bytecode: bytecode.into(),
                constants_tag: constants_tag.into(),
                constants_data: constants_data.into(),
                frame_size,
            },
        )
    }
    pub fn alloc_array_elements(&self, elements: &[Value]) -> &Object<List> {
        let (value_tag, elements) = if elements.is_empty() {
            (ValueTag::Void, vec![])
        } else {
            let (first_element_tag, _) = elements[0].to_raw_parts();
            let mut data = vec![];
            for element in elements {
                let (element_tag, element_data) = element.to_raw_parts();
                if element_tag != first_element_tag {
                    panic!("elements not all the same type")
                }
                data.push(AtomicPtr::new(element_data));
            }
            (first_element_tag, data)
        };
        self.alloc(ObjectTag::Array, List { value_tag, elements })
    }
    pub fn alloc_array_default_size(&self, default: Value, size: usize) -> &Object<List> {
        let (tag, data) = default.to_raw_parts();
        self.alloc(
            ObjectTag::Array,
            List {
                value_tag: tag,
                elements: (0..size).map(|_| AtomicPtr::new(data)).collect(),
            },
        )
    }
    // pub fn alloc_stack(&self) -> &Object<Stack> {
    //     let (tag, data) = default.to_raw_parts();
    //     self.alloc(
    //         ObjectTag::Array,
    //         List {
    //             value_tag: tag,
    //             elements: (0..size).map(|_| AtomicPtr::new(data)).collect(),
    //         },
    //     )
    // }
    pub fn collect(&mut self) {
        unsafe {
            unmark_objects_without_handles(*self.objects_head.get_mut());
            dealloc_unmarked_objects(self.objects_head.get_mut());
        }
    }
}

unsafe fn unmark_objects_without_handles(mut current: *mut ObjectHeader) {
    unsafe {
        while !current.is_null() {
            *(*current).marked.get_mut() = *(*current).handle_count.get_mut() > 0;
            current = *(*current).next.get_mut();
        }
    }
}

unsafe fn dealloc_unmarked_objects(head: &mut *mut ObjectHeader) {
    unsafe {
        let mut current = *head;
        let mut prev: *mut ObjectHeader = ptr::null_mut();

        while !current.is_null() {
            let next = *(*current).next.get_mut();
            if !*(*current).marked.get_mut() {
                if !prev.is_null() {
                    *(*prev).next.get_mut() = next;
                } else {
                    *head = next;
                }
                match (*current).tag {
                    ObjectTag::Function => drop(Box::from_raw(current.cast::<Object<Function>>())),
                    ObjectTag::Stack => drop(Box::from_raw(current.cast::<Object<Stack>>())),
                    ObjectTag::Array => drop(Box::from_raw(current.cast::<Object<List>>())),
                }
            } else {
                prev = current;
            }
            current = next;
        }
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        self.collect();
    }
}
