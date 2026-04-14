int global_count = 3;
float scale = 1.5;
const int max_value = 100;
const float default_bias = 0.25;
const int offsets[3] = {2, 4, 6};
int numbers[5] = {1, 2, 3, 4, 5};

int add(int lhs, int rhs) {
    return lhs + rhs;
}

int sum_array(int arr[], int n) {
    int i = 0;
    int total = 0;
    while (i < n) {
        total = total + arr[i];
        i = i + 1;
    }
    return total;
}

void fill(int arr[], int n) {
    int i;
    for (i = 0; i < n; i = i + 1) {
        if (i % 2 == 0) {
            arr[i] = getint();
        } else {
            arr[i] = add(arr[i - 1], 1);
        }

        if (arr[i] > max_value) {
            break;
        }
    }
    return;
}

int main() {
    int local[3] = {0, 1, 2};
    const int local_limit = 3;
    const int adjust[3] = {1, 0, -1};
    const int base = offsets[1];
    int i = 0;
    int total = 0;

    fill(local, local_limit);

    local[0] + base;

    for (i = 0; i < local_limit; i = i + 1) {
        if (local[i] > 10) {
            continue;
        } else {
            total = total + local[i] + adjust[i] + base;
        }
    }

    putint(total);
    putfloat(scale + default_bias);
    return sum_array(numbers, 3) + global_count;
}