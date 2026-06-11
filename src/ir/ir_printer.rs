use std::rc::Rc;

use crate::ir::{
    module::ModulePtr,
    type_::{ArrayType, Type, TypeData},
    user::{ConstantClass, ConstantPtr, GVPtr, InstPtr, OpID},
    value::{BasicBlockPtr, FunctionPtr, ValuePtr},
};

// ===== Type printing =====

pub fn print_type(ty: &Type) -> String {
    match &ty.tdata() {
        TypeData::VoidType => "void".to_string(),
        TypeData::LabelType => "label".to_string(),
        TypeData::IntegerType => "i32".to_string(),
        TypeData::FloatType => "float".to_string(),
        TypeData::BoolType => "i1".to_string(),
        TypeData::FunctionType(f) => {
            let mut s = print_type_rc(&f.result);
            s.push_str(" (");
            for (i, arg) in f.args.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&print_type_rc(arg));
            }
            s.push(')');
            s
        }
        TypeData::PointerType(p) => {
            let mut s = print_type_rc(&p.item);
            s.push('*');
            s
        }
        TypeData::ArrayType(a) => {
            let mut s = String::from("[");
            s.push_str(&a.elem_count.to_string());
            s.push_str(" x ");
            s.push_str(&print_type_rc(&a.item));
            s.push(']');
            s
        }
    }
}

fn print_type_rc(ty: &Rc<Type>) -> String {
    print_type(ty)
}

// ===== OpID to name =====

pub fn op_id_name(id: OpID) -> &'static str {
    match id {
        OpID::Ret => "ret",
        OpID::Br => "br",
        OpID::Add => "add",
        OpID::Sub => "sub",
        OpID::Mul => "mul",
        OpID::SDiv => "sdiv",
        OpID::FAdd => "fadd",
        OpID::FSub => "fsub",
        OpID::FMul => "fmul",
        OpID::FDiv => "fdiv",
        OpID::Alloca => "alloca",
        OpID::Load => "load",
        OpID::Store => "store",
        OpID::Ge => "sge",
        OpID::Gt => "sgt",
        OpID::Le => "sle",
        OpID::Lt => "slt",
        OpID::Eq => "eq",
        OpID::Ne => "ne",
        OpID::FGe => "uge",
        OpID::FGt => "ugt",
        OpID::FLe => "ule",
        OpID::FLt => "ult",
        OpID::FEq => "ueq",
        OpID::FNe => "une",
        OpID::Phi => "phi",
        OpID::Call => "call",
        OpID::GetElementPtr => "getelementptr",
        OpID::ZExt => "zext",
        OpID::FPToSI => "fptosi",
        OpID::SIToFP => "sitofp",
    }
}

// ===== Value as operand =====

/// 打印一个 Value 作为操作数引用
/// print_ty: 是否在前面加类型
pub fn print_as_op(v: &ValuePtr, print_ty: bool) -> String {
    let mut s = String::new();
    if print_ty {
        s.push_str(&print_type_rc(&v.get_type()));
        s.push(' ');
    }
    // 判断是否是全局变量或函数 -> @name
    // 判断是否是常量 -> 直接打印值
    // 否则 -> %name
    if v.to_gv().is_some() || v.to_function().is_some() {
        s.push('@');
        s.push_str(&v.get_name());
    } else if let Some(c) = v.to_const() {
        s.push_str(&print_constant(&c));
    } else {
        s.push('%');
        s.push_str(&v.get_name());
    }
    s
}

// ===== Constant printing =====

pub fn print_constant(c: &ConstantPtr) -> String {
    match &c.0.borrow().ext {
        ConstantClass::Int(val) => {
            let ty = c.0.borrow().user.to_val().get_type();
            if ty.is_bool() {
                if *val == 0 { "false".to_string() } else { "true".to_string() }
            } else {
                val.to_string()
            }
        }
        ConstantClass::Float(val) => {
            // LLVM IR 用 0xHEX 表示 double
            let d = *val as f64;
            let bits = d.to_bits();
            format!("0x{:016X}", bits)
        }
        ConstantClass::Zero => "zeroinitializer".to_string(),
        ConstantClass::Arr(elems) => {
            let ty = c.0.borrow().user.to_val().get_type();
            let mut s = print_type_rc(&ty);
            s.push_str(" [");
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                let elem_ty = elem.0.borrow().user.to_val().get_type();
                s.push_str(&print_type_rc(&elem_ty));
                s.push(' ');
                s.push_str(&print_constant(elem));
            }
            s.push(']');
            s
        }
    }
}

