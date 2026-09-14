int shadow(int x) {
    int y = x * 2;
    {
        int y = x * 10;
        x = y;
    }
    return x + y;
}

int main() {
    int a = 1;
    {
        int a = 2;
        {
            int a = 3;
            output(a);
        }
        output(a);
    }
    output(a);

    const int LIMIT = 4;
    int i = 0;
    int s = 0;
    while (i < LIMIT) {
        s = s + i;
        i = i + 1;
    }
    output(s);
    output(shadow(3));
    return 0;
}
