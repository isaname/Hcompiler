int classify(int x) {
    if (x > 100) {
        return 3;
    } else {
        if (x > 10) {
            return 2;
        } else {
            if (x > 0) {
                return 1;
            }
        }
    }
    return 0;
}

int main() {
    output(classify(500));
    output(classify(50));
    output(classify(5));
    output(classify(-5));
    if (1) {
        output(11);
    }
    if (0) {
        output(22);
    } else {
        output(33);
    }
    return 0;
}
