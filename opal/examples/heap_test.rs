use opal::heap2::{FunObjectRef, Heap, HeapObject};

fn main() {
    let heap = Heap::init().expect("could not create heap");

    let object = heap.alloc::<FunObjectRef>(());

    let instrs = object.instrs();

    let x = object.upcast();
    let p = x.downcast::<FunObjectRef>().unwrap();
}
