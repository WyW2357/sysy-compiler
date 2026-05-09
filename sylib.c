#include <stdio.h>
#include <time.h>

static struct timespec g_timer_start;
static int g_timer_started = 0;

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
    printf("%d", x);
}

void putch(int x) {
    putchar(x);
}

void putstr(int s[]) {
    int i = 0;
    while (s[i] != 0) {
        putchar(s[i]);
        i = i + 1;
    }
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

void starttime(void) {
    clock_gettime(CLOCK_MONOTONIC, &g_timer_start);
    g_timer_started = 1;
}

void stoptime(void) {
    struct timespec end;
    long sec;
    long nsec;
    double ms;

    if (!g_timer_started) {
        printf("[timer] starttime not called\n");
        return;
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    sec = end.tv_sec - g_timer_start.tv_sec;
    nsec = end.tv_nsec - g_timer_start.tv_nsec;
    if (nsec < 0) {
        sec = sec - 1;
        nsec = nsec + 1000000000L;
    }

    ms = (double)sec * 1000.0 + (double)nsec / 1000000.0;
    printf("[timer] %.3f ms\n", ms);
    g_timer_started = 0;
}