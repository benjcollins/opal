module concat;

fun test_concat() {
    let a = [1, 2, 3];
    let b = [4, 5];
    let c = concat(a, b);
    assert(c == [1, 2, 3, 4, 5]);
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
