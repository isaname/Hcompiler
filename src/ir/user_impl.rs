use std::cell::RefCell;
use std::{ptr, rc::Rc};

use crate::ir::user::{ConstantPtr, GVPtr, GlobalVariable, InstPtr, UserClass};
use crate::ir::value::ValueClass;
use crate::{downgrade, item_mut};
use crate::{
    ir::{
        type_::Type,
        user::{User, UserPtr},
        value::{Value, ValuePtr},
    },
    item, make_ptr, ptr,
};

impl UserPtr {
    pub fn to_val(&self) -> ValuePtr {
        self.0.borrow().value.clone()
    }

    /// 创建父类，使用父类初始化子类对象（子类能找到父类），修改父类的特定字段（父类能找到子类）
    pub fn new(ty: Rc<Type>, name: String) -> Self {
        let v = ValuePtr::new(ty, name);
        let item = User {
            value: v.clone(),
            operands: make_ptr!(vec![]),
            class: None,
        };
        let ptr = make_ptr!(item);
        v.0.borrow_mut().class = Some(ValueClass::User(downgrade!(&ptr)));
        UserPtr(ptr)
    }

    pub fn clone(&self) -> Self {
        UserPtr(self.0.clone())
    }

    pub fn get_operands(&self) -> ptr!(Vec<ValuePtr>) {
        self.0.borrow().operands.clone()
    }

    pub fn set_operand(&mut self, arg_no: usize, v: ValuePtr) {
        // let this = self;
        item_mut!(self)
            .operands
            .borrow_mut()
            .get_mut(arg_no)
            .unwrap()
            .remove_use(self.clone(), arg_no);
        v.clone().add_use(self.clone(), arg_no);
        if let Some(a) = item_mut!(self).operands.borrow_mut().get_mut(arg_no) {
            *a = v;
        } else {
            panic!("超出数组范围")
        }
    }

    pub fn add_operand(&mut self, v: ValuePtr) {
        v.clone().add_use(self.clone(), self.get_operands().borrow().len());
        self.0.borrow_mut().operands.borrow_mut().push(v);
    }

    pub fn remove_all_operands(&mut self) {
        let mut arg_no = 0;
        for item in self.get_operands().borrow_mut().iter_mut() {
            item.remove_use(self.clone(), arg_no);
            arg_no += 1;
        }
        item!(self).operands.borrow_mut().clear();
    }

    pub fn remove_operand(&mut self, arg_no: usize) {
        let n = self.get_operands().borrow().len();
        assert!(n > arg_no, "超出数组范围");
        for i in arg_no + 1..n {
            self.get_operands()
                .borrow_mut()
                .get_mut(i)
                .unwrap()
                .remove_use(self.clone(), i);
            self.get_operands()
                .borrow_mut()
                .get_mut(i)
                .unwrap()
                .add_use(self.clone(), i - 1);
        }
        self.get_operands()
            .borrow_mut()
            .get_mut(arg_no)
            .unwrap()
            .remove_use(self.clone(), arg_no);
        self.get_operands().borrow_mut().remove(arg_no);
    }
}


/// 类型转换函数
impl UserPtr {
    pub fn to_gv(&self) -> Option<GVPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                UserClass::GlobalVariable(a) => {
                    Some(GVPtr(a.upgrade().unwrap()))
                },
                _ => None,
            }
        }
        None
    }

    pub fn to_inst(&self) -> Option<InstPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                UserClass::Inst(a) => {
                    Some(InstPtr(a.upgrade().unwrap()))
                },
                _ => None,
            }
        }
        None
    }

    pub fn to_const(&self) -> Option<ConstantPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                UserClass::Constant(a) => {
                    Some(ConstantPtr(a.upgrade().unwrap()))
                },
                _ => None,
            }
        }
        None
    }
}