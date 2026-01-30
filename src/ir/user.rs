use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::id;
use std::rc::{Rc, Weak};

use crate::downgrade;
use crate::ir::module::Module;
use crate::ir::module::ModulePtr;
use crate::make_ptr;
use crate::weak_ptr;
use crate::{opt_ptr, ptr};

use super::type_::*;
use super::value::*;

pub struct UserPtr(pub Rc<RefCell<User>>);

pub struct User {
    pub ub: UserBase,
    pub ux: UserExt,
}

pub struct UserBase {
    operands: Vec<ptr!(Value)>,
}

pub enum UserExt {
    GlobalVariable(GlobalVariable),
    Instruction(Instruction),
    Constant(Constant),
}
pub struct GlobalVariable {
    is_const: bool,
    init_val: opt_ptr!(Value), // * User: Constant
}

pub enum Constant {
    Int(i32),
    Array(ArrayConstant),
    Zero,
    Float(f32),
}

pub struct ArrayConstant {
    value: Vec<ptr!(Value)>, //* User: Constant
}

pub struct Instruction {
    parent: weak_ptr!(Value), //* Value: BasicBlock
    op_id: OpId,
}

#[derive(PartialEq)]
pub enum OpId {
    IBinary(IBinaryId),
    FBinary(FBinaryId),
    ICmp(ICmpId),
    FCmp(FCmpId),
    Ret,
    Br,
    // Memory operators
    Alloca,
    Load,
    Store,
    // Other operators
    Phi,
    Call,
    GEP,
    Zext,
    Fptosi,
    Sitofp,
}
#[derive(PartialEq)]
pub enum IBinaryId {
    Add,
    Sub,
    Mul,
    Div,
}
#[derive(PartialEq)]
pub enum FBinaryId {
    Add,
    Sub,
    Mul,
    Div,
}
#[derive(PartialEq)]
pub enum ICmpId {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
    Ne,
}
#[derive(PartialEq)]
pub enum FCmpId {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
    Ne,
}

impl UserBase {
    pub fn new() -> Self {
        UserBase {
            operands: Vec::new(),
        }
    }
}

impl GlobalVariable {
    pub fn new(is_const: bool, init_val: opt_ptr!(Value)) -> Self {
        GlobalVariable { is_const, init_val }
    }
}

impl ArrayConstant {
    pub fn new() -> Self {
        ArrayConstant { value: Vec::new() }
    }
    pub fn new_with_vec(val:Vec<Value>) -> Self {
        let mut v = Vec::new();
        for i in val {
            v.push(make_ptr!(i));
        }
        ArrayConstant { value: v }
    }
}

impl Instruction {
    pub fn new(parent: weak_ptr!(Value), op_id: OpId) -> Self {
        Instruction { parent, op_id }
    }
}

impl User {
    pub fn new(ub: UserBase, ux: UserExt) -> Self {
        User { ub, ux }
    }
    pub fn get_operands(&self) -> &Vec<ptr!(Value)> {
        &self.ub.operands
    }
    pub fn get_num_operand(&self) -> usize {
        self.ub.operands.len()
    }
    pub fn get_operand_by_idx(&self, idx: usize) -> opt_ptr!(Value) {
        self.ub.operands.get(idx).cloned()
    }
}

impl UserPtr {
    // * base
    pub fn set_operand_by_idx(&mut self, idx: usize, value: ptr!(Value)) {
        assert!(
            self.0.borrow().ub.operands.len() > idx,
            "set operand index out of vec"
        );
        let user = self.0.clone();
        user.borrow().ub.operands[idx]
            .borrow_mut()
            .remove_use(downgrade!(&user), idx);
        value.borrow_mut().add_use(downgrade!(&user), idx);
        *user.borrow_mut().ub.operands.get_mut(idx).unwrap() = value;
    }
    pub fn add_operand(&mut self, value: ptr!(Value)) {
        let user = self.0.clone();
        value
            .borrow_mut()
            .add_use(downgrade!(&user), user.borrow().get_num_operand());
        user.borrow_mut().ub.operands.push(value);
    }
    pub fn remove_all_operands(&mut self) {
        let user = self.0.clone();
        let n = user.borrow().get_num_operand();
        for arg_no in 0..n {
            user.borrow()
                .ub
                .operands
                .get(arg_no)
                .unwrap()
                .borrow_mut()
                .remove_use(downgrade!(&user), arg_no);
        }
        user.borrow_mut().ub.operands.clear();
    }
    pub fn remove_operand_by_idx(&mut self, idx: usize) {
        assert!(
            self.0.borrow().ub.operands.len() > idx,
            "remove operand index out of vec"
        );
        let user = self.0.clone();
        let n = user.borrow().get_num_operand();
        for i in idx + 1..n {
            user.borrow()
                .ub
                .operands
                .get(i)
                .unwrap()
                .borrow_mut()
                .remove_use(downgrade!(&user), i);
            user.borrow()
                .ub
                .operands
                .get(i)
                .unwrap()
                .borrow_mut()
                .add_use(downgrade!(&user), i - 1);
        }
        user.borrow()
            .ub
            .operands
            .get(idx)
            .unwrap()
            .borrow_mut()
            .remove_use(downgrade!(&user), idx);
        user.borrow_mut().ub.operands.remove(idx);
    }
}

impl UserPtr {
    // * check type
    pub fn is_gv(&self) -> bool {
        matches!(self.0.borrow().ux, UserExt::GlobalVariable(_))
    }
    pub fn is_const(&self) -> bool {
        matches!(self.0.borrow().ux, UserExt::Constant(_))
    }
    pub fn is_inst(&self) -> bool {
        matches!(self.0.borrow().ux, UserExt::Instruction(_))
    }
}
