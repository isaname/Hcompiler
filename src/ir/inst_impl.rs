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
        let mut base = Self::new(ty, id, parent.clone());
        for i in v {
            base.to_user().add_operand(i);
        }
        parent.add_inst(base.clone()); // 这个动作差点忘了
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
        assert!((lhs.get_type().is_int() || lhs.get_type().is_bool()) && (rhs.get_type().is_int() || rhs.get_type().is_bool()));
        assert!(matches!(id, OpID::Ge|OpID::Gt|OpID::Le|OpID::Lt|OpID::Eq|OpID::Ne));
        let v = vec![lhs, rhs];
        Self::create(bb.get_module().get_bool_ty(), id, bb,v)
    }

    pub fn fcmp_inst(id: OpID, lhs:ValuePtr, rhs:ValuePtr, bb:BasicBlockPtr) -> Self {
        assert!(lhs.get_type().is_float()&&rhs.get_type().is_float());
        assert!(matches!(id, OpID::FGe|OpID::FGt|OpID::FLe|OpID::FLt|OpID::FEq|OpID::FNe));
        let v = vec![lhs, rhs];
        Self::create(bb.get_module().get_bool_ty(), id, bb,v)
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

    /// 条件分支指令
    pub fn cond_br_inst(cond: ValuePtr, if_true: BasicBlockPtr, if_false: BasicBlockPtr, bb: BasicBlockPtr) -> Self {
        assert!(cond.get_type().is_bool(), "BranchInst condition is not bool");
        let inst = Self::create(bb.get_module().get_void_ty(), OpID::Br, bb.clone(), vec![]);
        inst.to_user().add_operand(cond);
        inst.to_user().add_operand(if_true.to_val());
        inst.to_user().add_operand(if_false.to_val());
        // prev/succ
        if_true.clone().add_pre_bbs(bb.clone());
        if_false.clone().add_pre_bbs(bb.clone());
        bb.clone().add_succ_bbs(if_true);
        bb.clone().add_succ_bbs(if_false);
        inst
    }

    /// 无条件跳转指令
    pub fn br_inst(if_true: BasicBlockPtr, bb: BasicBlockPtr) -> Self {
        let inst = Self::create(bb.get_module().get_void_ty(), OpID::Br, bb.clone(), vec![]);
        inst.to_user().add_operand(if_true.to_val());
        // prev/succ
        if_true.clone().add_pre_bbs(bb.clone());
        bb.clone().add_succ_bbs(if_true);
        inst
    }

    /// 是否是条件分支
    pub fn is_cond_br(&self) -> bool {
        self.get_inst_op_id() as u8 == OpID::Br as u8
            && self.to_user().get_operands().borrow().len() == 3
    }

    /// 返回指令（带返回值）
    pub fn ret_inst(val: ValuePtr, bb: BasicBlockPtr) -> Self {
        let func = bb.get_parent();
        assert!(!func.get_return_type().is_void(), "Void function returning a value");
        assert!(Rc::ptr_eq(&func.get_return_type(), &val.get_type()),
            "ReturnInst type is different from function return type");
        Self::create(bb.get_module().get_void_ty(), OpID::Ret, bb, vec![val])
    }

    /// 返回指令（void）
    pub fn void_ret_inst(bb: BasicBlockPtr) -> Self {
        let func = bb.get_parent();
        assert!(func.get_return_type().is_void(), "Non-void function missing return value");
        Self::create(bb.get_module().get_void_ty(), OpID::Ret, bb, vec![])
    }

    /// 是否是 void return
    pub fn is_void_ret(&self) -> bool {
        self.to_user().get_operands().borrow().len() == 0
    }

    /// GEP 指令的元素类型推导
    fn get_element_type(ptr: &ValuePtr, idxs: &[ValuePtr]) -> Rc<Type> {
        assert!(ptr.get_type().is_ptr(), "GetElementPtrInst ptr is not a pointer");
        let mut ty = ptr.get_type().get_ptr_elem_ty().unwrap();
        assert!(ty.is_arr() || ty.is_int() || ty.is_float(),
            "GetElementPtrInst ptr is wrong type");
        if ty.is_arr() {
            for i in 1..idxs.len() {
                ty = ty.get_arr_elem_ty().unwrap();
                if i < idxs.len() - 1 {
                    assert!(ty.is_arr(), "Index error!");
                }
            }
        }
        ty
    }

    /// GEP 指令
    pub fn gep_inst(ptr: ValuePtr, idxs: Vec<ValuePtr>, bb: BasicBlockPtr) -> Self {
        let elem_ty = Self::get_element_type(&ptr, &idxs);
        let result_ty = bb.get_module().get_ptr_ty(elem_ty);
        let inst = Self::create(result_ty, OpID::GetElementPtr, bb, vec![]);
        inst.to_user().add_operand(ptr);
        for idx in idxs {
            assert!(idx.get_type().is_int() || idx.get_type().is_bool_or_int(),
                "Index is not integer");
            inst.to_user().add_operand(idx);
        }
        inst
    }

    /// Store 指令
    pub fn store_inst(val: ValuePtr, ptr: ValuePtr, bb: BasicBlockPtr) -> Self {
        assert!(Rc::ptr_eq(
            &ptr.get_type().get_ptr_elem_ty().unwrap(),
            &val.get_type()
        ), "StoreInst ptr is not a pointer to val type");
        Self::create(bb.get_module().get_void_ty(), OpID::Store, bb, vec![val, ptr])
    }

    /// Load 指令
    pub fn load_inst(ptr: ValuePtr, bb: BasicBlockPtr) -> Self {
        let elem_ty = ptr.get_type().get_ptr_elem_ty().unwrap();
        assert!(elem_ty.is_int() || elem_ty.is_float() || elem_ty.is_ptr(),
            "Should not load value with type except int/float/pointer");
        Self::create(elem_ty, OpID::Load, bb, vec![ptr])
    }

    /// Alloca 指令
    pub fn alloca_inst(ty: Rc<Type>, bb: BasicBlockPtr) -> Self {
        assert!(ty.is_int() || ty.is_float() || ty.is_arr() || ty.is_ptr(),
            "Not allowed type for alloca");
        let ptr_ty = bb.get_module().get_ptr_ty(ty);
        Self::create(ptr_ty, OpID::Alloca, bb, vec![])
    }

    /// ZExt 指令（零扩展，bool -> int）
    pub fn zext_inst(val: ValuePtr, ty: Rc<Type>, bb: BasicBlockPtr) -> Self {
        assert!(val.get_type().is_bool_or_int(), "ZextInst operand is not integer");
        assert!(ty.is_bool_or_int(), "ZextInst destination type is not integer");
        Self::create(ty, OpID::ZExt, bb, vec![val])
    }

    /// ZExt to i32 便捷函数
    pub fn zext_to_i32_inst(val: ValuePtr, bb: BasicBlockPtr) -> Self {
        Self::zext_inst(val, bb.get_module().get_int_ty(), bb)
    }

    /// FpToSi 指令（float -> int）
    pub fn fptosi_inst(val: ValuePtr, ty: Rc<Type>, bb: BasicBlockPtr) -> Self {
        assert!(val.get_type().is_float(), "FpToSiInst operand is not float");
        assert!(ty.is_bool_or_int(), "FpToSiInst destination type is not integer");
        Self::create(ty, OpID::FPToSI, bb, vec![val])
    }

    /// FpToSi to i32 便捷函数
    pub fn fptosi_to_i32_inst(val: ValuePtr, bb: BasicBlockPtr) -> Self {
        Self::fptosi_inst(val, bb.get_module().get_int_ty(), bb)
    }

    /// SiToFp 指令（int -> float）
    pub fn sitofp_inst(val: ValuePtr, bb: BasicBlockPtr) -> Self {
        assert!(val.get_type().is_bool_or_int(), "SiToFpInst operand is not integer");
        let float_ty = bb.get_module().get_float_ty();
        Self::create(float_ty, OpID::SIToFP, bb, vec![val])
    }

    /// Phi 指令
    pub fn phi_inst(ty: Rc<Type>, vals: Vec<ValuePtr>, val_bbs: Vec<BasicBlockPtr>, bb: BasicBlockPtr) -> Self {
        assert!(vals.len() == val_bbs.len(), "Unmatched vals and bbs");
        let inst = Self::new(ty.clone(), OpID::Phi, bb.clone());
        for i in 0..vals.len() {
            assert!(Rc::ptr_eq(&ty, &vals[i].get_type()), "Bad type for phi");
            inst.to_user().add_operand(vals[i].clone());
            inst.to_user().add_operand(val_bbs[i].to_val());
        }
        bb.add_inst_at_begin(inst.clone());
        inst
    }
}