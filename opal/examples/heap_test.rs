use opal::{heap::Heap, value::Value};

fn main() {
    let mut heap = Heap::init().expect("could not create heap!");

    let function = heap.alloc_function(&[], &[Value::Int(10)], 0);
    let array = heap.alloc_list_elements(&[Value::Int(5), Value::Int(10)]);
    let stack = heap.alloc_stack(function).to_handle();

    function.set_constant(0, Value::Int(5));
    array.set(0, Value::Int(10));

    let function = function.to_handle();

    heap.collect();

    println!("{}", function.to_object(&heap).get_constant(0));
}
