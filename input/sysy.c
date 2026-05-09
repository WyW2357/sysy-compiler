int g_scalar = 7;
const int g_const = 11;
int g_arr[5] = {1, 3, 5, 7, 9};
float g_f = 1.25;
const float g_fc = 0.75;
/* 下面的字符串常量以整数数组的形式定义 */
int msg_begin[13] = {66, 69, 71, 73, 78, 95, 84, 69, 83, 84, 83, 10, 0}; // "BEGIN_TESTS\n"
int msg_test[6] = {84, 69, 83, 84, 95, 0}; // "TEST_"
int msg_pass[8] = {58, 80, 65, 83, 83, 33, 10, 0}; // ":PASS!\n"
int msg_fail[8] = {58, 70, 65, 73, 76, 33, 10, 0}; // ":FAIL!\n"

int add3(int a, int b, int c) {
    return a + b + c;
}

int passthrough_int(int x) {
    return x;
}

float passthrough_float(float x) {
    return x;
}

int clamp_nonneg(int x) {
    if (x > 0) {
        return x;
    }
    else {
        return 0;
    }
}

void fill_linear(int arr[], int n, int start) {
    int i = 0;
    for (i = 0; i < n; i = i + 1) {
        arr[i] = start + i * 2;
    }
}

void print_case_result(int case_id, int value) {
    putstr(msg_test);
    putint(case_id);
    if (value) {
        putstr(msg_pass);
    } else {
        putstr(msg_fail);
    }
}

// 测试点1：整数算术、运算符优先级、赋值表达式
int test_arith_and_assign() {
    int flag = 0;
    int x = 2 + 3 * 4;
    if (x == 14) {
        flag = flag + 1;
    }
    x = (x - 5) / 3;
    if (x == 3) {
        flag = flag + 1;
    }
    x = x % 2;
    if (x == 1) {
        flag = flag + 1;
    }
    return flag == 3;
}

// 测试点2：一元正负号、逻辑非、空语句
int test_unary_and_empty_stmt() {
    int a = 3;
    int b = -a;
    int c = +b;
    int ok = 0;
    ;

    if (!ok) {
        ok = ok + 1;
    }

    return a == 3 && b == -3 && c == -3 && ok == 1;
}

// 测试点3：if-else、逻辑与或非、比较运算
int test_if_else_logic() {
    int flag = 0;
    int a = -3;
    int b = clamp_nonneg(a);

    if (a < 0 && b == 0) {
        flag = flag + 1;
    }

    if (a > 0 || b == 0) {
        flag = flag + 1;
    }

    if (!(a == 0)) {
        flag = flag + 1;
    }

    return flag == 3;
}

// 测试点4：嵌套 if-else 分支结构
int test_nested_branch() {
    int a = 2;
    int b = 5;
    int result = 0;

    if (a < b) {
        if (a + b == 7) {
            result = 1;
        } else {
            result = 100;
        }
    } else {
        result = 100;
    }

    return result == 1;
}

// 测试点5：while 循环、break、continue
int test_while_break_continue() {
    int i = 0;
    int sum = 0;

    while (i < 8) {
        i = i + 1;
        if (i % 2 == 0) {
            continue;
        }
        sum = sum + i;
        if (sum > 10) {
            break;
        }
    }

    return sum == 16;
}

// 测试点6：for 零次迭代边界
int test_for_zero_iteration() {
    int i = 0;
    int sum = 0;

    for (i = 0; i < 0; i = i + 1) {
        sum = sum + 1;
    }

    return sum == 0;
}

// 测试点7：for 循环、break、continue
int test_for_break_continue() {
    int i = 0;
    int acc = 0;

    for (i = 0; i < 10; i = i + 1) {
        if (i == 2) {
            continue;
        }
        if (i == 7) {
            break;
        }
        acc = acc + i;
    }

    return acc == 19;
}

// 测试点8：数组、数组传参、数组读写
int test_array() {
    int local[4] = {0, 0, 0, 0};
    int i = 0;
    int sum = 0;

    fill_linear(local, 4, 3);
    while (i < 4) {
        sum = sum + local[i];
        i = i + 1;
    }

    return sum == 24 && local[2] == 7;
}

// 测试点9：前置/后置自增自减
int test_prefix_postfix() {
    int x = 5;
    int y = 0;

    y = x++;
    y = y + ++x;
    y = y + x--;
    y = y + --x;

    return x == 5 && y == 24;
}

