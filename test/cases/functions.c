int add(int a, int b) {
    return a + b;
}

int sumEight(int a, int b, int c, int d, int e, int f, int g, int h) {
    return a + b + c + d + e + f + g + h;
}

void printTwice(int x) {
    output(x);
    output(x);
    return;
}

int apply(int x) {
    return add(x, sumEight(1, 2, 3, 4, 5, 6, 7, 8));
}

int main() {
    output(add(20, 22));
    output(sumEight(1, 2, 3, 4, 5, 6, 7, 8));
    printTwice(7);
    output(apply(10));
    return 0;
}
