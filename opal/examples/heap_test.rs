use opal::{heap2::Heap, value::Value};

fn main() {
    let mut heap = Heap::init().expect("could not create heap!");

    let function = heap.alloc_function(&[], &[Value::int(10)], 0);
    let array = heap.alloc_array_elements(&[Value::int(5), Value::int(10)]);

    function.set_constant(0, Value::int(5));
    array.set_element(0, Value::int(10));

    let function = function.to_handle();

    heap.collect();

    println!("{}", function.to_object(&heap).get_constant(0));
}
