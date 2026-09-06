use super::ag::{AsmGen, AsmInst, Context};
use super::codegen_util::*;
use super::register::{FReg, Reg};
use crate::ir::user::{ConstantClass, ConstantPtr, OpID};
use crate::ir::value::{BasicBlockPtr, FunctionPtr, ValuePtr};
use crate::ir::module::ModulePtr;
use std::rc::Rc;

impl AsmGen {
    pub fn new(m: ModulePtr) -> Self {
        AsmGen {
            m,
            context: Context::new(),
            output: Vec::new(),
        }
    }

    pub fn append_inst(&mut self, content: String) {
        self.output.push(AsmInst::Instruction(content));
    }

    pub fn append_inst_with_type(&mut self, content: String, ty: AsmInst) {
        self.output.push(ty);
    }

    pub fn append_inst_parts(&mut self, inst: &str, args: Vec<String>) {
        let mut content = inst.to_string() + " ";
        for arg in args {
            content += &arg;
            content += ", ";
        }
        if content.ends_with(", ") {
            content.pop();
            content.pop();
        }
        self.output.push(AsmInst::Instruction(content));
    }

    // 获取类型大小的辅助函数
    fn get_type_size(ty: &crate::ir::type_::Type) -> usize {
        if ty.is_bool() {
            1
        } else if ty.is_int() {
            4
        } else if ty.is_float() {
            4
        } else if ty.is_ptr() {
            8
        } else if ty.is_arr() {
            let elem_ty = ty.get_arr_elem_ty().unwrap();
            let elem_size = Self::get_type_size(&elem_ty);
            // 获取数组元素数量
            if let crate::ir::type_::TypeData::ArrayType(arr) = ty.tdata() {
                elem_size * arr.elem_count
            } else {
                panic!("Expected array type");
            }
        } else {
            panic!("Cannot get size of this type");
        }
    }

    pub fn allocate(&mut self) {
        let func = self.context.func.upgrade().unwrap();
        let func_ptr = FunctionPtr(func);

        let mut offset = PROLOGUE_OFFSET_BASE;

        // 为每个参数分配栈空间
        for arg in &func_ptr.0.borrow().args {
            let size = Self::get_type_size(&arg.to_val().get_type());
            offset = align(offset + size, size);
            let val_id = arg.to_val().get_id();
            self.context.offset_map.insert(val_id, -(offset as i32));
        }

        // 为指令结果分配栈空间
        for bb in &func_ptr.0.borrow().bbs {
            for inst in &bb.0.borrow().insts {
                let inst_type = inst.to_user().to_val().get_type();
                // 为非 void 结果分配栈空间
                if !inst_type.is_void() {
                    let size = Self::get_type_size(&inst_type);
                    offset = align(offset + size, size);
                    let val_id = inst.to_user().to_val().get_id();
                    self.context.offset_map.insert(val_id, -(offset as i32));
                }
                // Alloca 副作用：分配额外空间
                if inst.get_inst_op_id() as u8 == OpID::Alloca as u8 {
                    let alloca_ty = inst_type.get_ptr_elem_ty().unwrap();
                    let alloc_size = Self::get_type_size(&alloca_ty);
                    offset += alloc_size;
                }
            }
        }

        // 分配栈空间，需要是 16 的倍数
        self.context.frame_size = align(offset, PROLOGUE_ALIGN);
    }

    pub fn load_to_greg(&mut self, val: &ValuePtr, reg: &Reg) {
        let val_type = val.get_type();
        assert!(val_type.is_int() || val_type.is_bool() || val_type.is_ptr());

        // 检查是否是常量
        if let Some(const_val) = val.to_const() {
            match &const_val.0.borrow().ext {
                ConstantClass::Int(v) => {
                    let int_val = *v;
                    if is_imm_12(int_val) {
                        self.append_inst_parts(
                            ADDI,
                            vec![reg.print(), "zero".to_string(), int_val.to_string()],
                        );
                    } else {
                        self.load_large_int32(int_val, reg);
                    }
                }
                _ => {}
            }
        } else if let Some(gv) = val.to_gv() {
            // 全局变量
            self.append_inst_parts(LA, vec![reg.print(), gv.to_val().get_name()]);
        } else {
            // 从栈加载
            self.load_from_stack_to_greg(val, reg);
        }
    }

