int main() {
    int i;
    int sum;

    // continue：跳过偶数
    sum = 0;
    i = 0;
    while (i < 10) {
        i = i + 1;
        if (i / 2 * 2 == i) {
            continue;
        }
        sum = sum + i;
    }
    output(sum);

    // break：提前退出
    sum = 0;
    i = 0;
    while (i < 100) {
        i = i + 1;
        if (i > 5) {
            break;
        }
        sum = sum + i;
    }
    output(sum);
    output(i);

    // break/continue 后的语句不可达
    i = 0;
    while (i < 10) {
        i = i + 1;
        break;
        output(999);
    }
    output(i);

    // 嵌套循环：break/continue 只作用于最内层
    int j;
    int total;
    total = 0;
    i = 0;
    while (i < 4) {
        i = i + 1;
        j = 0;
        while (j < 4) {
            j = j + 1;
            if (j == 2) {
                continue;
            }
            if (j == 4) {
                break;
            }
            total = total + 1;
        }
    }
    output(total);

    // 外层 break，内层跑完
    total = 0;
    i = 0;
    while (1) {
        i = i + 1;
        j = 0;
        while (j < 3) {
            j = j + 1;
            total = total + 1;
        }
        if (i == 3) {
            break;
        }
    }
    output(i);
    output(total);

    // 条件里带 && / ||，配合 break/continue
    sum = 0;
    i = 0;
    while (i < 20) {
        i = i + 1;
        if (i > 3 && i < 8) {
            continue;
        }
        if (i == 15 || i == 16) {
            break;
        }
        sum = sum + i;
    }
    output(sum);
    output(i);
    return 0;
}
