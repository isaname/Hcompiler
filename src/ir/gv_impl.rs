use std::cell::RefCell;
use std::rc::Rc;

use crate::{downgrade, ir::{module::ModulePtr, type_::Type, user::{ConstantPtr, GVPtr, GlobalVariable, UserClass, UserPtr}}, make_ptr};

impl GVPtr{
    pub fn new(name: String, m: ModulePtr, ty:Rc<Type>, is_const: bool, init: Option<ConstantPtr>) -> Self {
        let u = UserPtr::new(ty, name);
        let item = GlobalVariable {
            user: u.clone(),
            is_const: is_const,
            init_val: init
        };
        let ptr = make_ptr!(item);
        u.0.borrow_mut().class = Some(UserClass::GlobalVariable(downgrade!(&ptr)));
        GVPtr(ptr)
    }

    pub fn clone(&self) -> Self {
        GVPtr(self.0.clone())
    }

    pub fn create(name: String, m: ModulePtr, ty:Rc<Type>, is_const: bool, init: Option<ConstantPtr>) -> Self {
        Self::new(name, m, ty, is_const, init)
    }
    
    pub fn get_init(&self) -> ConstantPtr {
        match &(self.0.borrow().init_val) {
            Some(a) => {
                a.clone()
            },
            None => {panic!()}
        }
    }

    pub fn is_const(&self) -> bool {
        self.0.borrow().is_const
    }
}