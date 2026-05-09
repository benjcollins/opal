use std::{
    collections::VecDeque,
    ptr,
    sync::{
        Mutex, Once,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    },
};

use crate::{
    heap::{
        function::Function,
        list::List,
        object::{Object, ObjectHeader, ObjectTag},
        stack::{Stack, StackInner},
    },
    instr::Instr,
    value::{Value, ValueTag},
};

pub mod bytes;
pub mod function;
pub mod handle;
pub mod list;
pub mod object;
pub mod stack;

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
                constants_tags: constants_tag.into(),
                constants_data: constants_data.into(),
                frame_size,
            },
        )
    }
    pub fn alloc_list_elements(&self, elements: &[Value]) -> &Object<List> {
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
        self.alloc(ObjectTag::List, List { value_tag, elements })
    }
    pub fn alloc_list_default_size(&self, default: Value, size: usize) -> &Object<List> {
        let (tag, data) = default.to_raw_parts();
        self.alloc(
            ObjectTag::List,
            List {
                value_tag: tag,
                elements: (0..size).map(|_| AtomicPtr::new(data)).collect(),
            },
        )
    }
    pub fn alloc_stack(&self, function: &Object<Function>) -> &Object<Stack> {
        let (tag, data) = Value::Unit.to_raw_parts();
        self.alloc(
            ObjectTag::Stack,
            Stack(Mutex::new(StackInner {
                call_stack: vec![],
                value_stack_tag: vec![tag; 1024],
                value_stack_data: vec![data; 1024],
                function: ptr::from_ref(function).cast_mut(),
                base_ptr: 0,
                instr_ptr: 0,
            })),
        )
    }
    pub fn collect(&mut self) {
        unsafe {
            let mut work_list = VecDeque::new();
            unmark_objects_without_handles(*self.objects_head.get_mut(), &mut work_list);
            trace_live_objects(&mut work_list);
            dealloc_unmarked_objects(self.objects_head.get_mut());
        }
    }
}

unsafe fn unmark_objects_without_handles(mut current: *mut ObjectHeader, work_list: &mut VecDeque<*mut ObjectHeader>) {
    unsafe {
        while !current.is_null() {
            if *(*current).handle_count.get_mut() > 0 {
                work_list.push_back(current);
                *(*current).marked.get_mut() = true;
            } else {
                *(*current).marked.get_mut() = false;
            }
            current = *(*current).next.get_mut();
        }
    }
}

unsafe fn enqueue_object(object: *mut ObjectHeader, work_list: &mut VecDeque<*mut ObjectHeader>) {
    unsafe {
        if !*(*object).marked.get_mut() {
            *(*object).marked.get_mut() = true;
            work_list.push_back(object);
        }
    }
}

unsafe fn trace_value<'h>(value: Value<'h>, work_list: &mut VecDeque<*mut ObjectHeader>) {
    unsafe {
        let header = ptr::from_ref(match value {
            Value::List(object) => &object.header,
            Value::VMFun(object) => &object.header,
            _ => return,
        })
        .cast_mut();
        enqueue_object(header, work_list);
    }
}

unsafe fn trace_live_objects(work_list: &mut VecDeque<*mut ObjectHeader>) {
    unsafe {
        while let Some(object) = work_list.pop_front() {
            match (*object).tag {
                ObjectTag::Function => {
                    let function = object.cast::<Object<Function>>().as_ref_unchecked();
                    for constant in function.constants() {
                        trace_value(constant, work_list);
                    }
                }
                ObjectTag::List => {
                    let list = object.cast::<Object<List>>().as_ref_unchecked();
                    for element in list.iter() {
                        trace_value(element, work_list);
                    }
                }
                ObjectTag::Stack => {
                    let stack = object.cast::<Object<Stack>>().as_ref_unchecked();
                    let mut stack = stack.lock();
                    for value in stack.values() {
                        trace_value(value, work_list);
                    }
                    for frame in &stack.call_stack {
                        enqueue_object(&raw mut (*frame.function).header, work_list);
                    }
                    enqueue_object(&raw mut (*stack.function).header, work_list);
                }
            }
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
                    ObjectTag::List => drop(Box::from_raw(current.cast::<Object<List>>())),
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