    pub fn load_large_int32(&mut self, val: i32, reg: &Reg) {
        // 对于 RV64，使用 lui + addi 加载 32 位立即数
        let high_20 = (val >> 12) & 0xFFFFF;
        let low_12 = val & 0xFFF;

        // 如果低 12 位是负数（第 11 位为 1）需要调整
        let (high_20, low_12) = if low_12 & 0x800 != 0 {
            // 如果低位部分是负数，高位部分加 1
            (high_20 + 1, low_12 | !0xFFF)
        } else {
            (high_20, low_12)
        };

        self.append_inst_parts(LUI, vec![reg.print(), format!("{}", high_20)]);
        if low_12 != 0 {
            self.append_inst_parts(ADDI, vec![reg.print(), reg.print(), format!("{}", low_12 as i32)]);
        }
    }

    pub fn load_large_int64(&mut self, val: i64, reg: &Reg) {
        // 对于 RV64，加载 64 位值
        // 分解为多个 lui + addi 操作
        let low_32 = val as i32;
        self.load_large_int32(low_32, reg);

        if val > i32::MAX as i64 || val < i32::MIN as i64 {
            // 需要加载高 32 位
            let high_32 = (val >> 32) as i32;
            if high_32 != 0 && high_32 != -1 {
                let temp = Reg::t(5);
                self.load_large_int32(high_32, &temp);
                // 左移 32 位并相加
                self.append_inst_parts("slli", vec![temp.print(), temp.print(), "32".to_string()]);
                self.append_inst_parts(ADD, vec![reg.print(), reg.print(), temp.print()]);
            }
        }
    }

    pub fn load_from_stack_to_greg(&mut self, val: &ValuePtr, reg: &Reg) {
        let offset = *self.context.offset_map.get(&val.get_id()).unwrap();
        let offset_str = offset.to_string();
        let val_type = val.get_type();

        if is_imm_12(offset) {
            if val_type.is_bool() {
                self.append_inst_parts(LB, vec![reg.print(), format!("{}(s0)", offset_str)]);
            } else if val_type.is_int() {
                self.append_inst_parts(LW, vec![reg.print(), format!("{}(s0)", offset_str)]);
            } else {
                // 指针
                self.append_inst_parts(LD, vec![reg.print(), format!("{}(s0)", offset_str)]);
            }
        } else {
            self.load_large_int64(offset as i64, reg);
            self.append_inst_parts(ADD, vec![reg.print(), "s0".to_string(), reg.print()]);
            if val_type.is_bool() {
                self.append_inst_parts(LB, vec![reg.print(), format!("0({})", reg.print())]);
            } else if val_type.is_int() {
                self.append_inst_parts(LW, vec![reg.print(), format!("0({})", reg.print())]);
            } else {
                // 指针
                self.append_inst_parts(LD, vec![reg.print(), format!("0({})", reg.print())]);
            }
        }
    }

