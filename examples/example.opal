module main;

fun main() {
    let nums = [1, 2, 3];
    let total = 0;
    let i = 0;
    while (i < len(nums)) {
        total += nums[i];
        i += 1;
    }
    print(total);
}
