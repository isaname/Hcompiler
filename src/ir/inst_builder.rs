use std::rc::Rc;

use crate::ir::{
    module::ModulePtr,
    type_::Type,
    user::{InstPtr, OpID},
    value::{BasicBlockPtr, FunctionPtr, ValuePtr},
};

pub struct InstBuilder {
    bb: Option<BasicBlockPtr>,
    m: ModulePtr,
}

impl InstBuilder {
    pub fn new(m: ModulePtr) -> Self {
        InstBuilder { bb: None, m }
    }

    pub fn get_module(&self) -> ModulePtr {
        self.m.clone()
    }

    pub fn get_insert_block(&self) -> BasicBlockPtr {
        self.bb.as_ref().expect("insert block not set").clone()
    }

    pub fn set_insert_point(&mut self, bb: BasicBlockPtr) {
        self.bb = Some(bb);
    }

    fn bb(&self) -> BasicBlockPtr {
        self.get_insert_block()
    }

    // ===== Integer Binary =====

    pub fn create_iadd(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::ibinary_inst(OpID::Add, lhs, rhs, self.bb())
    }

    pub fn create_isub(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::ibinary_inst(OpID::Sub, lhs, rhs, self.bb())
    }

    pub fn create_imul(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::ibinary_inst(OpID::Mul, lhs, rhs, self.bb())
    }

    pub fn create_isdiv(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::ibinary_inst(OpID::SDiv, lhs, rhs, self.bb())
    }

    // ===== Integer Compare =====

    pub fn create_icmp_eq(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::icmp_inst(OpID::Eq, lhs, rhs, self.bb())
    }

    pub fn create_icmp_ne(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::icmp_inst(OpID::Ne, lhs, rhs, self.bb())
    }

    pub fn create_icmp_gt(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::icmp_inst(OpID::Gt, lhs, rhs, self.bb())
    }

    pub fn create_icmp_ge(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::icmp_inst(OpID::Ge, lhs, rhs, self.bb())
    }

    pub fn create_icmp_lt(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::icmp_inst(OpID::Lt, lhs, rhs, self.bb())
    }

    pub fn create_icmp_le(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::icmp_inst(OpID::Le, lhs, rhs, self.bb())
    }

    // ===== Float Binary =====

    pub fn create_fadd(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fbinary_inst(OpID::FAdd, lhs, rhs, self.bb())
    }

    pub fn create_fsub(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fbinary_inst(OpID::FSub, lhs, rhs, self.bb())
    }

    pub fn create_fmul(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fbinary_inst(OpID::FMul, lhs, rhs, self.bb())
    }

    pub fn create_fdiv(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fbinary_inst(OpID::FDiv, lhs, rhs, self.bb())
    }

    // ===== Float Compare =====

    pub fn create_fcmp_eq(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fcmp_inst(OpID::FEq, lhs, rhs, self.bb())
    }

    pub fn create_fcmp_ne(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fcmp_inst(OpID::FNe, lhs, rhs, self.bb())
    }

    pub fn create_fcmp_gt(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fcmp_inst(OpID::FGt, lhs, rhs, self.bb())
    }

    pub fn create_fcmp_ge(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fcmp_inst(OpID::FGe, lhs, rhs, self.bb())
    }

    pub fn create_fcmp_lt(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fcmp_inst(OpID::FLt, lhs, rhs, self.bb())
    }

    pub fn create_fcmp_le(&self, lhs: ValuePtr, rhs: ValuePtr) -> InstPtr {
        InstPtr::fcmp_inst(OpID::FLe, lhs, rhs, self.bb())
    }

    // ===== Call =====

    pub fn create_call(&self, func: FunctionPtr, args: Vec<ValuePtr>) -> InstPtr {
        InstPtr::call_inst(func, args, self.bb())
    }

    // ===== Branch =====

    pub fn create_br(&self, if_true: BasicBlockPtr) -> InstPtr {
        InstPtr::br_inst(if_true, self.bb())
    }

    pub fn create_cond_br(&self, cond: ValuePtr, if_true: BasicBlockPtr, if_false: BasicBlockPtr) -> InstPtr {
        InstPtr::cond_br_inst(cond, if_true, if_false, self.bb())
    }

    // ===== Return =====

    pub fn create_ret(&self, val: ValuePtr) -> InstPtr {
        InstPtr::ret_inst(val, self.bb())
    }

    pub fn create_void_ret(&self) -> InstPtr {
        InstPtr::void_ret_inst(self.bb())
    }

    // ===== GEP =====

    pub fn create_gep(&self, ptr: ValuePtr, idxs: Vec<ValuePtr>) -> InstPtr {
        InstPtr::gep_inst(ptr, idxs, self.bb())
    }

    // ===== Store / Load =====

    pub fn create_store(&self, val: ValuePtr, ptr: ValuePtr) -> InstPtr {
        InstPtr::store_inst(val, ptr, self.bb())
    }

    pub fn create_load(&self, ptr: ValuePtr) -> InstPtr {
        assert!(ptr.get_type().is_ptr(), "ptr must be pointer type");
        InstPtr::load_inst(ptr, self.bb())
    }

    // ===== Alloca =====

    pub fn create_alloca(&self, ty: Rc<Type>) -> InstPtr {
        InstPtr::alloca_inst(ty, self.bb())
    }

    // ===== Type Conversion =====

    pub fn create_zext(&self, val: ValuePtr, ty: Rc<Type>) -> InstPtr {
        InstPtr::zext_inst(val, ty, self.bb())
    }

    pub fn create_sitofp(&self, val: ValuePtr) -> InstPtr {
        InstPtr::sitofp_inst(val, self.bb())
    }

    pub fn create_fptosi(&self, val: ValuePtr, ty: Rc<Type>) -> InstPtr {
        InstPtr::fptosi_inst(val, ty, self.bb())
    }
}
