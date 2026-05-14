module fcf;

fun map[T](input: List[T], mapping: fun(T) -> T, default: T) -> List[T] {
    var output = [default; len(input)];
    var i = 0;
    while (i < len(input)) {
        output[i] := mapping(input[i]);
        i += 1;
    }
    return output;
}

fun filter[T](input: List[T], cond: fun(T) -> Bool, default: T) -> List[T] {
    var temp = [default; len(input)];
    var i = 0;
    var j = 0;
    while (i < len(input)) {
        if (cond(input[i])) {
            temp[j] := input[i];
            j += 1;
        }
        i += 1;
    }
    var output = [default; j];
    var i = 0;
    while (i < j) {
        output[i] := temp[i];
        i += 1;
    }
    return output;
}

fun is_even(n: Int) -> Bool {
    return n % 2 == 0;
}

fun double(n: Int) -> Int {
    return n * 2;
}

fun test_double() {
    var input = [1, 2, 3];
    var output = map(input, double, 0);
    assert(output == [2, 4, 6]);
}

fun test_filter() {
    var input = [1, 2, 3, 4, 5, 6];
    var output = filter(input, is_even, 0);
    assert(output == [2, 4, 6]);
}
