#include <stdio.h>

int getint(void) {
    int value = 0;
    if (scanf("%d", &value) != 1) {
        return 0;
    }
    return value;
}

int getch(void) {
    int value = getchar();
    return value == EOF ? 0 : value;
}

float getfloat(void) {
    float value = 0.0f;
    if (scanf("%f", &value) != 1) {
        return 0.0f;
    }
    return value;
}

int getarray(int a[]) {
    int n = getint();
    for (int i = 0; i < n; ++i) {
        a[i] = getint();
    }
    return n;
}

int getfarray(float a[]) {
    int n = getint();
    for (int i = 0; i < n; ++i) {
        a[i] = getfloat();
    }
    return n;
}

void putint(int x) {
    printf("%d\n", x);
}

void putch(int x) {
    putchar(x);
}

void putfloat(float x) {
    printf("%.6f\n", x);
}

void putarray(int n, int a[]) {
    printf("%d:", n);
    for (int i = 0; i < n; ++i) {
        printf(" %d", a[i]);
    }
    putchar('\n');
}

void putfarray(int n, float a[]) {
    printf("%d:", n);
    for (int i = 0; i < n; ++i) {
        printf(" %.6f", a[i]);
    }
    putchar('\n');
}

void starttime(void) {}

void stoptime(void) {}