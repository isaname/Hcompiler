# Hcompiler 编译器
## 简介
全面支持SysY语法，支持`int input()` `void output(int a)` `void outputFloat(float a)` 三个输入输出函数

## 环境配置
**安装 RISC-V 工具链和 QEMU：**
`sudo apt-get install -y gcc-riscv64-linux-gnu qemu-user`

## 使用方法
1. cargo build 生成可执行文件
2. ./target/debug/Hcompiler [input].c -s [output] 生成汇编代码
3. ./run_rv.sh [output] 链接上io.c生成可执行文件, 并且使用qemu执行

## 测试
`./test/run_tests.sh` 运行 `test/cases/` 下的全部用例，`./test/run_tests.sh arith scope` 只跑指定用例。

每个用例由同名文件组成：`NAME.c` 源程序、`NAME.out` 期望输出、`NAME.in`（可选）标准输入。
新增用例时把这几个文件放进 `test/cases/` 即可，脚本会自动发现。
