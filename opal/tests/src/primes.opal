module primes;

fun generate_sieve(size: Int) -> List[Bool] {
    let sieve = [false; size];
    let i = 2;
    while (i < len(sieve)) {
        let j = i * 2;
        while (j < len(sieve)) {
            sieve[j] := true;
            j += i;
        }
        i += 1;
    }
    return sieve;
}

fun count_primes(sieve: List[Bool]) -> Int {
    let i = 1;
    let prime_count = 0;
    while (i < len(sieve)) {
        if (!sieve[i]) {
            prime_count += 1;
        }
        i += 1;
    }
    return prime_count;
}

fun collect_primes(sieve: List[Bool]) -> List[Int] {
    let prime_count = count_primes(sieve);
    let primes = [0; prime_count];
    let i = 1;
    let j = 0;
    while (i < len(sieve)) {
        if (!sieve[i]) {
            primes[j] := i;
            j += 1;
        }
        i += 1;
    }
    return primes;
}

fun primes(n: Int) -> List[Int] {
    let sieve = generate_sieve(n);
    return collect_primes(sieve);
}

fun test_primes_10() {
    let primes = primes(10);
    assert(primes == [1, 2, 3, 5, 7]);
}