// ===== Instruction printing =====

pub fn print_inst(inst: &InstPtr) -> String {
    let op = inst.get_inst_op_id();
    let operands = inst.to_user().get_operands();
    let ops = operands.borrow();
    let name = inst.to_val().get_name();

    match op {
        OpID::Add | OpID::Sub | OpID::Mul | OpID::SDiv |
        OpID::FAdd | OpID::FSub | OpID::FMul | OpID::FDiv => {
            // %name = add type op0, op1
            let mut s = format!("%{} = {} ", name, op_id_name(op));
            s.push_str(&print_type_rc(&ops[0].get_type()));
            s.push(' ');
            s.push_str(&print_as_op(&ops[0], false));
            s.push_str(", ");
            s.push_str(&print_as_op(&ops[1], false));
            s
        }
        OpID::Ge | OpID::Gt | OpID::Le | OpID::Lt | OpID::Eq | OpID::Ne => {
            let mut s = format!("%{} = icmp {} ", name, op_id_name(op));
            s.push_str(&print_type_rc(&ops[0].get_type()));
            s.push(' ');
            s.push_str(&print_as_op(&ops[0], false));
            s.push_str(", ");
            s.push_str(&print_as_op(&ops[1], false));
            s
        }
        OpID::FGe | OpID::FGt | OpID::FLe | OpID::FLt | OpID::FEq | OpID::FNe => {
            let mut s = format!("%{} = fcmp {} ", name, op_id_name(op));
            s.push_str(&print_type_rc(&ops[0].get_type()));
            s.push(' ');
            s.push_str(&print_as_op(&ops[0], false));
            s.push_str(", ");
            s.push_str(&print_as_op(&ops[1], false));
            s
        }
        OpID::Call => {
            let ret_ty = inst.to_val().get_type();
            let mut s = String::new();
            if !ret_ty.is_void() {
                s.push_str(&format!("%{} = ", name));
            }
            s.push_str("call ");
            s.push_str(&print_type_rc(&ret_ty));
            s.push(' ');
            s.push_str(&print_as_op(&ops[0], false));
            s.push('(');
            for i in 1..ops.len() {
                if i > 1 {
                    s.push_str(", ");
                }
                s.push_str(&print_as_op(&ops[i], true));
            }
            s.push(')');
            s
        }
        OpID::Br => {
            if ops.len() == 3 {
                // cond br
                let mut s = String::from("br ");
                s.push_str(&print_as_op(&ops[0], true));
                s.push_str(", ");
                s.push_str(&print_as_op(&ops[1], true));
                s.push_str(", ");
                s.push_str(&print_as_op(&ops[2], true));
                s
            } else {
                // uncond br
                let mut s = String::from("br ");
                s.push_str(&print_as_op(&ops[0], true));
                s
            }
        }
        OpID::Ret => {
            if ops.is_empty() {
                "ret void".to_string()
            } else {
                let mut s = String::from("ret ");
                s.push_str(&print_as_op(&ops[0], true));
                s
            }
        }
        OpID::GetElementPtr => {
            let mut s = format!("%{} = getelementptr ", name);
            // print the base element type (ptr's pointee)
            let ptr_ty = ops[0].get_type();
            s.push_str(&print_type_rc(&ptr_ty.get_ptr_elem_ty().unwrap()));
            s.push_str(", ");
            for (i, op) in ops.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&print_as_op(op, true));
            }
            s
        }
        OpID::Store => {
            let mut s = String::from("store ");
            s.push_str(&print_as_op(&ops[0], true));
            s.push_str(", ");
            s.push_str(&print_as_op(&ops[1], true));
            s
        }
        OpID::Load => {
            let mut s = format!("%{} = load ", name);
            let ptr_ty = ops[0].get_type();
            s.push_str(&print_type_rc(&ptr_ty.get_ptr_elem_ty().unwrap()));
            s.push_str(", ");
            s.push_str(&print_as_op(&ops[0], true));
            s
        }
        OpID::Alloca => {
            let mut s = format!("%{} = alloca ", name);
            // alloca 的结果类型是 ptr(T)，需要打印 T
            let result_ty = inst.to_val().get_type();
            s.push_str(&print_type_rc(&result_ty.get_ptr_elem_ty().unwrap()));
            s
        }
        OpID::ZExt => {
            let mut s = format!("%{} = zext ", name);
            s.push_str(&print_as_op(&ops[0], true));
            s.push_str(" to ");
            s.push_str(&print_type_rc(&inst.to_val().get_type()));
            s
        }
        OpID::FPToSI => {
            let mut s = format!("%{} = fptosi ", name);
            s.push_str(&print_as_op(&ops[0], true));
            s.push_str(" to ");
            s.push_str(&print_type_rc(&inst.to_val().get_type()));
            s
        }
        OpID::SIToFP => {
            let mut s = format!("%{} = sitofp ", name);
            s.push_str(&print_as_op(&ops[0], true));
            s.push_str(" to ");
            s.push_str(&print_type_rc(&inst.to_val().get_type()));
            s
        }
        OpID::Phi => {
            let mut s = format!("%{} = phi ", name);
            s.push_str(&print_type_rc(&ops[0].get_type()));
            s.push(' ');
            let pairs = ops.len() / 2;
            for i in 0..pairs {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str("[ ");
                s.push_str(&print_as_op(&ops[2 * i], false));
                s.push_str(", ");
                s.push_str(&print_as_op(&ops[2 * i + 1], false));
                s.push_str(" ]");
            }
            s
        }
    }
}

