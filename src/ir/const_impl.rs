use std::rc::Rc;
use std::cell::RefCell;

use crate::{downgrade, ir::{type_::Type, user::{Constant, ConstantClass, ConstantPtr, UserClass, UserPtr}}, make_ptr, ptr};

/// 这里和之前的设计方式相同，代码中的类型信息被完全消除
impl ConstantPtr {
    pub fn clone(&self) -> Self {
        ConstantPtr(self.0.clone())
    }

    pub fn new_int(ty: Rc<Type>, name: String, val:i32) -> Self{
        let parent = UserPtr::new(ty, name);
        let item = Constant{
            user: parent.clone(),
            ext: ConstantClass::Int(val)
        };
        let son = make_ptr!(item);
        parent.0.borrow_mut().class = Some(UserClass::Constant(downgrade!(&son)));
        ConstantPtr(son)
    }
    pub fn new_zero(ty: Rc<Type>, name: String) -> Self{
        let parent = UserPtr::new(ty, name);
        let item = Constant{
            user: parent.clone(),
            ext: ConstantClass::Zero
        };
        let son = make_ptr!(item);
        parent.0.borrow_mut().class = Some(UserClass::Constant(downgrade!(&son)));
        ConstantPtr(son)
    }
    pub fn new_float(ty: Rc<Type>, name: String, val:f32) -> Self{
        let parent = UserPtr::new(ty, name);
        let item = Constant{
            user: parent.clone(),
            ext: ConstantClass::Float(val)
        };
        let son = make_ptr!(item);
        parent.0.borrow_mut().class = Some(UserClass::Constant(downgrade!(&son)));
        ConstantPtr(son)
    }
    pub fn new_arr(ty: Rc<Type>, name: String, val:Vec<ConstantPtr>) -> Self{
        let mut parent = UserPtr::new(ty, name);
        for item in &val {
            parent.add_operand(item.0.borrow().user.0.borrow().value.clone());
        }
        let item = Constant{
            user: parent.clone(),
            ext: ConstantClass::Arr(val)
        };
        let son = make_ptr!(item);
        parent.0.borrow_mut().class = Some(UserClass::Constant(downgrade!(&son)));
        ConstantPtr(son)
    }
}