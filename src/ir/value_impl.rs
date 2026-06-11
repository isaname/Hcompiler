use std::{cell::RefCell, rc::Rc, slice::SliceIndex, vec};

use crate::{downgrade, ir::{type_::Type, user::{ConstantPtr, GVPtr, InstPtr, User, UserClass, UserPtr}, value::{ArgPtr, BasicBlockPtr, Cnt, FunctionPtr, Use, Value, ValueClass, ValuePtr}}, item, item_mut, make_ptr, ptr};
use std::sync::{LazyLock, Mutex};

static ID: LazyLock<Mutex<usize>> = LazyLock::new(|| { Mutex::new(0) });


impl ValuePtr {
    pub fn clone(&self) -> Self {
        ValuePtr(self.0.clone())
    }

    pub fn new(ty: Rc<Type>, name: String) -> Self {
        let id;
        {
            let mut a = ID.lock().unwrap();
            *a+=1;
            id = *a;
        }
        let item = Value{
            type_: ty,
            name: name,
            use_list: make_ptr!(vec![]),
            class: None,
            id: id
        };
        ValuePtr(make_ptr!(
            item
        ))
    }

    pub fn get_id(&self) -> usize {
        self.0.borrow().id
    }

    pub fn get_name(&self) -> String {
        item!(self).name.clone()
    }

    pub fn get_type(&self) -> Rc<Type> {
        item!(self).type_.clone()
    }

    pub fn get_use_list(&self) -> ptr!(Vec<Use>) {
        item!(self).use_list.clone()
    }

    pub fn set_name(&mut self, name: String) -> bool {
        if item!(self).name.len()==0 {
            item_mut!(self).name = name;
            return true;
        }
        false
    }

    pub fn add_use(&mut self, user: UserPtr, arg_no: usize) {
        let item = Use {
            user: downgrade!(&user.0),
            arg_no: arg_no
        };
        item_mut!(self).use_list.borrow_mut().push(item);
    }

    pub fn remove_use(&mut self, user: UserPtr, arg_no: usize) {
        item_mut!(self).use_list.borrow_mut().retain(|a| {
            a.arg_no != arg_no || !a.user.ptr_eq(&downgrade!(&user.0))
        });
    }

    pub fn replace_all_use_with(&mut self, new_val: ValuePtr) {
        if Rc::ptr_eq(&self.0, &new_val.0) {
            return;
        }
        while !item!(self).use_list.borrow().is_empty() {
            let arg_no = item!(self).use_list.borrow().first().unwrap().arg_no;
            UserPtr(item!(self).use_list.borrow_mut().first_mut().unwrap().user.upgrade().unwrap()).set_operand(arg_no, new_val.clone());
        }
    }
}

/// 类型转换函数
impl ValuePtr {
    pub fn to_gv(&self) -> Option<GVPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                ValueClass::User(a) => {
                    UserPtr(a.upgrade().unwrap()).to_gv()
                },
                _ => None,
            }
        }
        None
    }

    pub fn to_inst(&self) -> Option<InstPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                ValueClass::User(a) => {
                    UserPtr(a.upgrade().unwrap()).to_inst()
                },
                _ => None,
            }
        }
        None
    }

    pub fn to_const(&self) -> Option<ConstantPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                ValueClass::User(a) => {
                    UserPtr(a.upgrade().unwrap()).to_const()
                },
                _ => None,
            }
        }
        None
    }
    
    pub fn to_arg(&self) -> Option<ArgPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                ValueClass::Arg(a) => {
                    Some(ArgPtr(a.upgrade().unwrap()))
                },
                _ => None,
            }
        }
        None
    }

    pub fn to_bb(&self) -> Option<BasicBlockPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                ValueClass::BasicBlock(a) => {
                    Some(BasicBlockPtr(a.upgrade().unwrap()))
                },
                _ => None,
            }
        }
        None
    }

    pub fn to_function(&self) -> Option<FunctionPtr> {
        if let Some(temp)  = &self.0.borrow().class {
            return match temp {
                ValueClass::Function(a) => {
                    Some(FunctionPtr(a.upgrade().unwrap()))
                },
                _ => None,
            }
        }
        None
    }
}