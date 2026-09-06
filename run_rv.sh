#!/bin/bash
# RISC-V 64 汇编代码运行脚本

set -e
# 检查必需工具
check_tools() {
    local missing=0

    if ! command -v riscv64-linux-gnu-gcc &> /dev/null; then
        echo "未找到 riscv64-linux-gnu-gcc"
        missing=1
    else
        echo "✓ RISC-V GCC: $(riscv64-linux-gnu-gcc --version | head -1)"
    fi

    if ! command -v qemu-riscv64 &> /dev/null; then
        echo "未找到 qemu-riscv64"
        missing=1
    else
        echo "✓ QEMU RISC-V: $(qemu-riscv64 --version | head -1)"
    fi

    if [ $missing -eq 1 ]; then
        echo
        echo "安装命令："
        echo "  sudo apt-get update"
        echo "  sudo apt-get install gcc-riscv64-linux-gnu qemu-user"
        exit 1
    fi
    echo
}

# 使用说明
usage() {
    echo "用法："
    echo "  $0 <汇编文件.s>           # 编译并运行"
    echo "  $0 hello_rv64.s           # 示例"
    echo
    echo "选项："
    echo "  -h, --help               # 显示帮助"
    exit 0
}

# 解析参数
KEEP_FILES=0
VERBOSE=0
ASM_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            ;;
        *)
            ASM_FILE="$1"
            shift
            ;;
    esac
done

# 检查输入文件
if [ -z "$ASM_FILE" ]; then
    echo "错误：请提供汇编文件"
    echo
    usage
fi

if [ ! -f "$ASM_FILE" ]; then
    echo "错误：文件不存在: $ASM_FILE"
    exit 1
fi

# 检查工具
check_tools

# 准备文件名
BASE_NAME="${ASM_FILE%.s}"
OBJ_FILE="${BASE_NAME}.o"
EXE_FILE="${BASE_NAME}"


# 编译汇编到目标文件
riscv64-linux-gnu-as -march=rv64imfd -o "$OBJ_FILE" "$ASM_FILE" 2>&1 | grep -i "error" || true

if [ ! -f "$OBJ_FILE" ]; then
    echo "汇编失败"
    exit 1
fi

# 链接生成可执行文件（链接 io.c 运行时库）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IO_C="${SCRIPT_DIR}/io.c"
IO_OBJ="${SCRIPT_DIR}/io_rv64.o"

if [ ! -f "$IO_C" ]; then
    exit 1
fi

riscv64-linux-gnu-gcc -c -O2 -o "$IO_OBJ" "$IO_C"

riscv64-linux-gnu-gcc -static -o "$EXE_FILE" "$OBJ_FILE" "$IO_OBJ" 2>&1 | grep -i "error" || true

if [ ! -f "$EXE_FILE" ]; then
    echo "链接失败"
    exit 1
fi

# 运行程序
qemu-riscv64 -L /usr/riscv64-linux-gnu "$EXE_FILE"
EXIT_CODE=$?


echo
echo "========================================="
echo "程序退出码: $EXIT_CODE"
echo "========================================="

# 清理中间文件
rm -f "$OBJ_FILE" "$IO_OBJ"
