module shadowing;

fun test_shadowing() {
    let x = 3;
    if (true) {
        let x = true;
        assert(x);
    }
    assert(x == 3);
}
