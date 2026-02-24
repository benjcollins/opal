module basic;

// line comment /*
    block comment inside of line comment
*/ still a comment

/* block comment
    // line comment inside of block comment
*/

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

fun test_int_comp_branch_invert() {
    while (5 == 2) { fail(); }
    while (5 != 5) { fail(); }
    while (5 < 2) { fail(); }
    while (2 > 5) { fail(); }
    while (5 >= 9) { fail(); }
    while (5 <= 2) { fail(); }
}

fun test_float_comp_branch() {
    if (5.0 == 2.0) { fail(); }
    if (5.0 != 5.0) { fail(); }
    if (5.0 < 2.0) { fail(); }
    if (2.0 > 5.0) { fail(); }
    if (5.0 >= 9.0) { fail(); }
    if (5.0 <= 2.0) { fail(); }
}

fun test_float_comp_branch_invert() {
    while (5.0 == 2.0) { fail(); }
    while (5.0 != 5.0) { fail(); }
    while (5.0 < 2.0) { fail(); }
    while (2.0 > 5.0) { fail(); }
    while (5.0 >= 9.0) { fail(); }
    while (5.0 <= 2.0) { fail(); }
}

fun test_logical_op() {
    assert(true && true);
    if (true && false) { fail(); }
    if (false && true) { fail(); }
    if (false && false) { fail(); }

    assert(true || true);
    assert(true || false);
    assert(false || true);
    if (false || false) { fail(); }
}

fun test_logical_op_invert() {
    while (true && false) { fail(); }
    while (false && true) { fail(); }
    while (false && false) { fail(); }
    while (false || false) { fail(); }

    let x = true;
    while (x && x) { x := false; }
    assert(x == false);

    let x = true;
    while (x || x) { x := false; }
    assert(x == false);

    let x = true;
    while (x || false) { x := false; }
    assert(x == false);

    let x = true;
    while (false || x) { x := false; }
    assert(x == false);
}

fun test_branch() {
    let b = false;
    if (b) {
        fail();
    }
}

fun test_prefix_ops() {
    assert(!false);
    assert(~5 == -6);
    assert(+5 == 5);
    assert(3.0 - 5.0 == -2.0);
    assert(3 - 5 == -2);
}

fun test_unit() {
    ();
}
