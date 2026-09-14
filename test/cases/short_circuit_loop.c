// 用短路求值搭出一个能被观察到的算法：素数筛 + 累加。
// 短路和 break/continue 都是结果正确的必要条件，
// 任一 bug 回归都会改变输出而不只是崩溃。

int probes;

int is_prime(int n) {
    int d;
    if (n < 2) {
        return 0;
    }
    d = 2;
    while (d * d <= n) {
        probes = probes + 1;
        if (n % d == 0) {
            return 0;
        }
        d = d + 1;
    }
    return 1;
}

// 找出 [lo, hi] 内前 k 个素数之和；用 break 提前收工
int sum_first_primes(int lo, int hi, int k) {
    int n;
    int found;
    int acc;
    n = lo;
    found = 0;
    acc = 0;
    while (n <= hi) {
        if (n < 2 || !is_prime(n)) {
            n = n + 1;
            continue;
        }
        acc = acc + n;
        found = found + 1;
        if (found >= k) {
            break;
        }
        n = n + 1;
    }
    return acc;
}

int main() {
    int i;
    int c;
    int acc;

    probes = 0;
    output(sum_first_primes(1, 100, 5));
    output(sum_first_primes(10, 50, 3));
    output(sum_first_primes(1, 100, 100));

    // 计数：跳过被 3 或 5 整除的数，遇到 60 收手
    c = 0;
    acc = 0;
    i = 0;
    while (i < 200) {
        i = i + 1;
        if (i % 3 == 0 || i % 5 == 0) {
            continue;
        }
        if (i > 60) {
            break;
        }
        c = c + 1;
        acc = acc + i;
    }
    output(c);
    output(acc);
    output(i);

    // 短路保证不会对 0 取模
    int step;
    c = 0;
    step = 0;
    i = 0;
    while (i < 10) {
        i = i + 1;
        if (step != 0 && i % step == 0) {
            c = c + 1;
        }
        if (i == 5) {
            step = 2;
        }
    }
    output(c);

    // 双层：内层 break 只结束内层，外层靠短路条件收敛
    int a;
    int b;
    int hits;
    hits = 0;
    a = 0;
    while (a < 6 && hits < 12) {
        a = a + 1;
        b = 0;
        while (b < 6) {
            b = b + 1;
            if (b == a) {
                continue;
            }
            if (a + b > 8) {
                break;
            }
            hits = hits + 1;
        }
    }
    output(a);
    output(b);
    output(hits);

    // gcd：while 条件用短路，循环体用 continue/break
    int x;
    int y;
    int t;
    x = 1071;
    y = 462;
    while (x != 0 && y != 0) {
        if (x > y) {
            x = x - y;
            continue;
        }
        if (x == y) {
            break;
        }
        y = y - x;
    }
    output(x);
    output(y);
    return 0;
}
