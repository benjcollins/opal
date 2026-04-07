module fcf;

fun map_int(input: Array[Int], mapping: fun(Int) -> Int) -> Array[Int] {
    let output = [0; len(input)];
    let i = 0;
    while (i < len(input)) {
        output[i] := mapping(input[i]);
        i += 1;
    }
    return output;
}

fun filter_int(input: Array[Int], filterer: fun(Int) -> Bool) -> Array[Int] {
    let temp = [0; len(input)];
    let i = 0;
    let j = 0;
    while (i < len(input)) {
        if (filterer(input[i])) {
            temp[j] := input[i];
            j += 1;
        }
        i += 1;
    }
    let output = [0; j];
    let i = 0;
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
    let input = [1, 2, 3];
    let output = map_int(input, double);
    assert(output == [2, 4, 6]);
}

fun test_filter() {
    let input = [1, 2, 3, 4, 5, 6];
    let output = filter_int(input, is_even);
    assert(output == [2, 4, 6]);
}