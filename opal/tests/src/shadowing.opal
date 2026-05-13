module shadowing;

fun test_shadowing() {
    var x = 3;
    if (true) {
        var x = true;
        assert(x);
    }
    assert(x == 3);
}
