#!/bin/bash
# 测试用例运行脚本
#
# 每个用例由 test/cases/ 下的同名文件组成：
#   NAME.c    源程序（SysY）
#   NAME.out  期望的标准输出
#   NAME.in   可选，作为标准输入喂给程序
#
# 流程：Hcompiler 生成汇编 -> riscv64 汇编链接 io.c -> qemu 运行 -> 比对输出

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
CASE_DIR="${SCRIPT_DIR}/cases"
WORK_DIR="${SCRIPT_DIR}/.work"
COMPILER="${ROOT_DIR}/target/debug/Hcompiler"
IO_C="${ROOT_DIR}/io.c"

usage() {
    echo "用法："
    echo "  $0                # 运行全部用例"
    echo "  $0 arith scope    # 只运行指定用例"
    echo
    echo "选项："
    echo "  -h, --help        # 显示帮助"
    exit 0
}

[ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ] && usage

# 检查必需工具
for tool in riscv64-linux-gnu-gcc qemu-riscv64; do
    if ! command -v "$tool" &> /dev/null; then
        echo "未找到 $tool"
        echo "安装命令：sudo apt-get install -y gcc-riscv64-linux-gnu qemu-user"
        exit 1
    fi
done

rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

# 构建编译器，warning 数量较多，输出留到日志里
BUILD_LOG="${WORK_DIR}/cargo-build.log"
echo "构建 Hcompiler ..."
if ! (cd "$ROOT_DIR" && cargo build) > "$BUILD_LOG" 2>&1; then
    echo "cargo build 失败，详见 ${BUILD_LOG#"$ROOT_DIR"/}"
    tail -20 "$BUILD_LOG"
    exit 1
fi

# 预编译运行时库
IO_OBJ="${WORK_DIR}/io.o"
if ! riscv64-linux-gnu-gcc -c -O2 -o "$IO_OBJ" "$IO_C"; then
    echo "编译 io.c 失败"
    exit 1
fi

# 待运行的用例列表
if [ $# -gt 0 ]; then
    CASES=("$@")
else
    CASES=()
    for src in "$CASE_DIR"/*.c; do
        name="$(basename "$src" .c)"
        CASES+=("$name")
    done
fi

PASSED=0
FAILED=0
FAILED_NAMES=()

for name in "${CASES[@]}"; do
    src="${CASE_DIR}/${name}.c"
    expected="${CASE_DIR}/${name}.out"
    stdin_file="${CASE_DIR}/${name}.in"
    asm="${WORK_DIR}/${name}.s"
    exe="${WORK_DIR}/${name}"
    actual="${WORK_DIR}/${name}.actual"
    log="${WORK_DIR}/${name}.log"

    if [ ! -f "$src" ]; then
        echo "✗ ${name}：找不到用例 ${src}"
        FAILED=$((FAILED + 1))
        FAILED_NAMES+=("$name")
        continue
    fi

    if [ ! -f "$expected" ]; then
        echo "✗ ${name}：缺少期望输出 ${name}.out"
        FAILED=$((FAILED + 1))
        FAILED_NAMES+=("$name")
        continue
    fi

    # 1. 生成汇编
    if ! "$COMPILER" "$src" -s "$asm" > "$log" 2>&1; then
        echo "✗ ${name}：生成汇编失败（详见 ${log#"$ROOT_DIR"/}）"
        FAILED=$((FAILED + 1))
        FAILED_NAMES+=("$name")
        continue
    fi

    # 2. 汇编 + 链接 io.c
    if ! riscv64-linux-gnu-gcc -static -o "$exe" "$asm" "$IO_OBJ" >> "$log" 2>&1; then
        echo "✗ ${name}：汇编或链接失败（详见 ${log#"$ROOT_DIR"/}）"
        FAILED=$((FAILED + 1))
        FAILED_NAMES+=("$name")
        continue
    fi

    # 3. 在 qemu 中运行，10 秒超时防止死循环
    if [ -f "$stdin_file" ]; then
        timeout 10 qemu-riscv64 -L /usr/riscv64-linux-gnu "$exe" < "$stdin_file" > "$actual" 2>&1
    else
        timeout 10 qemu-riscv64 -L /usr/riscv64-linux-gnu "$exe" < /dev/null > "$actual" 2>&1
    fi
    run_code=$?

    if [ $run_code -eq 124 ]; then
        echo "✗ ${name}：运行超时（10 秒）"
        FAILED=$((FAILED + 1))
        FAILED_NAMES+=("$name")
        continue
    fi

    # 4. 比对输出
    if diff -u "$expected" "$actual" > "${WORK_DIR}/${name}.diff" 2>&1; then
        echo "✓ ${name}"
        PASSED=$((PASSED + 1))
    else
        echo "✗ ${name}：输出不符（- 期望 / + 实际）"
        sed '1,2d' "${WORK_DIR}/${name}.diff" | sed 's/^/    /'
        FAILED=$((FAILED + 1))
        FAILED_NAMES+=("$name")
    fi
done

echo
echo "========================================="
echo "通过: ${PASSED}  失败: ${FAILED}"
if [ $FAILED -ne 0 ]; then
    echo "失败用例: ${FAILED_NAMES[*]}"
fi
echo "========================================="

[ $FAILED -eq 0 ]
