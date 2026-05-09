//void starttime(void); void stoptime(void);

int r = 0;

int fib_iter(int n) {
    int a = 0;
    int b = 1;
    int c = 0;
    int i = 0;

    while (i < n) {
        c = a + b;
        a = b;
        b = c;
        i = i + 1;
    }

    return a;
}

int workload(int rounds, int seed) {
    int acc = seed;
    int i = 0;
    int n = 0;

    while (i < rounds) {
        n = 30 + (acc % 8);
        acc = acc + fib_iter(n);
        i = i + 1;
    }

    return acc;
}

int main() {
    starttime();
    r = workload(100000, 1);
    stoptime();

    starttime();
    r = workload(100000, 3);
    stoptime();

    starttime();
    r = workload(100000, 5);
    stoptime();

    starttime();
    r = workload(100000, 7);
    stoptime();

    starttime();
    r = workload(100000, 9);
    stoptime();

    return 0;
}
