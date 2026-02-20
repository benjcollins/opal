module many_args;

fun test_many_args() {
    many_args(1 + 1 + 3, 2, 9);
}

fun many_args(a: Int, b: Int, c: Int) {
    assert(a == 5);
    assert(b == 2);
    assert(c == 9);
}
