use crate::{downgrade, ir::{module::{Module, ModulePtr}, user::{Inst, InstPtr, OpID, User}, value::{BasicBlock, BasicBlockPtr, Function, FunctionPtr, ValueClass, ValuePtr}}, make_ptr, ptr, weak_ptr};
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

impl BasicBlockPtr{
    pub fn to_val(&self) -> ValuePtr {
        self.0.borrow().value.clone()
    }

    pub fn clone(&self) -> Self {
        BasicBlockPtr(self.0.clone())
    }

    pub fn eq(&self, other: &BasicBlockPtr) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn create(m:ModulePtr, name: String, parent: FunctionPtr) -> Self {
        let v = ValuePtr::new(m.get_lable_ty(), name);
        let item = BasicBlock {
            value: v.clone(),
            function: downgrade!(&parent.0),
            pre_bbs: vec![],
            succ_bbs: vec![],
            insts: vec![]
        };
        let ptr = make_ptr!(item);
        v.0.borrow_mut().class = Some(ValueClass::BasicBlock(downgrade!(&ptr)));
        let bb = BasicBlockPtr(ptr);
        parent.0.borrow_mut().bbs.push(bb.clone());
        bb
    }

    pub fn add_pre_bbs(&mut self, bb: BasicBlockPtr) {
        self.0.borrow_mut().pre_bbs.push(downgrade!(&bb.0));
    }

    pub fn add_succ_bbs(&mut self, bb: BasicBlockPtr) {
        self.0.borrow_mut().succ_bbs.push(downgrade!(&bb.0));
    }

    pub fn remove_pre_bb(&mut self, bb: &BasicBlockPtr) {
        self.0.borrow_mut().pre_bbs.retain(|a| {
            !bb.eq(&BasicBlockPtr(a.upgrade().unwrap()))
        });
    }

    pub fn remove_succ_bb(&mut self, bb: &BasicBlockPtr) {
        self.0.borrow_mut().succ_bbs.retain(|a| {
            !bb.eq(&BasicBlockPtr(a.upgrade().unwrap()))
        });
    }

    pub fn is_terminated(&self) -> bool {
        if self.0.borrow().insts.is_empty() {
            return false;
        }
        match self.0.borrow().insts.last().unwrap().get_inst_op_id() {
            OpID::Br | OpID::Ret => {
                true
            },
            _ => false
        }
    }

    pub fn get_terminator(&self) -> InstPtr {
        assert!(self.is_terminated(),"[error] block dont have terminator");
        self.0.borrow().insts.last().unwrap().clone()
    }

    pub fn add_inst(&self, inst: InstPtr) {
        assert!(!self.is_terminated(), "Inserting instruction to terminated bb");
        self.0.borrow_mut().insts.push(inst);
    }

    pub fn add_inst_at_begin(&self, inst: InstPtr) {
        self.0.borrow_mut().insts.insert(0, inst);
    }

    pub fn remove_inst(&self, inst: InstPtr) {
        let target = inst.0.as_ptr();
        self.0.borrow_mut().insts.retain(|i| {
            i.0.as_ptr() != target
        });
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().insts.is_empty()
    }

    pub fn get_num_of_inst(&self) -> usize {
        self.0.borrow().insts.len()
    }

    pub fn get_parent(&self) -> FunctionPtr {
        FunctionPtr(self.0.borrow().function.upgrade().unwrap())
    }

    pub fn get_module(&self) -> ModulePtr {
        self.get_parent().get_parent()
    }

    pub fn erase_from_parent(&self) {
        let mut parent = self.get_parent();
        parent.remove(self.clone());
    }
}