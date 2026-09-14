// 同时压两个修复：循环的条件和守卫全部用 && / ||（短路 + phi），
// 由它们来驱动 break / continue。任一处坏掉，本用例就会失败。

int calls;

int bump(int ret) {
    calls = calls + 1;
    return ret;
}

int main() {
    int i;
    int sum;
    int j;
    int total;

    // while 条件本身是 &&：两个子条件都在每轮重新求值
    i = 0;
    sum = 0;
    while (i < 100 && sum < 20) {
        i = i + 1;
        sum = sum + i;
    }
    output(i);
    output(sum);

    // while 条件是 ||：任一成立就继续
    i = 0;
    j = 10;
    while (i < 3 || j > 7) {
        i = i + 1;
        j = j - 1;
    }
    output(i);
    output(j);

    // 短路条件里 continue，短路条件里 break
    sum = 0;
    i = 0;
    while (1) {
        i = i + 1;
        if (i > 3 && i < 8) {
            continue;
        }
        if (i >= 12 || sum > 500) {
            break;
        }
        sum = sum + i;
    }
    output(i);
    output(sum);

    // break / continue 的判定依赖 && 的短路：右侧有副作用
    // 若右侧被错误求值，calls 会偏大，且 break 时机会变
    calls = 0;
    i = 0;
    sum = 0;
    while (i < 6) {
        i = i + 1;
        if (i == 2 && bump(1)) {
            continue;
        }
        if (i == 5 && bump(0)) {
            break;
        }
        sum = sum + i;
    }
    output(sum);
    output(calls);

    // 短路是语义必需的：右侧会除以 0
    int d;
    int n;
    n = 12;
    d = 0;
    if (d != 0 && n / d > 100) {
        output(901);
    } else {
        output(1);
    }
    d = 3;
    if (d != 0 && n / d == 4) {
        output(2);
    }

    // || 短路同样保护右侧
    d = 0;
    if (d == 0 || n / d > 100) {
        output(3);
    }

    // 嵌套循环：内外层条件都是短路表达式
    total = 0;
    i = 0;
    while (i < 5 && total < 100) {
        i = i + 1;
        j = 0;
        while (j < 5 || j < i) {
            j = j + 1;
            if (j == 2 || j == 4) {
                continue;
            }
            if (j > 4 && i > 3) {
                break;
            }
            total = total + 1;
        }
    }
    output(i);
    output(total);

    // continue 回到的是 cond 块，条件必须重新求值（含短路副作用）
    calls = 0;
    i = 0;
    while (bump(1) && i < 4) {
        i = i + 1;
        if (i == 2) {
            continue;
        }
    }
    output(i);
    output(calls);

    // ! 与短路混合：! 只能作用于 AddExp，用 (i - 5) 这类算术表达式取非
    i = 0;
    sum = 0;
    while (i < 5) {
        i = i + 1;
        if (!(i - 3) || i > 4) {
            continue;
        }
        sum = sum + i;
    }
    output(i);
    output(sum);

    // ! 的双重否定 + 短路右侧不求值
    calls = 0;
    i = 0;
    while (i < 4) {
        i = i + 1;
        if (!i || bump(1)) {
            sum = sum + 1;
        }
    }
    output(sum);
    output(calls);
    return 0;
}
