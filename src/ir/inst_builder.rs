use crate::ir::type_::Type;
use crate::{make_ptr, module_ptr, opt_ptr};
use crate::{
    ir::{
        module::{Module, ModulePtr},
        value::Value,
    },
    ptr, weak_ptr,
};
use std::{cell::RefCell, rc::Rc};
pub struct InstBuilder {
    bb: opt_ptr!(Value), // * BasicBlock
    m: ModulePtr,
}

impl InstBuilder {
    /// 创建一个新的指令构建器，初始插入点为指定的基本块
    ///
    /// # 参数
    /// - `bb` : 目标基本块（必须是 BasicBlock 类型）
    /// - `m`  : 所属模块（用于类型查询、常量创建、类型创建）
    pub fn new(m: ModulePtr) -> Self {
        InstBuilder { bb:None, m }
    }

    /// 获取当前插入的基本块
    pub fn get_insert_block(&self) -> ptr!(Value) {
        self.bb.as_ref().unwrap().clone()
    }

    /// 设置新的插入点（后续创建的指令都会插入到这个基本块）
    pub fn set_insert_point(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb(), "bb must be a BasicBlock");
        self.bb = Some(bb);
    }

    // ────────────────────────────────────────────────
    // 整数二元运算
    // ────────────────────────────────────────────────

    /// 创建整数加法指令：`%res = add lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是整数类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为i32（通常 i32）
    pub fn create_iadd(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_add(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    /// 创建整数减法指令：`%res = sub lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是整数类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为i32
    pub fn create_isub(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_sub(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    /// 创建整数乘法指令：`%res = mul lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是整数类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为i32
    pub fn create_imul(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_mul(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    /// 创建有符号整数除法指令：`%res = sdiv lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是整数类型（除数不为零需由前端保证）
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为i32
    pub fn create_isdiv(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_div(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    // ────────────────────────────────────────────────
    // 浮点二元运算
    // ────────────────────────────────────────────────
    /// 创建浮点加法指令：`%res = fadd lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是浮点类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为f32（通常 float）
    pub fn create_fadd(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fadd(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点减法指令：`%res = fsub lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是浮点类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为f32
    pub fn create_fsub(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fsub(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点乘法指令：`%res = fmul lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是浮点类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为f32
    pub fn create_fmul(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fmul(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点除法指令：`%res = fdiv lhs, rhs`
    ///
    /// # 参数约束
    /// - `lhs`, `rhs`：必须都是浮点类型
    ///
    /// # 返回值
    /// - `Value` —— 结果值，类型为f32
    pub fn create_fdiv(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fdiv(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    // ────────────────────────────────────────────────
    // 整数比较
    // ────────────────────────────────────────────────
    /// 创建整数相等比较：`%res = icmp eq lhs, rhs` → bool
    pub fn create_icmp_eq(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_eq(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建整数不相等比较：`%res = icmp ne lhs, rhs` → bool
    pub fn create_icmp_ne(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_ne(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建整数有符号大于比较：`%res = icmp sgt lhs, rhs` → bool
    pub fn create_icmp_gt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_gt(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建整数有符号大于等于比较：`%res = icmp sge lhs, rhs` → bool
    pub fn create_icmp_ge(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_ge(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建整数有符号小于比较：`%res = icmp slt lhs, rhs` → bool
    pub fn create_icmp_lt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_lt(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建整数有符号小于等于比较：`%res = icmp sle lhs, rhs` → bool
    pub fn create_icmp_le(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_le(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    /// bool -> bool
    pub fn create_unary_not(&self, val: ptr!(Value)) -> Value{
        let zero = Value::create_const_zero(self.m.get_bool_ty());
        self.create_icmp_eq(val, make_ptr!(zero))
    }

    /// int -> int , float -> float
    pub fn create_unary_neg(&self, val: ptr!(Value)) -> Value {
        if val.borrow().get_type().is_int() {
            let zero = Value::create_const_zero(self.m.get_int_ty());
            return self.create_isub(make_ptr!(zero), val);
        } 
        if val.borrow().get_type().is_float() {
            let zero = Value::create_const_float(self.m.get_float_ty(), 0.);
            return self.create_fsub(make_ptr!(zero), val);
        } 
        panic!("neg 指令只对int 和 float有效")
    }

    // ────────────────────────────────────────────────
    // 浮点比较
    // ────────────────────────────────────────────────
    /// 创建浮点相等比较：`%res = fcmp oeq lhs, rhs` → bool
    pub fn create_fcmp_eq(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_feq(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点不相等比较：`%res = fcmp one lhs, rhs` → bool
    pub fn create_fcmp_ne(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fne(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点大于比较：`%res = fcmp ogt lhs, rhs` → bool
    pub fn create_fcmp_gt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fgt(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点大于等于比较：`%res = fcmp oge lhs, rhs` → bool
    pub fn create_fcmp_ge(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fge(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点小于比较：`%res = fcmp olt lhs, rhs` → bool
    pub fn create_fcmp_lt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_flt(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }
    /// 创建浮点小于等于比较：`%res = fcmp ole lhs, rhs` → bool
    pub fn create_fcmp_le(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fle(lhs, rhs, self.bb.as_ref().unwrap().clone())
    }

    // ────────────────────────────────────────────────
    // 控制流
    // ────────────────────────────────────────────────

    /// 创建无条件跳转指令：`br label %if_true`
    ///
    /// # 参数约束
    /// - `if_true`：目标基本块（必须是 BasicBlock ）
    ///
    /// # 返回值
    /// - `Value` —— 跳转指令本身（类型通常为 void）
    pub fn create_br(&self, if_true: ptr!(Value)) -> Value {
        Value::create_inst_br(if_true, self.bb.as_ref().unwrap().clone())
    }

    /// 创建条件分支指令：`br bool %cond, label %if_true, label %if_false`
    ///
    /// # 参数约束
    /// - `cond`     ：条件值，必须是 bool 类型
    /// - `if_true`  ：条件为真时的目标基本块
    /// - `if_false` ：条件为假时的目标基本块
    ///
    /// # 返回值
    /// - `Value` —— 分支指令本身（类型通常为 void）
    pub fn create_cond_br(
        &self,
        cond: ptr!(Value),
        if_true: ptr!(Value),
        if_false: ptr!(Value),
    ) -> Value {
        Value::create_inst_cond_br(cond, if_true, if_false, self.bb.as_ref().unwrap().clone())
    }

    /// 创建带返回值的返回指令：`ret %val`
    ///
    /// # 参数约束
    /// - `val`：返回值，其类型必须与当前函数的返回类型匹配
    ///
    /// # 返回值
    /// - `Value` —— 返回指令本身（类型通常为 void）
    pub fn create_ret(&self, val: ptr!(Value)) -> Value {
        Value::create_inst_ret(val, self.bb.as_ref().unwrap().clone())
    }

    /// 创建空返回指令：`ret void`
    ///
    /// # 参数约束
    /// - 当前函数的返回类型必须为 void
    ///
    /// # 返回值
    /// - `Value` —— 返回指令本身（类型通常为 void）
    pub fn create_void_ret(&self) -> Value {
        Value::create_inst_void_ret(self.bb.as_ref().unwrap().clone())
    }

    // ────────────────────────────────────────────────
    // 内存操作
    // ────────────────────────────────────────────────

    /// 创建取元素指针指令：`%res = getelementptr %ptr, ...`
    ///
    /// # 参数约束
    /// - `ptr`  ：基址指针，必须是指针类型
    /// - `idxs` ：索引列表
    ///   - 每个索引必须是整数类型或 bool
    ///   - 索引序列必须与指针指向的类型结构匹配
    ///
    /// # 返回值
    /// - `Value` —— 新的指针类型，指向最终偏移后的元素
    pub fn create_gep(&self, ptr: ptr!(Value), idxs: Vec<ptr!(Value)>) -> Value {
        Value::create_inst_gep(ptr, idxs, self.bb.as_ref().unwrap().clone())
    }

    /// 创建存储指令：`store %val, %ptr`
    ///
    /// # 参数约束
    /// - `val`：要存储的值
    /// - `ptr`：目标地址，必须是指针类型，且指向的元素类型与 `val` 匹配
    ///
    /// # 返回值
    /// - `Value` —— store 指令本身（类型通常为 void）
    pub fn create_store(&self, val: ptr!(Value), ptr: ptr!(Value)) -> Value {
        Value::create_inst_store(val, ptr, self.bb.as_ref().unwrap().clone())
    }

    /// 创建加载指令：`%res = load %ptr`
    ///
    /// # 参数约束
    /// - `ptr`：要加载的指针，必须是指针类型
    ///
    /// # 返回值
    /// - `Value` —— 加载到的值，类型为指针指向的元素类型
    pub fn create_load(&self, ptr: ptr!(Value)) -> Value {
        assert!(ptr.borrow().get_type().is_ptr(), "ptr must be pointer type");
        Value::create_inst_load(ptr, self.bb.as_ref().unwrap().clone())
    }

    /// 创建栈上分配指令：`%res = alloca %ty`
    ///
    /// # 参数约束
    /// - `ty`：要分配的类型（支持i32、bool、float、数组、指针等）
    ///
    /// # 返回值
    /// - `Value` —— 指向分配空间的指针（类型为 `ptr ty`）
    pub fn create_alloca(&self, ty: Rc<Type>) -> Value {
        Value::create_inst_alloc(ty, self.bb.as_ref().unwrap().clone())
    }

    // ────────────────────────────────────────────────
    // 类型转换
    // ────────────────────────────────────────────────

    /// 创建零扩展指令（当前主要支持 bool → integer）
    ///
    /// # 参数约束
    /// - `val`：输入值，必须为 bool（bool）
    ///
    /// # 返回值
    /// - `Value` —— 扩展后的值，类型为 i32
    pub fn create_zext(&self, val: ptr!(Value)) -> Value {
        Value::create_inst_zext(val, self.bb.as_ref().unwrap().clone())
    }

    /// 创建有符号整数到浮点转换：`sitofp`
    ///
    /// # 参数约束
    /// - `val`：有符号整数或 bool
    ///
    /// # 返回值
    /// - `Value` —— 转换后的浮点值（目标类型为模块默认浮点类型f32）
    pub fn create_sitofp(&self, val: ptr!(Value)) -> Value {
        Value::create_inst_si2fp(val, self.bb.as_ref().unwrap().clone())
    }

    /// 创建浮点到有符号整数转换：`fptosi`
    ///
    /// # 参数约束
    /// - `val`：浮点值
    /// - `ty` ：i32 或 bool
    ///
    /// # 返回值
    /// - `Value` —— 转换后的整数值，类型为 `ty`
    pub fn create_fptosi(&self, val: ptr!(Value), ty: Rc<Type>) -> Value {
        Value::create_inst_fp2si(val, ty, self.bb.as_ref().unwrap().clone())
    }

    // ────────────────────────────────────────────────
    // 函数调用
    // ────────────────────────────────────────────────

    /// 创建函数调用指令：`%res = call %func(args...)`
    ///
    /// # 参数约束
    /// - `func`：被调用函数（必须是 Function ）
    /// - `args`：参数列表
    ///   - 数量必须与函数签名匹配
    ///   - 每个参数类型必须与函数对应形参类型一致
    ///
    /// # 返回值
    /// - `Value` —— 调用结果
    ///   - 若函数有返回值，则为该类型
    ///   - 若函数返回 void，则为 void 类型
    pub fn create_call(&self, func: ptr!(Value), args: Vec<ptr!(Value)>) -> Value {
        Value::create_inst_call(func, args, self.bb.as_ref().unwrap().clone())
    }
}
