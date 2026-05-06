int global_count = 3; // 测试点：全局变量初始化
float scale = 1.5; // 测试点：全局浮点初始化
const int max_value = 100; // 测试点：全局常量传播
const float default_bias = 0.25; // 测试点：浮点常量参与表达式
const int offsets[3] = {2, 4, 6}; // 测试点：全局常量数组初始化
int numbers[5] = {1, 2, 3, 4, 5}; // 测试点：全局数组初始化

int add(int lhs, int rhs) {
    return lhs + rhs; // 测试点：二元运算与返回
}

int sum_array(int arr[], int n) {
    int i = 0;
    int total = 0;
    while (i < n) { // 测试点：while 循环与条件分支
        total = total + arr[i]; // 测试点：数组索引读取与累加
        i = i + 1; // 测试点：循环变量更新
    }
    return total;
}

void fill(int arr[], int n) {
    int i;
    for (i = 0; i < n; i = i + 1) { // 测试点：for 循环控制流
        if (i % 2 == 0) { // 测试点：取模与条件分支
            arr[i] = getint(); // 测试点：外部函数调用
        } else {
            arr[i] = add(arr[i - 1], 1); // 测试点：用户函数调用与数组读写
        }

        if (arr[i] > max_value) { // 测试点：比较与 break
            break; // 测试点：提前退出循环
        }
    }
    return;
}

int main() {
    int local[3] = {0, 1, 2}; // 测试点：局部数组初始化
    const int local_limit = 3; // 测试点：局部常量初始化
    const int adjust[3] = {1, 0, -1}; // 测试点：局部常量数组初始化
    const int base = offsets[1]; // 测试点：常量数组下标求值
    int i = 0;
    int total = 0;

    fill(local, local_limit); // 测试点：函数调用与数组传参

    local[0] + base; // 测试点：无副作用表达式（用于观察优化）

    for (i = 0; i < local_limit; i = i + 1) { // 测试点：for 循环
        if (local[i] > 10) { // 测试点：条件分支
            continue; // 测试点：continue 跳转
        } else {
            total = total + local[i] + adjust[i] + base; // 测试点：混合表达式降低
        }
    }

    putint(total); // 测试点：整型输出调用
    putfloat(scale + default_bias); // 测试点：浮点表达式与输出调用
    return sum_array(numbers, 3) + global_count; // 测试点：函数返回值参与最终表达式
}