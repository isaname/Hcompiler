#![feature(linked_list_retain)]
mod ast;
mod ir;
mod macros;
mod irgen;
mod asmgen;
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{read_to_string, write};
use std::io::Result;

use crate::asmgen::ag::AsmGen;
use crate::irgen::visit::IRGenerator;

lalrpop_mod! {
  #[allow(clippy::all)]
  sysy
}

fn main() -> Result<()> {
  // 解析命令行参数
  let mut args = args();
  args.next();
  let input = args.next().unwrap();
  let mode = args.next().unwrap();
  let output = args.next().unwrap();

  // 读取输入文件
  let input = read_to_string(input)?;

  // 调用 lalrpop 生成的 parser 解析输入文件
  let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
  // println!("{:#?}", ast);
  // * 生成ir
  let mut irgenerator = IRGenerator::new();
  irgenerator.visit(ast);

  // 根据 mode 选择输出：-l 生成 LLVM IR，-s 生成 RV64 汇编
  match mode.as_str() {
    "-l" => {
      let ir = irgenerator.dump_to_string();
      write(output, ir)?;
    }
    "-s" => {
      let mut asmgen = AsmGen::new(irgenerator.get_module());
      asmgen.run();
      let asm = asmgen.print();
      write(output, asm)?;
    }
    _ => {
      eprintln!("未知的 mode: {}（支持 -l 生成 LLVM IR，-s 生成汇编）", mode);
      std::process::exit(1);
    }
  }
  Ok(())
}