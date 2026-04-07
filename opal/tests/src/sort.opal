module sort;

fun test_sort() {
    let array = [9, 3, 4, 7, 1];
    sort(array);
    assert(array == [1, 3, 4, 7, 9]);
}

fun sort(array: Array[Int]) {
    let i = 0;
    while (i < len(array)) {
        let j = i + 1;
        while (j < len(array)) {
            if (array[i] > array[j]) {
                let temp = array[i];
                array[i] := array[j];
                array[j] := temp;
            }
            j += 1;
        }
        i += 1;
    }
}
