module concat;

fun test_concat() {
    let a = [1, 2, 3];
    let b = [4, 5];
    let c = concat(a, b);
    assert(array_eq(c, [1, 2, 3, 4, 5]));
}

fun concat(a: Array[Int], b: Array[Int]) -> Array[Int] {
    let c = [0; len(a) + len(b)];
    copy(a, c, 0, 0, len(a));
    copy(b, c, 0, len(a), len(b));
    return c;
}

fun copy(src: Array[Int], dst: Array[Int], src_offset: Int, dst_offset: Int, length: Int) {
    let i = 0;
    while (i < length) {
        dst[dst_offset + i] := src[src_offset + i];
        i += 1;
    }
}

fun array_eq(array1: Array[Int], array2: Array[Int]) -> Bool {
    if (len(array1) != len(array2)) {
        return false;
    }
    let i = 0;
    while (i < len(array1)) {
        if (array1[i] != array2[i]) {
            return false;
        }
        i += 1;
    }
    return true;
}
