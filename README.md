# Hcompiler 编译器

## 使用方法
1. cargo build 生成可执行文件
2. ./target/debug/Hcompiler <input>.c -o <output>.ll 生成llvm形式的中间代码
3. clang -O0 <output>.ll io.c -o <output>  链接上io.c生成可执行文件

## 简介
全面支持SysY语法，支持`int input()` `void output(int a)` `void outputFloat(float a)` 三个输入输出函数