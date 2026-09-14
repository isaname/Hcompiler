int fib(int n) {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int gcd(int u, int v) {
    if (v == 0) {
        return u;
    }
    return gcd(v, u % v);
}

int power(int base, int exp) {
    if (exp == 0) {
        return 1;
    }
    return base * power(base, exp - 1);
}

int main() {
    output(fib(15));
    output(gcd(84, 36));
    output(gcd(17, 5));
    output(power(3, 7));
    return 0;
}