    pub fn store_from_greg(&mut self, val: &ValuePtr, reg: &Reg) {
        let offset = *self.context.offset_map.get(&val.get_id()).unwrap();
        let offset_str = offset.to_string();
        let val_type = val.get_type();

        if is_imm_12(offset) {
            if val_type.is_bool() {
                self.append_inst_parts(SB, vec![reg.print(), format!("{}(s0)", offset_str)]);
            } else if val_type.is_int() {
                self.append_inst_parts(SW, vec![reg.print(), format!("{}(s0)", offset_str)]);
            } else {
                // 指针
                self.append_inst_parts(SD, vec![reg.print(), format!("{}(s0)", offset_str)]);
            }
        } else {
            let addr = Reg::t(6);
            self.load_large_int64(offset as i64, &addr);
            self.append_inst_parts(ADD, vec![addr.print(), "s0".to_string(), addr.print()]);
            if val_type.is_bool() {
                self.append_inst_parts(SB, vec![reg.print(), format!("0({})", addr.print())]);
            } else if val_type.is_int() {
                self.append_inst_parts(SW, vec![reg.print(), format!("0({})", addr.print())]);
            } else {
                // 指针
                self.append_inst_parts(SD, vec![reg.print(), format!("0({})", addr.print())]);
            }
        }
    }

    pub fn load_to_freg(&mut self, val: &ValuePtr, freg: &FReg) {
        assert!(val.get_type().is_float());

        if let Some(const_val) = val.to_const() {
            if let ConstantClass::Float(f) = const_val.0.borrow().ext {
                self.load_float_imm(f, freg);
            }
        } else {
            let offset = *self.context.offset_map.get(&val.get_id()).unwrap();
            let offset_str = offset.to_string();

            if is_imm_12(offset) {
                self.append_inst_parts(
                    &(FLW),
                    vec![freg.print(), "s0".to_string(), offset_str],
                );
            } else {
                let addr = Reg::t(8);
                self.load_large_int64(offset as i64, &addr);
                self.append_inst_parts(
                    &(ADD),
                    vec![addr.print(), "s0".to_string(), addr.print()],
                );
                self.append_inst_parts(
                    &(FLW),
                    vec![freg.print(), addr.print(), "0".to_string()],
                );
            }
        }
    }

    pub fn load_float_imm(&mut self, val: f32, r: &FReg) {
        let bytes: i32 = unsafe { std::mem::transmute(val) };
        self.load_large_int32(bytes, &Reg::t(8));
        self.append_inst_parts(
            &(FMV_W_X),
            vec![r.print(), Reg::t(8).print()],
        );
    }

    pub fn store_from_freg(&mut self, val: &ValuePtr, r: &FReg) {
        let offset = *self.context.offset_map.get(&val.get_id()).unwrap();

        if is_imm_12(offset) {
            let offset_str = offset.to_string();
            self.append_inst_parts(
                &(FSW),
                vec![r.print(), "s0".to_string(), offset_str],
            );
        } else {
            let addr = Reg::t(8);
            self.load_large_int64(offset as i64, &addr);
            self.append_inst_parts(
                &(ADD),
                vec![addr.print(), "s0".to_string(), addr.print()],
            );
            self.append_inst_parts(
                &(FSW),
                vec![r.print(), addr.print(), "0".to_string()],
            );
        }
    }

    pub fn gen_prologue(&mut self) {
        let frame_size = self.context.frame_size;

        // 寄存器备份和栈帧设置
        if is_imm_12(-(frame_size as i32)) {
            self.append_inst("sd ra, -8(sp)".to_string());
            self.append_inst("sd s0, -16(sp)".to_string());
            self.append_inst("addi s0, sp, 0".to_string());
            self.append_inst(format!("addi sp, sp, {}", -(frame_size as i32)));
        } else {
            self.load_large_int64(frame_size as i64, &Reg::t(0));
            self.append_inst("sd ra, -8(sp)".to_string());
            self.append_inst("sd s0, -16(sp)".to_string());
            self.append_inst("sub sp, sp, t0".to_string());
            self.append_inst("add s0, sp, t0".to_string());
        }

        // 将函数参数移动到栈帧
        let func = self.context.func.upgrade().unwrap();
        let func_ptr = FunctionPtr(func);
        let mut garg_cnt = 0;
        let mut farg_cnt = 0;

        for arg in &func_ptr.0.borrow().args {
            let arg_val = arg.to_val();
            if arg_val.get_type().is_float() {
                self.store_from_freg(&arg_val, &FReg::fa(farg_cnt));
                farg_cnt += 1;
            } else {
                // int 或 pointer
                self.store_from_greg(&arg_val, &Reg::a(garg_cnt));
                garg_cnt += 1;
            }
        }
    }