// ===== BasicBlock printing =====

pub fn print_bb(bb: &BasicBlockPtr) -> String {
    let mut s = String::new();
    s.push_str(&bb.to_val().get_name());
    s.push(':');
    // print predecessors
    let pre_bbs = &bb.0.borrow().pre_bbs;
    if !pre_bbs.is_empty() {
        s.push_str("                                                ; preds = ");
        for (i, pre) in pre_bbs.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let pre_bb = BasicBlockPtr(pre.upgrade().unwrap());
            s.push('%');
            s.push_str(&pre_bb.to_val().get_name());
        }
    }
    s.push('\n');
    for inst in &bb.0.borrow().insts {
        s.push_str("  ");
        s.push_str(&print_inst(inst));
        s.push('\n');
    }
    s
}

// ===== Function printing =====

pub fn print_function(func: &FunctionPtr) -> String {
    let mut s = String::new();
    let is_decl = func.0.borrow().bbs.is_empty();
    if !is_decl {
        func.clone().set_inst_name();
    }
    if is_decl {
        s.push_str("declare ");
    } else {
        s.push_str("define ");
    }
    s.push_str(&print_type_rc(&func.get_return_type()));
    s.push(' ');
    s.push('@');
    s.push_str(&func.to_val().get_name());
    s.push('(');
    if is_decl {
        let func_ty = func.get_type();
        let n = func_ty.get_func_arg_num().unwrap();
        for i in 0..n {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&print_type_rc(&func_ty.get_func_param_ty(i).unwrap()));
        }
    } else {
        let args = &func.0.borrow().args;
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&print_type_rc(&arg.to_val().get_type()));
            s.push_str(" %");
            s.push_str(&arg.to_val().get_name());
        }
    }
    s.push(')');
    if is_decl {
        s.push('\n');
    } else {
        s.push_str(" {\n");
        for bb in &func.0.borrow().bbs {
            s.push_str(&print_bb(bb));
        }
        s.push_str("}\n");
    }
    s
}

// ===== GlobalVariable printing =====

pub fn print_gv(gv: &GVPtr) -> String {
    let mut s = String::from("@");
    s.push_str(&gv.0.borrow().user.to_val().get_name());
    s.push_str(" = ");
    if gv.is_const() {
        s.push_str("constant ");
    } else {
        s.push_str("global ");
    }
    // GV 的类型是 ptr(T), 打印 T
    let gv_ty = gv.0.borrow().user.to_val().get_type();
    let elem_ty = gv_ty.get_ptr_elem_ty().unwrap_or(gv_ty.clone());
    s.push_str(&print_type_rc(&elem_ty));
    s.push(' ');
    match &gv.0.borrow().init_val {
        Some(init) => s.push_str(&print_constant(init)),
        None => s.push_str("zeroinitializer"),
    }
    s
}

// ===== Module printing =====

pub fn print_module(m: &ModulePtr) -> String {
    let mut s = String::new();
    for gv in &m.0.borrow().gv_list {
        s.push_str(&print_gv(gv));
        s.push('\n');
    }
    if !m.0.borrow().gv_list.is_empty() {
        s.push('\n');
    }
    for func in &m.0.borrow().func_list {
        s.push_str(&print_function(func));
        s.push('\n');
    }
    s
}
