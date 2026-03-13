use std::{cell::RefCell, marker::PhantomData};

use opal::gc::{Gc, GcRef, GcSlice, Trace};
use opal_proc::{Rootable, Trace};

#[derive(Trace)]
struct Node<'gc, T> {
    prev: RefCell<Option<GcRef<'gc, Node<'gc, T>>>>,
    payload: T,
    next: RefCell<Option<GcRef<'gc, Node<'gc, T>>>>,
}

#[derive(Rootable)]
#[lifetime('gc)]
#[root(Node<'gc, T>)]
struct NodeRoot<T: 'static>(PhantomData<T>);

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

#[derive(Trace, Clone, Copy)]
struct Tree<'gc, T: Trace> {
    payload: T,
    children: GcSlice<'gc, GcRef<'gc, Tree<'gc, T>>>,
}

#[derive(Rootable)]
#[lifetime('gc)]
#[root(Tree<'gc, T>)]
struct TreeRoot<T: Trace + 'static>(PhantomData<T>);

fn main() {
    let mut gc = Gc::init().expect("could not acquire global gc");

    let a = gc.alloc(Tree {
        payload: "A",
        children: gc.alloc_slice(&[]),
    });
    let b = gc.alloc(Tree {
        payload: "B",
        children: gc.alloc_slice(&[]),
    });
    let c = gc.alloc(Tree {
        payload: "C",
        children: gc.alloc_slice(&[a, b]),
    });

    let data: GcSlice<'_, i32> = gc.alloc_slice(&[1, 2, 3]);

    // let a = gc.alloc(Node::new("A"));
    // let b = gc.alloc(Node::new("B"));
    // let c = gc.alloc(Node::new("C"));

    // a.set_next(Some(b));
    // b.set_next(Some(c));
    // b.set_prev(Some(a));
    // c.set_prev(Some(b));

    let root = gc.root::<TreeRoot<_>>(c);

    println!("{}", gc.allocation_count());

    gc.collect();

    println!("{}", gc.allocation_count());
}

// garbage collector todo:
// mark + sweep phases [DONE]
// derive macro for trace [DONE]
// slice types [DONE]
// derive macro for root [DONE]
// optimisation to skip elements with no gc refs [DONE]
// drop for gc type
// trace for enums
