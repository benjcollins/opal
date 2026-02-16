module basic;

fun test_int_arith() {
    assert(5 + 2 == 7);
    assert(5 - 2 == 3);
    assert(5 * 2 == 10);
    assert(4 / 2 == 2);
    assert(5 % 2 == 1);
}

fun test_float_arith() {
    assert(5.0 + 2.0 == 7.0);
    assert(5.0 - 2.0 == 3.0);
    assert(5.0 * 2.0 == 10.0);
    assert(5.0 / 2.0 == 2.5);
    assert(5.0 % 2.0 == 1.0);
}

fun test_int_comp() {
    assert(5 > 2);
    assert(2 < 5);
    assert(5 >= 5);
    assert(5 <= 5);
    assert(5 == 5);
    assert(5 != 2);
}

fun test_float_comp_set() {
    assert(5.0 > 2.0);
    assert(2.0 < 5.0);
    assert(5.0 >= 5.0);
    assert(5.0 <= 5.0);
    assert(5.0 == 5.0);
    assert(5.0 != 2.0);
}

fun test_int_comp_branch() {
    if (5 == 2) { fail(); }
    if (5 != 5) { fail(); }
    if (5 < 2) { fail(); }
    if (2 > 5) { fail(); }
    if (5 >= 9) { fail(); }
    if (5 <= 2) { fail(); }
}

fun test_float_comp_branch() {
    if (5.0 == 2.0) { fail(); }
    if (5.0 != 5.0) { fail(); }
    if (5.0 < 2.0) { fail(); }
    if (2.0 > 5.0) { fail(); }
    if (5.0 >= 9.0) { fail(); }
    if (5.0 <= 2.0) { fail(); }
}

fun test_branch() {
    let b = false;
    if (b) {
        fail();
    }
}

fun test_unit() {
    ();
}