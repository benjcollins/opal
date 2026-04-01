use opal::{heap::Heap, value::Value};

fn main() {
    let heap = Heap::new().expect("could not create heap!");

    let mutator = heap.mutator();
    let array = mutator.alloc_array(&[Value::int(2), Value::int(5)]);

    let mut stack = heap.create_stack();

    stack.grow(10, &mutator);
    stack.set(0, Value::array(array), &mutator);
}