    pub fn gen_epilogue(&mut self) {
        let func = self.context.func.upgrade().unwrap();
        let func_ptr = FunctionPtr(func);
        let func_name = func_ptr.to_val().get_name();

        self.output.push(AsmInst::Label(func_name.clone() + "_exit"));

        if is_imm_12(self.context.frame_size as i32) {
            self.append_inst("ld ra, -8(s0)".to_string());
            self.append_inst("ld s0, -16(s0)".to_string());
            self.append_inst(format!("addi sp, sp, {}", self.context.frame_size));
        } else {
            self.load_large_int64(self.context.frame_size as i64, &Reg::t(0));
            self.append_inst("ld ra, -8(s0)".to_string());
            self.append_inst("ld s0, -16(s0)".to_string());
            self.append_inst("add sp, sp, t0".to_string());
        }
        self.append_inst("ret".to_string());
    }

    pub fn label_name(bb: &BasicBlockPtr) -> String {
        let bb_name = bb.to_val().get_name();
        let func = bb.0.borrow().function.upgrade().unwrap();
        let func_ptr = FunctionPtr(func);
        let func_name = func_ptr.to_val().get_name();
        format!(".{}_", func_name) + &bb_name
    }

    pub fn fcmp_label_name(bb: &BasicBlockPtr, cnt: usize) -> String {
        Self::label_name(bb) + "_fcmp_" + &cnt.to_string()
    }

    pub fn print(&self) -> String {
        let mut result = String::new();
        for inst in &self.output {
            result += &inst.format();
        }
        result
    }

