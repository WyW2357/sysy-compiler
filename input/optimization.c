int helper_twice(int x) {
    return x + x;
}

// 特征1：复杂表达式，包含重复子表达式
int case_complex_expr(int a, int b) {
    int r = (a + b) * (a + b) + (a + b);
    return r;
}

// 特征2：循环中重复表达式 + 无用表达式语句
int case_loop_feature(int n) {
    int i = 0;
    int sum = 0;
    int term = 0;
    while (i < n) {
        term = (i + 2) + (i + 2);
        i + n;
        sum = sum + term;
        i = i + 1;
    }
    return sum;
}

// 特征3：函数调用链
int case_call_chain(int x) {
    int u = helper_twice(x);
    int v = helper_twice(x);
    return case_complex_expr(u, v);
}

// 特征4：显式死代码（结果未使用）
int case_dead_expr(int p, int q) {
    p + q;
    (p + q) * (p + q);
    return p - q;
}

// 特征5：常量分支
int case_const_branch() {
    int v = 0;
    if (1) {
        v = v + 7;
    } else {
        v = v + 100;
    }
    return v;
}

int main() {
    int ok = 0;

    if (case_complex_expr(2, 3) == 30) {
        ok = ok + 1;
    }

    if (case_loop_feature(4) == 28) {
        ok = ok + 1;
    }

    if (case_call_chain(3) == 156) {
        ok = ok + 1;
    }

    if (case_dead_expr(9, 4) == 5) {
        ok = ok + 1;
    }

    if (case_const_branch() == 7) {
        ok = ok + 1;
    }

    return 5 - ok;
}