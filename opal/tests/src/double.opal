module double;

fun test_double() {
    let array = [1, 2, 3];

    double(array);

    assert(array[0] == 2);
    assert(array[1] == 4);
    assert(array[2] == 6);
}

fun double(array: Array[Int]) {
    let i = 0;
    while (i < len(array)) {
        array[i] *= 2;
        i += 1;
    }
}