// 测试点10：作用域遮蔽、函数调用、全局变量读取
int test_scope_and_call() {
    int x = 10;
    {
        int x = 3;
        x = x + 2;
        if (x != 5) {
            return 0;
        }
    }

    return add3(x, g_scalar, g_const) == 28;
}

// 测试点11：浮点表达式与比较
int test_float_expr() {
    float v = g_f + g_fc;
    if (v > 1.9 && v < 2.1) {
        return 1;
    }
    return 0;
}

// 测试点12：浮点加减乘除综合运算与区间比较
int test_float_arith_extended() {
    float a = 1.5;
    float b = 2.0;
    float c = a * b + b / a - 1.0;

    if (c > 3.3 && c < 3.4) {
        return 1;
    }
    return 0;
}

// 测试点13：强制类型转换（通过赋值与函数参数触发 int<->float 转换）
int test_forced_conversion() {
    int i = 5;
    float f = 2.6;
    float to_float = i;
    int to_int = f;
    int mixed_assign = i + f;
    int via_call_int = passthrough_int(f);
    float via_call_float = passthrough_float(i);
    int ok = 0;

    if (to_float > 4.9 && to_float < 5.1) {
        ok = ok + 1;
    }
    if (to_int == 2) {
        ok = ok + 1;
    }
    if (mixed_assign == 7) {
        ok = ok + 1;
    }
    if (via_call_int == 2) {
        ok = ok + 1;
    }
    if (via_call_float > 4.9 && via_call_float < 5.1) {
        ok = ok + 1;
    }

    return ok == 5;
}

// 测试点14：按行优先存储的多维数组访问
int test_multidim_row_major() {
    int m[2][3];
    int i = 0;
    int j = 0;
    int k = 1;

    for (i = 0; i < 2; i = i + 1) {
        for (j = 0; j < 3; j = j + 1) {
            m[i][j] = k;
            k = k + 1;
        }
    }

    return m[0][0] == 1
        && m[0][1] == 2
        && m[0][2] == 3
        && m[1][0] == 4
        && m[1][1] == 5
        && m[1][2] == 6;
}

// 测试点15：常量、全局常量、数组下标
int test_const_and_global_index() {
    const int idx = 3;
    int v = g_arr[idx - 1];
    return v == 5 && g_const == 11;
}

// 测试点16：综合流程（全局变量、常量、数组、函数调用、for/while、if-else）
int integrated_test() {
    int buf[5] = {0, 0, 0, 0, 0};
    int i = 0;
    int k = 0;
    int total = 0;

    fill_linear(buf, 5, 1);

    for (i = 0; i < 5; i = i + 1) {
        total = total + buf[i] * g_arr[4 - i];
    }

    if (total == 85) {
        total = total + add3(1, 2, 3);
    } else {
        total = total - 100;
    }

    while (k < 3) {
        total = total + g_arr[k + 1];
        k = k + 1;
    }

    return total == 106;
}

int main() {
    int failed = 16;
    int result = 0;

    putstr(msg_begin);

    result = test_arith_and_assign();
    print_case_result(1, result);
    failed = failed - result;

    result = test_unary_and_empty_stmt();
    print_case_result(2, result);
    failed = failed - result;

    result = test_if_else_logic();
    print_case_result(3, result);
    failed = failed - result;

    result = test_nested_branch();
    print_case_result(4, result);
    failed = failed - result;

    result = test_while_break_continue();
    print_case_result(5, result);
    failed = failed - result;

    result = test_for_zero_iteration();
    print_case_result(6, result);
    failed = failed - result;

    result = test_for_break_continue();
    print_case_result(7, result);
    failed = failed - result;

    result = test_array();
    print_case_result(8, result);
    failed = failed - result;

    result = test_prefix_postfix();
    print_case_result(9, result);
    failed = failed - result;

    result = test_scope_and_call();
    print_case_result(10, result);
    failed = failed - result;

    result = test_float_expr();
    print_case_result(11, result);
    failed = failed - result;

    result = test_float_arith_extended();
    print_case_result(12, result);
    failed = failed - result;

    result = test_forced_conversion();
    print_case_result(13, result);
    failed = failed - result;

    result = test_multidim_row_major();
    print_case_result(14, result);
    failed = failed - result;

    result = test_const_and_global_index();
    print_case_result(15, result);
    failed = failed - result;

    result = integrated_test();
    print_case_result(16, result);
    failed = failed - result;

    return failed;
}