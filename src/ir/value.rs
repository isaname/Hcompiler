use crate::create_inst;
use crate::downgrade;
use crate::ir::module::Module;
use crate::ir::module::ModulePtr;
use crate::ir::user;
use crate::make_ptr;
use crate::module_ptr;
use crate::opt_ptr;
use crate::ptr;
use crate::weak_ptr;

use super::type_::Type;
use super::user::*;

use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::env;
use std::ops::Not;
use std::ptr;
use std::rc::{Rc, Weak};

pub struct Use {
    pub(crate) user: weak_ptr!(User),
    pub(crate) arg_no: usize,
}

pub struct Cnt {
    id: usize
}

pub struct Value {
    pub(crate) type_: Rc<Type>,
    pub(crate) name: String,
    pub(crate) use_list: ptr!(Vec<Use>),
    pub(crate) class: Option<ValueClass>,
    pub id: usize
}

pub enum ValueClass {
    User(weak_ptr!(User)),
    Function(weak_ptr!(Function)),
    BasicBlock(weak_ptr!(BasicBlock)),
    Arg(weak_ptr!(Arg)),
}

pub struct ValuePtr(pub ptr!(Value));

pub struct Function {
    pub value: ValuePtr,
    pub module: weak_ptr!(Module),
    pub seq_cnt: usize,
    pub args: Vec<ArgPtr>,
    pub bbs: Vec<BasicBlockPtr>,
}
pub struct FunctionPtr(pub ptr!(Function));

pub struct BasicBlock {
    pub value: ValuePtr,
    pub function: weak_ptr!(Function),
    pub pre_bbs: Vec<weak_ptr!(BasicBlock)>,
    pub succ_bbs: Vec<weak_ptr!(BasicBlock)>,
    pub insts: Vec<InstPtr>,
}
pub struct BasicBlockPtr(pub ptr!(BasicBlock));
pub struct Arg {
    pub value: ValuePtr,
    pub function: weak_ptr!(Function),
    pub arg_no: usize
}
pub struct ArgPtr(pub ptr!(Arg));
