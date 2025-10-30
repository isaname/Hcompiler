mod scopes;
use koopa::ir::builder::ValueBuilder;
use koopa::{front::ast::Error, ir::{FunctionData, Program, Type, builder::{BasicBlockBuilder,LocalInstBuilder}, values::ZeroInit}};

use crate::ast::*;

pub type Result<T> = std::result::Result<T,Error>;

pub fn gen_ir(comp_unit: &CompUnit, num:i32) -> Result<Program> {
    let mut ir = Program::new();
    // let mut scope = Scope::new();
    let func = &comp_unit.func_def;
    let mut data = FunctionData::new(format!("@{}",&func.ident),Vec::new(),Type::get_i32());
    let entry = data.dfg_mut().new_bb().basic_block(Some("%entry".into()));

    let fff = ir.new_func(data);
    let func_data = ir.func_mut(fff);
    {
        func_data.layout_mut().bbs_mut().extend([entry]);
        let a = func_data.dfg_mut().new_value().integer(num);
        let ret_val = func_data.dfg_mut().new_value().ret(Some(a));


        func_data.layout_mut().bb_mut(entry).insts_mut().push_key_back(ret_val).unwrap();
    }
    Ok(ir)
}