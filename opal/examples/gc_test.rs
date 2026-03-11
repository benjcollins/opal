use std::{cell::RefCell, marker::PhantomData};

use opal::gc::{Gc, GcRef, Rootable, Trace};
use opal_proc::Trace;

#[derive(Trace)]
struct Node<'gc, T: Trace> {
    prev: RefCell<Option<GcRef<'gc, Node<'gc, T>>>>,
    payload: T,
    next: RefCell<Option<GcRef<'gc, Node<'gc, T>>>>,
}

struct NodeRoot<T: Trace>(PhantomData<T>);

impl<T: Trace> Rootable for NodeRoot<T> {
    type Root<'gc>
        = Node<'gc, T>
    where
        T: 'gc;
}

impl<'gc, T: Trace> Node<'gc, T> {
    fn new(payload: T) -> Node<'gc, T> {
        Node {
            prev: RefCell::new(None),
            payload,
            next: RefCell::new(None),
        }
    }
    fn set_prev(&self, prev: Option<GcRef<'gc, Node<'gc, T>>>) {
        *self.prev.borrow_mut() = prev;
    }
    fn set_next(&self, next: Option<GcRef<'gc, Node<'gc, T>>>) {
        *self.next.borrow_mut() = next;
    }
}

fn main() {
    let mut gc = Gc::init().expect("could not acquire global gc");

    let a = gc.alloc(Node::new("A"));
    let b = gc.alloc(Node::new("B"));
    let c = gc.alloc(Node::new("C"));

    a.set_next(Some(b));
    b.set_next(Some(c));
    b.set_prev(Some(a));
    c.set_prev(Some(b));

    let root = gc.root::<NodeRoot<&'static str>>(b);

    println!("{}", gc.allocation_count());

    gc.collect();

    println!("{}", gc.allocation_count());
}

// garbage collector todo:
// mark + sweep phases [DONE]
// derive macro for trace [DONE]
// drop for gc type
// array types
// derive macro for root
