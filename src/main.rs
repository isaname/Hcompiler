#![feature(linked_list_retain)]
mod ast;
mod ir;
mod macros;
mod irgen;
// mod irgen;
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
use std::io::Result;

lalrpop_mod! {
  #[allow(clippy::all)]
  sysy
}

fn main() -> Result<()> {
  // 解析命令行参数
  let mut args = args();
  args.next();
  let mode = args.next().unwrap();
  let input = args.next().unwrap();
  args.next();
  let output = args.next().unwrap();

  // 读取输入文件
  let input = read_to_string(input)?;

  // 调用 lalrpop 生成的 parser 解析输入文件
  let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
  println!("{:#?}", ast);
  // * 生成ir
  // let ret_val = ast.func_def.block.stmt.num;
  // let ir = irgen::gen_ir(&ast,ret_val).unwrap();
  // if mode=="-koopa" {
  //   return KoopaGenerator::from_path(output).unwrap().generate_on(&ir);
  // }
  Ok(())
}