    pub fn copy_stmt(&mut self) {
        let bb = self.context.bb.upgrade().unwrap();
        let bb_ptr = BasicBlockPtr(bb);

        for succ_weak in &bb_ptr.0.borrow().succ_bbs {
            let succ = BasicBlockPtr(succ_weak.upgrade().unwrap());
            for inst in &succ.0.borrow().insts {
                if inst.get_inst_op_id() as u8 == OpID::Phi as u8 {
                    let operands = inst.to_user().get_operands();
                    let ops = operands.borrow();

                    // 遍历 phi 操作数（value, bb 对）
                    let mut i = 1;
                    while i < ops.len() {
                        let bb_val = &ops[i];
                        if let Some(bb_from) = bb_val.to_bb() {
                            if Rc::ptr_eq(&bb_ptr.0, &bb_from.0) {
                                let lvalue = &ops[i - 1];
                                if lvalue.get_type().is_float() {
                                    self.load_to_freg(lvalue, &FReg::fa(0));
                                    self.store_from_freg(&inst.to_user().to_val(), &FReg::fa(0));
                                } else {
                                    self.load_to_greg(lvalue, &Reg::a(0));
                                    self.store_from_greg(&inst.to_user().to_val(), &Reg::a(0));
                                }
                                break;
                            }
                        }
                        i += 2;
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn gen_ret(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        if !inst_ptr.is_void_ret() {
            let val = &ops[0];
            if val.get_type().is_float() {
                self.load_to_freg(val, &FReg::fa(0));
            } else {
                self.load_to_greg(val, &Reg::a(0));
            }
        } else {
            self.append_inst("addi a0, zero, 0".to_string());
        }

        let func = self.context.func.upgrade().unwrap();
        let func_ptr = FunctionPtr(func);
        let func_name = func_ptr.to_val().get_name();
        self.append_inst_parts("j", vec![func_name + "_exit"]);
    }

    pub fn gen_br(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);

        if inst_ptr.is_cond_br() {
            let operands = inst_ptr.to_user().get_operands();
            let ops = operands.borrow();
            let cond = &ops[0];
            let if_true = ops[1].to_bb().unwrap();
            let if_false = ops[2].to_bb().unwrap();

            self.load_to_greg(cond, &Reg::t(0));
            self.append_inst_parts("bnez", vec!["t0".to_string(), Self::label_name(&if_true)]);
            self.append_inst_parts("j", vec![Self::label_name(&if_false)]);
        } else {
            let operands = inst_ptr.to_user().get_operands();
            let ops = operands.borrow();
            let branch_bb = ops[0].to_bb().unwrap();
            self.append_inst_parts("j", vec![Self::label_name(&branch_bb)]);
        }
    }

    pub fn gen_binary(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        self.load_to_greg(&ops[0], &Reg::t(0));
        self.load_to_greg(&ops[1], &Reg::t(1));

        match inst_ptr.get_inst_op_id() {
            OpID::Add => {
                self.append_inst("add t2, t0, t1".to_string());
            }
            OpID::Sub => {
                self.append_inst("sub t2, t0, t1".to_string());
            }
            OpID::Mul => {
                self.append_inst("mul t2, t0, t1".to_string());
            }
            OpID::SDiv => {
                self.append_inst("div t2, t0, t1".to_string());
            }
            _ => panic!("Unexpected binary operation"),
        }

        self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::t(2));
    }

    pub fn gen_float_binary(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        self.load_to_freg(&ops[0], &FReg::ft(0));
        self.load_to_freg(&ops[1], &FReg::ft(1));

        match inst_ptr.get_inst_op_id() {
            OpID::FAdd => {
                self.append_inst("fadd.s ft2, ft0, ft1".to_string());
            }
            OpID::FSub => {
                self.append_inst("fsub.s ft2, ft0, ft1".to_string());
            }
            OpID::FMul => {
                self.append_inst("fmul.s ft2, ft0, ft1".to_string());
            }
            OpID::FDiv => {
                self.append_inst("fdiv.s ft2, ft0, ft1".to_string());
            }
            _ => panic!("Unexpected float binary operation"),
        }

        self.store_from_freg(&inst_ptr.to_user().to_val(), &FReg::ft(2));
    }

    pub fn gen_alloca(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let inst_val = inst_ptr.to_user().to_val();
        let inst_type = inst_val.get_type();
        let alloca_ty = inst_type.get_ptr_elem_ty().unwrap();
        let alloca_size = Self::get_type_size(&alloca_ty);

        let offset = *self.context.offset_map.get(&inst_val.get_id()).unwrap();
        let s = offset - (alloca_size as i32);

        if is_imm_12(s) {
            self.append_inst_parts("addi", vec!["t0".to_string(), "s0".to_string(), s.to_string()]);
        } else {
            self.load_large_int32(s, &Reg::t(1));
            self.append_inst_parts("add", vec!["t0".to_string(), "s0".to_string(), "t1".to_string()]);
        }
        self.store_from_greg(&inst_val, &Reg::t(0));
    }

    pub fn gen_load(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();
        let ptr = &ops[0];
        let inst_val = inst_ptr.to_user().to_val();
        let val_type = inst_val.get_type();

        self.load_to_greg(ptr, &Reg::t(0));

        if val_type.is_float() {
            self.append_inst_parts(FLW, vec!["ft0".to_string(), "0(t0)".to_string()]);
            self.store_from_freg(&inst_val, &FReg::ft(0));
        } else {
            if val_type.is_bool() {
                self.append_inst_parts("lb", vec!["t0".to_string(), "0(t0)".to_string()]);
            } else if val_type.is_int() {
                self.append_inst_parts("lw", vec!["t0".to_string(), "0(t0)".to_string()]);
            } else {
                // 指针
                self.append_inst_parts("ld", vec!["t0".to_string(), "0(t0)".to_string()]);
            }
            self.store_from_greg(&inst_val, &Reg::t(0));
        }
    }

    pub fn gen_store(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();
        let val = &ops[0];
        let ptr = &ops[1];
        let val_type = val.get_type();

        self.load_to_greg(ptr, &Reg::t(0));

        if val_type.is_float() {
            self.load_to_freg(val, &FReg::ft(1));
            self.append_inst_parts("fst.s", vec!["ft1".to_string(), "t0".to_string(), "0".to_string()]);
        } else if val_type.is_int() {
            self.load_to_greg(val, &Reg::t(1));
            self.append_inst_parts("sw", vec!["t1".to_string(), "0(t0)".to_string()]);
        } else if val_type.is_bool() {
            self.load_to_greg(val, &Reg::t(1));
            self.append_inst_parts("sb", vec!["t1".to_string(), "0(t0)".to_string()]);
        } else if val_type.is_ptr() {
            self.load_to_greg(val, &Reg::t(1));
            self.append_inst_parts("sd", vec!["t1".to_string(), "0(t0)".to_string()]);
        }
    }

    pub fn gen_icmp(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        self.load_to_greg(&ops[0], &Reg::t(0));
        self.load_to_greg(&ops[1], &Reg::t(1));

        match inst_ptr.get_inst_op_id() {
            OpID::Ge => {
                // t0 >= t1 <=> !(t0 < t1) <=> (t0 < t1) xor 1
                self.append_inst("slt t2, t0, t1".to_string());
                self.append_inst("xori t2, t2, 1".to_string());
            }
            OpID::Gt => {
                // t0 > t1 <=> t1 < t0
                self.append_inst("slt t2, t1, t0".to_string());
            }
            OpID::Le => {
                // t0 <= t1 <=> !(t1 < t0) <=> (t1 < t0) xor 1
                self.append_inst("slt t2, t1, t0".to_string());
                self.append_inst("xori t2, t2, 1".to_string());
            }
            OpID::Lt => {
                // t0 < t1
                self.append_inst("slt t2, t0, t1".to_string());
            }
            OpID::Eq => {
                // t0 == t1 <=> (t0 xor t1) == 0 <=> (unsigned)(t0 xor t1) < 1
                self.append_inst("xor t2, t0, t1".to_string());
                self.append_inst("sltiu t2, t2, 1".to_string());
            }
            OpID::Ne => {
                // t0 != t1 <=> (t0 xor t1) != 0 <=> (unsigned)(t0 xor t1) > 0
                self.append_inst("xor t2, t0, t1".to_string());
                self.append_inst("sltu t2, zero, t2".to_string());
            }
            _ => panic!("Unexpected icmp operation"),
        }

        self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::t(2));
    }

    pub fn gen_fcmp(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        self.load_to_freg(&ops[0], &FReg::ft(0));
        self.load_to_freg(&ops[1], &FReg::ft(1));

        // RISC-V 只有 fle.s, flt.s, feq.s
        // 对于 ge, gt 需要交换操作数或取反结果
        match inst_ptr.get_inst_op_id() {
            OpID::FGe => {
                // ft0 >= ft1 => ft1 <= ft0
                self.append_inst("fle.s t2, ft1, ft0".to_string());
            }
            OpID::FGt => {
                // ft0 > ft1 => ft1 < ft0
                self.append_inst("flt.s t2, ft1, ft0".to_string());
            }
            OpID::FLe => {
                self.append_inst("fle.s t2, ft0, ft1".to_string());
            }
            OpID::FLt => {
                self.append_inst("flt.s t2, ft0, ft1".to_string());
            }
            OpID::FEq => {
                self.append_inst("feq.s t2, ft0, ft1".to_string());
            }
            OpID::FNe => {
                // ft0 != ft1 => !(ft0 == ft1)
                self.append_inst("feq.s t2, ft0, ft1".to_string());
                self.append_inst("xori t2, t2, 1".to_string());
            }
            _ => panic!("Unexpected fcmp operation"),
        }

        // 结果已经在 t2 中（1 或 0）
        self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::t(2));
    }

    pub fn gen_zext(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();
        let source = &ops[0];

        self.load_to_greg(source, &Reg::t(0));
        self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::t(0));
    }

