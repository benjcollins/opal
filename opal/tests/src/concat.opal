module concat;

fun test_concat() {
    var a = [1, 2, 3];
    var b = [4, 5];
    var c = concat(a, b);
    assert(c == [1, 2, 3, 4, 5]);
}

fun concat(a: List[Int], b: List[Int]) -> List[Int] {
    var c = [0; len(a) + len(b)];
    copy(a, c, 0, 0, len(a));
    copy(b, c, 0, len(a), len(b));
    return c;
}

fun copy(src: List[Int], dst: List[Int], src_offset: Int, dst_offset: Int, length: Int) {
    var i = 0;
    while (i < length) {
        dst[dst_offset + i] := src[src_offset + i];
        i += 1;
    }
}
