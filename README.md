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
