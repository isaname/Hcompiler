int main() {
    int i = 1;
    int fact = 1;
    while (i <= 6) {
        fact = fact * i;
        i = i + 1;
    }
    output(fact);

    int row = 1;
    int total = 0;
    while (row <= 4) {
        int col = 1;
        while (col <= row) {
            total = total + row * col;
            col = col + 1;
        }
        row = row + 1;
    }
    output(total);

    int n = 27;
    int steps = 0;
    while (n != 1) {
        if (n % 2 == 0) {
            n = n / 2;
        }
        if (n % 2 == 1) {
            if (n != 1) {
                n = 3 * n + 1;
            }
        }
        steps = steps + 1;
    }
    output(steps);
    return 0;
}
