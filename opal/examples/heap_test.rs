use opal::{heap::Heap, value::Value};

fn main() {
    let heap = Heap::new().expect("could not create heap!");
    let mut stack = heap.new_stack();

    heap.with_mutator(|mutator| {
        let array = mutator.alloc_array(2);
        stack.set(0, Value::array(array), &mutator);
    });

    heap.collect_garabge();

    heap.with_mutator(|mutator| {
        let _ = stack.get(0, mutator).as_array();
    });
}