    pub fn gen_call(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();
        let func = ops[0].to_function().unwrap();

        let mut gidx = 0;
        let mut fidx = 0;

        for (i, arg) in func.0.borrow().args.iter().enumerate() {
            let arg_type = arg.to_val().get_type();
            if arg_type.is_ptr() || arg_type.is_int() || arg_type.is_bool() {
                self.load_to_greg(&ops[i + 1], &Reg::a(gidx));
                gidx += 1;
            } else if arg_type.is_float() {
                self.load_to_freg(&ops[i + 1], &FReg::fa(fidx));
                fidx += 1;
            }
        }

        self.append_inst_parts(CALL, vec![func.to_val().get_name()]);

        let ret_type = func.get_return_type();
        if ret_type.is_int() || ret_type.is_bool() {
            self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::a(0));
        } else if ret_type.is_float() {
            self.store_from_freg(&inst_ptr.to_user().to_val(), &FReg::fa(0));
        }
    }

    pub fn gen_gep(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();
        let ptr = &ops[0];

        self.load_to_greg(ptr, &Reg::t(0));

        let mut current_type = ptr.get_type();
        for i in 1..ops.len() {
            if current_type.is_ptr() {
                current_type = current_type.get_ptr_elem_ty().unwrap();
            } else if current_type.is_arr() {
                current_type = current_type.get_arr_elem_ty().unwrap();
            }

            self.load_to_greg(&ops[i], &Reg::t(1));
            let new_off = Self::get_type_size(&current_type);
            self.load_large_int32(new_off as i32, &Reg::t(2));
            self.append_inst("mul t3, t1, t2".to_string());
            self.append_inst("add t0, t0, t3".to_string());
        }

        self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::t(0));
    }

    pub fn gen_sitofp(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        self.load_to_greg(&ops[0], &Reg::t(0));
        self.append_inst("fmv.w.x ft0, t0".to_string());
        self.append_inst("fcvt.s.w ft1, ft0".to_string());
        self.store_from_freg(&inst_ptr.to_user().to_val(), &FReg::ft(1));
    }

    pub fn gen_fptosi(&mut self) {
        let inst = self.context.inst.upgrade().unwrap();
        let inst_ptr = crate::ir::user::InstPtr(inst);
        let operands = inst_ptr.to_user().get_operands();
        let ops = operands.borrow();

        self.load_to_freg(&ops[0], &FReg::ft(0));
        self.append_inst("fcvt.w.s ft1, ft0, rtz".to_string());
        self.append_inst("fmv.x.w t0, ft1".to_string());
        self.store_from_greg(&inst_ptr.to_user().to_val(), &Reg::t(0));
    }

    pub fn run(&mut self) {
        // 确保每个函数的每个基本块名称都已设置
        let funcs: Vec<_> = self.m.0.borrow().func_list.iter().map(|f| f.clone()).collect();
        for mut func in funcs {
            func.set_inst_name();
        }

        // 使用 GNU 汇编指令为全局变量分配空间
        if !self.m.0.borrow().gv_list.is_empty() {
            self.output.push(AsmInst::Comment("Global variables".to_string()));
            self.output.push(AsmInst::Attribute(".text".to_string()));
            self.output.push(AsmInst::Attribute(".section .bss, \"aw\", @nobits".to_string()));

            let globals: Vec<_> = self.m.0.borrow().gv_list.iter().map(|g| g.clone()).collect();
            for global in &globals {
                let gv_val = global.to_val();
                let gv_name = gv_val.get_name();
                let gv_type = gv_val.get_type();
                let size = Self::get_type_size(&gv_type.get_ptr_elem_ty().unwrap());

                self.output.push(AsmInst::Attribute(format!(".globl {}", gv_name)));
                self.output.push(AsmInst::Attribute(format!(".type {}, @object", gv_name)));
                self.output.push(AsmInst::Attribute(format!(".size {}, {}", gv_name, size)));
                self.output.push(AsmInst::Label(gv_name));
                self.output.push(AsmInst::Attribute(format!(".space {}", size)));
            }
        }

        // 函数代码段
        self.output.push(AsmInst::Attribute(".text".to_string()));

        let funcs2: Vec<_> = self.m.0.borrow().func_list.iter().map(|f| f.clone()).collect();
        for func in &funcs2 {
            // 跳过声明
            if func.0.borrow().bbs.is_empty() {
                continue;
            }

            // 更新上下文
            self.context.clear();
            self.context.func = Rc::downgrade(&func.0);

            let func_name = func.to_val().get_name();

            // 函数信息
            self.output.push(AsmInst::Attribute(format!(".globl {}", func_name)));
            self.output.push(AsmInst::Attribute(format!(".type {}, @function", func_name)));
            self.output.push(AsmInst::Label(func_name.clone()));

            // 分配函数栈帧
            self.allocate();
            // 生成序言
            self.gen_prologue();

            for bb in &func.0.borrow().bbs {
                self.context.bb = Rc::downgrade(&bb.0);
                self.context.fcmp_cnt = 0;
                self.output.push(AsmInst::Label(Self::label_name(bb)));

                for inst in &bb.0.borrow().insts {
                    // 用于调试 - 打印指令类型
                    let inst_op = inst.get_inst_op_id();
                    let inst_print = match inst_op {
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
                        OpID::Ge => "ge",
                        OpID::Gt => "gt",
                        OpID::Le => "le",
                        OpID::Lt => "lt",
                        OpID::Eq => "eq",
                        OpID::Ne => "ne",
                        OpID::FGe => "fge",
                        OpID::FGt => "fgt",
                        OpID::FLe => "fle",
                        OpID::FLt => "flt",
                        OpID::FEq => "feq",
                        OpID::FNe => "fne",
                        OpID::Phi => "phi",
                        OpID::Call => "call",
                        OpID::GetElementPtr => "getelementptr",
                        OpID::ZExt => "zext",
                        OpID::FPToSI => "fptosi",
                        OpID::SIToFP => "sitofp",
                    };
                    self.output.push(AsmInst::Comment(inst_print.to_string()));

                    self.context.inst = Rc::downgrade(&inst.0);

                    match inst.get_inst_op_id() {
                        OpID::Ret => {
                            self.gen_ret();
                        }
                        OpID::Br => {
                            self.copy_stmt();
                            self.gen_br();
                        }
                        OpID::Add | OpID::Sub | OpID::Mul | OpID::SDiv => {
                            self.gen_binary();
                        }
                        OpID::FAdd | OpID::FSub | OpID::FMul | OpID::FDiv => {
                            self.gen_float_binary();
                        }
                        OpID::Alloca => {
                            self.gen_alloca();
                        }
                        OpID::Load => {
                            self.gen_load();
                        }
                        OpID::Store => {
                            self.gen_store();
                        }
                        OpID::Ge | OpID::Gt | OpID::Le | OpID::Lt | OpID::Eq | OpID::Ne => {
                            self.gen_icmp();
                        }
                        OpID::FGe | OpID::FGt | OpID::FLe | OpID::FLt | OpID::FEq | OpID::FNe => {
                            self.gen_fcmp();
                        }
                        OpID::Phi => {
                            // Phi 指令由 copy_stmt 处理
                        }
                        OpID::Call => {
                            self.gen_call();
                        }
                        OpID::GetElementPtr => {
                            self.gen_gep();
                        }
                        OpID::ZExt => {
                            self.gen_zext();
                        }
                        OpID::FPToSI => {
                            self.gen_fptosi();
                        }
                        OpID::SIToFP => {
                            self.gen_sitofp();
                        }
                    }
                }
            }

            // 生成尾声
            self.gen_epilogue();
        }
    }
}
