module sort;

fun test_sort() {
    let array = [9, 3, 4, 7, 1];
    sort(array);
    assert(array_eq(array, [1, 3, 4, 7, 9]));
}

fun sort(array: Array[Int]) {
    let i = 0;
    while (i < len(array)) {
        let j = i + 1;
        while (j < len(array)) {
            if (array[i] > array[j]) {
                let temp = array[i];
                array[i] = array[j];
                array[j] = temp;
            }
            j += 1;
        }
        i += 1;
    }
}

fun test_array_eq() {
    assert(array_eq([1, 2, 3], [1, 2, 3]));
}

fun test_array_not_eq() {
    if (array_eq([1, 2, 4], [1, 2, 3])) {
        fail();
    }
}

fun test_array_diff_len() {
    if (array_eq([1, 2], [1, 2, 3])) {
        fail();
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
