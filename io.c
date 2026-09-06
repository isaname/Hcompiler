#include <stdio.h>
#include <stdlib.h>
int input() {
    int a;
    (void)!scanf("%d", &a);
    return a;
}

void output(int a) { printf("%d\n", a); }

void outputFloat(float a) { printf("%f\n", a); }

