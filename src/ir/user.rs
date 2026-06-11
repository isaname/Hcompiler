use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::id;
use std::ptr;
use std::rc::{Rc, Weak};

use crate::downgrade;
use crate::ir::module::Module;
use crate::ir::module::ModulePtr;
use crate::make_ptr;
use crate::weak_ptr;
use crate::{opt_ptr, ptr};

use super::type_::*;
use super::value::*;

pub struct User {
    pub(crate) value: ValuePtr,
    pub(crate) operands: ptr!(Vec<ValuePtr>),
    pub(crate) class: Option<UserClass>
}
pub enum UserClass {
    GlobalVariable(weak_ptr!(GlobalVariable)),
    Inst(weak_ptr!(Inst)),
    Constant(weak_ptr!(Constant))
}
pub struct UserPtr(pub ptr!(User));
pub struct GlobalVariable {
    pub(crate) user: UserPtr,
    pub(crate) is_const: bool,
    pub(crate) init_val: Option<ConstantPtr>
}
pub struct GVPtr(pub ptr!(GlobalVariable));

#[derive(Clone, Copy)]
pub enum OpID{
    // Terminator Instructions
    Ret,
    Br,
    // Standard binary operators
    Add,
    Sub,
    Mul,
    SDiv,
    
    // Float binary operators
    FAdd,
    FSub,
    FMul,
    FDiv,
    
    // Memory operators
    Alloca,
    Load,
    Store,
    
    // Int compare operators
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
    Ne,
    
    // Float compare operators
    FGe,
    FGt,
    FLe,
    FLt,
    FEq,
    FNe,
    
    // Other operators
    Phi,
    Call,
    GetElementPtr,
    ZExt,   // zero extend
    FPToSI, // float to signed int
    SIToFP, // signed int to float
}
pub struct Inst {
    pub user: UserPtr,
    pub op_id: OpID,
    pub bb: weak_ptr!(BasicBlock)
}
pub struct  InstPtr(pub ptr!(Inst));

pub enum ConstantClass {
    Int(i32),
    Zero,
    Float(f32),
    Arr(Vec<ConstantPtr>)
}

pub struct Constant {
    pub user: UserPtr,
    pub ext: ConstantClass
}

pub struct ConstantPtr(pub ptr!(Constant));