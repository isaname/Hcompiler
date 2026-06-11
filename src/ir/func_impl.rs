use crate::{downgrade, ir::{module::{Module, ModulePtr}, type_::Type, user::{Inst, InstPtr, OpID, User}, value::{Arg, ArgPtr, BasicBlock, BasicBlockPtr, Function, FunctionPtr, Value, ValueClass, ValuePtr}}, make_ptr, ptr, weak_ptr};
use std::{cell::RefCell, collections::HashMap};
use std::rc::Rc;
use std::rc::Weak;

impl ArgPtr {
    pub fn to_val(&self) -> ValuePtr {
        self.0.borrow().value.clone()
    }

    pub fn new(ty: Rc<Type>, name: String, f: FunctionPtr, arg_no: usize) -> ArgPtr {
        let v = ValuePtr::new(ty, name);
        let arg = Arg {
            value: v.clone(),
            function: downgrade!(&f.0),
            arg_no: arg_no
        };
        let ptr = make_ptr!(arg);
        v.0.borrow_mut().class = Some(ValueClass::Arg(downgrade!(&ptr)));
        ArgPtr(ptr)
    }
}



impl FunctionPtr {
    pub fn to_val(&self) -> ValuePtr {
        self.0.borrow().value.clone()
    }

    pub fn clone(&self) -> Self {
        FunctionPtr(self.0.clone())
    }

    pub fn create(ty: Rc<Type>, name: String, mut parent:ModulePtr) -> Self {
        let v = ValuePtr::new(ty.clone(), name);
        
        let func = Function {
            value:v,
            module: downgrade!(&parent.0),
            seq_cnt: 0,
            args: vec![],
            bbs: vec![]
        };
        let ptr = make_ptr!(func);
        let res = FunctionPtr(ptr);
        for i in 0..res.get_num_of_args() {
            let arg = ArgPtr::new(ty.get_func_param_ty(i).unwrap(), String::from(""), res.clone(), i);
            res.0.borrow_mut().args.push(arg);
        }
        parent.add_function(res.clone());
        res
    }

    pub fn get_num_of_args(&self) -> usize {
        self.0.borrow().value.0.borrow().type_.get_func_arg_num().unwrap()
    }

    pub fn get_return_type(&self) -> Rc<Type> {
        self.0.borrow().value.0.borrow().type_.get_func_ret_ty().unwrap()
    }

    pub fn get_num_bbs(&self) -> usize {
        self.0.borrow().bbs.len()
    }

    pub fn get_type(&self) -> Rc<Type> {
        self.0.borrow().value.0.borrow().type_.clone()
    }

    pub fn get_parent(&self) -> ModulePtr {
        ModulePtr(self.0.borrow().module.upgrade().unwrap())
    }

    pub fn remove(&mut self, bb: BasicBlockPtr) {
        self.0.borrow_mut().bbs.retain(|a| {
            !a.eq(&bb)
        });
        for pre in &bb.0.borrow().pre_bbs {
            BasicBlockPtr(pre.upgrade().unwrap()).remove_succ_bb(&bb);
        }
        for succ in &bb.0.borrow().succ_bbs {
            BasicBlockPtr(succ.upgrade().unwrap()).remove_pre_bb(&bb);
        }
    }

    pub fn add_bbs(&mut self, bb: BasicBlockPtr) {
        self.0.borrow_mut().bbs.push(bb);
    }

    /// 为所有的arg， basic block， inst 设置name
    pub fn set_inst_name(&mut self) {
        for arg in &self.0.borrow().args {
            let mut v = arg.to_val();
            let id = v.get_id();
            v.set_name(String::from("arg")+&id.to_string());
        }
        for bb in &self.0.borrow().bbs {
            let mut v = bb.to_val();
            let id = v.get_id();
            v.set_name(String::from("lable")+&id.to_string());
            for inst in &bb.0.borrow().insts {
                let mut v = inst.to_val();
                let id = v.get_id();
                v.set_name(String::from("op")+&id.to_string());
            }
        }
    }
}