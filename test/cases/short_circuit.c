// SysY 里 && / || 只出现在 if / while 的条件中（Cond -> LOrExp），
// 所以这里统一通过条件语句来观察短路求值的结果和副作用。

int calls;

int bump(int ret) {
    calls = calls + 1;
    return ret;
}

int and2(int a, int b) {
    if (a && b) {
        return 1;
    }
    return 0;
}

int or2(int a, int b) {
    if (a || b) {
        return 1;
    }
    return 0;
}

int main() {
    // 真值表
    output(and2(1, 1));
    output(and2(1, 0));
    output(and2(0, 1));
    output(and2(0, 0));
    output(or2(1, 1));
    output(or2(1, 0));
    output(or2(0, 1));
    output(or2(0, 0));

    // 非零即真，结果规范化为 0 / 1
    output(and2(3, 4));
    output(and2(3, 0));
    output(or2(0, 7));
    output(and2(0 - 2, 5));

    // 短路：&& 左侧为假时右侧不求值
    calls = 0;
    if (0 && bump(1)) {
        output(901);
    }
    output(calls);

    // 短路：|| 左侧为真时右侧不求值
    calls = 0;
    if (1 || bump(1)) {
        output(1);
    }
    output(calls);

    // 不短路时右侧必须求值
    calls = 0;
    if (1 && bump(1)) {
        output(1);
    }
    output(calls);

    calls = 0;
    if (0 || bump(0)) {
        output(902);
    }
    output(calls);

    // 优先级：&& 紧于 ||
    if (0 && 1 || 1) {
        output(11);
    }
    if (1 || 0 && 0) {
        output(22);
    }

    // 多级链接与短路的传递
    calls = 0;
    if (0 && bump(1) && bump(1)) {
        output(903);
    }
    output(calls);

    calls = 0;
    if (1 || bump(1) || bump(1)) {
        output(33);
    }
    output(calls);

    calls = 0;
    if (bump(1) && bump(0) && bump(1)) {
        output(904);
    }
    output(calls);

    // 与关系 / 相等运算混合
    int i;
    i = 5;
    if (i > 0 && i < 10) {
        output(44);
    }
    if (i < 0 || i > 100) {
        output(905);
    } else {
        output(55);
    }
    if (i == 5 || i == 6) {
        output(66);
    }
    if (i != 5 && i > 0) {
        output(906);
    }

    // 在 while 条件里短路，右侧不该被求值
    calls = 0;
    while (0 && bump(1)) {
        output(907);
    }
    output(calls);

    // while 条件短路控制循环次数
    i = 0;
    while (i < 5 && i != 3) {
        i = i + 1;
    }
    output(i);
    return 0;
}
