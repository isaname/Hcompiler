use crate::{downgrade, ir::{module::{Module, ModulePtr}, type_::Type, user::{Inst, InstPtr, OpID, User, UserClass, UserPtr}, value::{BasicBlock, BasicBlockPtr, Function, FunctionPtr, ValueClass, ValuePtr}}, make_ptr, ptr, weak_ptr};
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
impl InstPtr {
    pub fn to_val(&self) -> ValuePtr {
        self.0.borrow().user.to_val()
    }

    pub fn to_user(&self) -> UserPtr {
        self.0.borrow().user.clone()
    }

    pub fn clone(&self) -> Self {
        InstPtr(self.0.clone())
    }

    pub fn get_inst_op_id(&self) -> OpID {
        self.0.borrow().op_id
    }

    pub fn new(ty: Rc<Type>, id: OpID, parent: BasicBlockPtr) -> Self {
        let user = UserPtr::new(ty, String::from(""));
        let item = Inst {
            user: user.clone(),
            op_id: id,
            bb: downgrade!(&parent.0)
        };
        let ptr = make_ptr!(item);
        user.0.borrow_mut().class = Some(UserClass::Inst(downgrade!(&ptr)));
        InstPtr(ptr)
    }

    pub fn create(ty: Rc<Type>, id: OpID, parent: BasicBlockPtr, v:Vec<ValuePtr>) -> Self {
        let mut base = Self::new(ty, id, parent);
        for i in v {
            base.to_user().add_operand(i);
        }
        base
    }

    pub fn get_func(&self) -> FunctionPtr {
        BasicBlockPtr(self.0.borrow().bb.upgrade().unwrap()).get_parent()
    }

    pub fn get_module(&self) -> ModulePtr {
        BasicBlockPtr(self.0.borrow().bb.upgrade().unwrap()).get_module()
    }

    pub fn ibinary_inst(id: OpID, v1:ValuePtr, v2:ValuePtr, bb: BasicBlockPtr) -> Self {
        assert!(v1.get_type().is_int()&&v2.get_type().is_int());
        assert!(matches!(id, OpID::Add|OpID::Sub|OpID::Mul|OpID::SDiv));
        let v = vec![v1, v2];
        Self::create(bb.get_module().get_int_ty(), id, bb,v)
    }

    pub fn fbinary_inst(id: OpID, v1:ValuePtr, v2:ValuePtr, bb: BasicBlockPtr) -> Self {
        assert!(v1.get_type().is_float()&&v2.get_type().is_float());
        assert!(matches!(id, OpID::FAdd|OpID::FSub|OpID::FMul|OpID::FDiv));
        let v = vec![v1, v2];
        Self::create(bb.get_module().get_int_ty(), id, bb,v)
    }

    pub fn icmp_inst(id: OpID, lhs:ValuePtr, rhs:ValuePtr, bb:BasicBlockPtr) -> Self {
        assert!(lhs.get_type().is_int()&&rhs.get_type().is_int());
        assert!(matches!(id, OpID::Ge|OpID::Gt|OpID::Le|OpID::Lt|OpID::Eq|OpID::Ne));
        let v = vec![lhs, rhs];
        Self::create(bb.get_module().get_int_ty(), id, bb,v)
    }

    pub fn fcmp_inst(id: OpID, lhs:ValuePtr, rhs:ValuePtr, bb:BasicBlockPtr) -> Self {
        assert!(lhs.get_type().is_float()&&rhs.get_type().is_float());
        assert!(matches!(id, OpID::FGe|OpID::FGt|OpID::FLe|OpID::FLt|OpID::FEq|OpID::FNe));
        let v = vec![lhs, rhs];
        Self::create(bb.get_module().get_int_ty(), id, bb,v)
    }

    pub fn call_inst(func: FunctionPtr, args: Vec<ValuePtr>, bb: BasicBlockPtr) -> Self {
        let inst = Self::create(func.get_return_type(), OpID::Call, bb, vec![]);
        assert!(func.get_num_of_args()==args.len());
        inst.to_user().add_operand(func.to_val());
        let ty = func.get_type();
        let mut idx = 0;
        for i in args {
            assert!(Rc::ptr_eq(&i.get_type(),&ty.get_func_param_ty(idx).unwrap()));
            idx += 1;
            inst.to_user().add_operand(i);
        }
        inst
    }
}