int main() {
    int n = input();
    int i = 0;
    int sum = 0;
    int maxv = 0;
    while (i < n) {
        int v = input();
        sum = sum + v;
        if (v > maxv) {
            maxv = v;
        }
        i = i + 1;
    }
    output(sum);
    output(maxv);
    output(sum / n);
    return 0;
}
