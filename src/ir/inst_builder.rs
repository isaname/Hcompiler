use crate::{ir::{module::{Module, ModulePtr}, value::Value}, ptr, weak_ptr};
use std::{rc::Rc, cell::RefCell};
use crate::module_ptr;
use crate::ir::type_::Type;
struct InstBuilder {
    bb: ptr!(Value), // * BasicBlock
    m: ModulePtr
}

impl InstBuilder {
    /// 创建一个新的指令构建器，初始插入点为指定的基本块
    ///
    /// # 参数
    /// - `bb` : 目标基本块（必须是 BasicBlock 类型）
    /// - `m`  : 所属模块（用于类型查询、常量创建等）
    pub fn new(bb: ptr!(Value), m:ModulePtr) -> Self {
        assert!(bb.borrow().is_bb(), "bb must be a BasicBlock");
        InstBuilder { bb, m }
    }

    /// 获取当前插入的基本块
    pub fn get_insert_block(&self) -> ptr!(Value) {
        self.bb.clone()
    }

    /// 设置新的插入点（后续创建的指令都会插入到这个基本块）
    pub fn set_insert_point(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb(), "bb must be a BasicBlock");
        self.bb = bb;
    }

    // ────────────────────────────────────────────────
    // 整数二元运算
    // ────────────────────────────────────────────────

    /// 创建整数加法指令：%res = add lhs, rhs
    pub fn create_iadd(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_add(lhs, rhs, self.bb.clone())
    }

    /// 创建整数减法指令：%res = sub lhs, rhs
    pub fn create_isub(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_sub(lhs, rhs, self.bb.clone())
    }

    /// 创建整数乘法指令：%res = mul lhs, rhs
    pub fn create_imul(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_mul(lhs, rhs, self.bb.clone())
    }

    /// 创建有符号整数除法指令：%res = sdiv lhs, rhs
    pub fn create_isdiv(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_div(lhs, rhs, self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 浮点二元运算
    // ────────────────────────────────────────────────

    pub fn create_fadd(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fadd(lhs, rhs, self.bb.clone())
    }

    pub fn create_fsub(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fsub(lhs, rhs, self.bb.clone())
    }

    pub fn create_fmul(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fmul(lhs, rhs, self.bb.clone())
    }

    pub fn create_fdiv(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fdiv(lhs, rhs, self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 整数比较
    // ────────────────────────────────────────────────

    pub fn create_icmp_eq(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_eq(lhs, rhs, self.bb.clone())
    }

    pub fn create_icmp_ne(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_ne(lhs, rhs, self.bb.clone())
    }

    pub fn create_icmp_gt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_gt(lhs, rhs, self.bb.clone())
    }

    pub fn create_icmp_ge(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_ge(lhs, rhs, self.bb.clone())
    }

    pub fn create_icmp_lt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_lt(lhs, rhs, self.bb.clone())
    }

    pub fn create_icmp_le(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_le(lhs, rhs, self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 浮点比较
    // ────────────────────────────────────────────────

    pub fn create_fcmp_eq(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_feq(lhs, rhs, self.bb.clone())
    }

    pub fn create_fcmp_ne(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fne(lhs, rhs, self.bb.clone())
    }

    pub fn create_fcmp_gt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fgt(lhs, rhs, self.bb.clone())
    }

    pub fn create_fcmp_ge(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fge(lhs, rhs, self.bb.clone())
    }

    pub fn create_fcmp_lt(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_flt(lhs, rhs, self.bb.clone())
    }

    pub fn create_fcmp_le(&self, lhs: ptr!(Value), rhs: ptr!(Value)) -> Value {
        Value::create_inst_fle(lhs, rhs, self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 控制流
    // ────────────────────────────────────────────────

    /// 无条件跳转：br label %if_true
    pub fn create_br(&self, if_true: ptr!(Value)) -> Value {
        Value::create_inst_br(if_true, self.bb.clone())
    }

    /// 条件跳转：br i1 %cond, label %if_true, label %if_false
    pub fn create_cond_br(
        &self,
        cond: ptr!(Value),
        if_true: ptr!(Value),
        if_false: ptr!(Value),
    ) -> Value {
        Value::create_inst_cond_br(cond, if_true, if_false, self.bb.clone())
    }

    /// 返回指令（带返回值）：ret %val
    pub fn create_ret(&self, val: ptr!(Value)) -> Value {
        Value::create_inst_ret(val, self.bb.clone())
    }

    /// 空返回：ret void
    pub fn create_void_ret(&self) -> Value {
        Value::create_inst_void_ret(self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 内存操作
    // ────────────────────────────────────────────────

    /// 取元素指针：%res = getelementptr %ptr, idxs...
    pub fn create_gep(&self, ptr: ptr!(Value), idxs: Vec<ptr!(Value)>) -> Value {
        Value::create_inst_gep(ptr, idxs, self.bb.clone())
    }

    /// 存储指令：store %val, %ptr
    pub fn create_store(&self, val: ptr!(Value), ptr: ptr!(Value)) -> Value {
        Value::create_inst_store(val, ptr, self.bb.clone())
    }

    /// 加载指令：%res = load %ptr
    pub fn create_load(&self, ptr: ptr!(Value)) -> Value {
        assert!(ptr.borrow().get_type().is_ptr(), "ptr must be pointer type");
        Value::create_inst_load(ptr, self.bb.clone())
    }

    /// 栈上分配：%res = alloca %ty
    pub fn create_alloca(&self, ty: Rc<Type>) -> Value {
        Value::create_inst_alloc(ty, self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 类型转换
    // ────────────────────────────────────────────────

    /// 零扩展（目前主要用于 bool -> int）
    pub fn create_zext(&self, val: ptr!(Value), ty: Rc<Type>) -> Value {
        Value::create_inst_zext(val, ty, self.bb.clone())
    }

    /// 有符号整数 → 浮点
    pub fn create_sitofp(&self, val: ptr!(Value)) -> Value {
        Value::create_inst_si2fp(val, self.bb.clone())
    }

    /// 浮点 → 有符号整数
    pub fn create_fptosi(&self, val: ptr!(Value), ty: Rc<Type>) -> Value {
        Value::create_inst_fp2si(val, ty, self.bb.clone())
    }

    // ────────────────────────────────────────────────
    // 函数调用
    // ────────────────────────────────────────────────

    /// 函数调用：%res = call %func(args...)
    pub fn create_call(&self, func: ptr!(Value), args: Vec<ptr!(Value)>) -> Value {
        Value::create_inst_call(func, args, self.bb.clone())
    }
}