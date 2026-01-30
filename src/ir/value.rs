use regex::CaptureNames;

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
use std::ops::Not;
use std::rc::{Rc, Weak};
pub struct Use {
    pub user: weak_ptr!(User),
    pub arg_no: usize,
}

impl Use {
    pub fn new(user: weak_ptr!(User), arg_no: usize) -> Use {
        Use { user, arg_no }
    }
}

pub struct ValuePtr(pub ptr!(Value));

pub enum ValueExt {
    BasicBlock(BasicBlock),
    Function(Function),
    Arg(Arg),
    User(UserPtr),
}

pub struct Value {
    pub vb: ValueBase,
    pub vx: ValueExt,
}

pub struct BasicBlock {
    pre_bbs: LinkedList<weak_ptr!(Value)>, // * BasicBlock
    succ_bbs: LinkedList<ptr!(Value)>,     // * BasicBlock
    insts: LinkedList<ptr!(Value)>,        // * Instruction
    parent: weak_ptr!(Value),              // * Function
    m: weak_ptr!(Module),
}

pub struct Function {
    bbs: LinkedList<ptr!(Value)>,  // * BasicBlock
    args: LinkedList<ptr!(Value)>, // * Arg
    parent: weak_ptr!(Module),
    count: usize,
}

pub struct Arg {
    parent: weak_ptr!(Value), // * Function
    arg_no: usize,
}

pub struct ValueBase {
    type_: Rc<Type>,
    use_list: RefCell<Vec<Use>>,
    name: Option<String>,
}

impl ValueBase {
    pub fn new(ty: Rc<Type>, name: Option<String>) -> Self {
        ValueBase {
            type_: ty,
            use_list: RefCell::new(Vec::new()),
            name,
        }
    }
}

impl BasicBlock {
    pub fn new(parent: Weak<RefCell<Value>>, m: weak_ptr!(Module)) -> Self {
        BasicBlock {
            pre_bbs: LinkedList::new(),
            succ_bbs: LinkedList::new(),
            insts: LinkedList::new(),
            parent,
            m,
        }
    }
}

impl Function {
    pub fn new(parent: Weak<RefCell<Module>>) -> Self {
        Function {
            bbs: LinkedList::new(),
            args: LinkedList::new(),
            parent,
            count: 0,
        }
    }
}

impl Arg {
    pub fn new(parent: Weak<RefCell<Value>>, arg_no: usize) -> Self {
        Arg { parent, arg_no }
    }
}

impl Value {
    // * base method
    pub fn new(vb: ValueBase, vx: ValueExt) -> Self {
        Value { vb, vx }
    }
    pub fn get_name(&self) -> &Option<String> {
        &self.vb.name
    }
    pub fn get_type(&self) -> &Rc<Type> {
        &self.vb.type_
    }
    pub fn set_name(&mut self, name: String) {
        self.vb.name = Some(name);
    }
    pub fn add_use(&mut self, user: weak_ptr!(User), arg_no: usize) {
        self.vb.use_list.borrow_mut().push(Use::new(user, arg_no));
    }
    pub fn remove_use(&mut self, user: weak_ptr!(User), arg_no: usize) {
        self.vb
            .use_list
            .borrow_mut()
            .retain(|a| !(Weak::ptr_eq(&user, &a.user) && arg_no == a.arg_no));
    }
    /// 在调用这个函数记得判断一下self和new_val是不是同一个对象
    pub fn replace_all_use_with(&mut self, new_val: ptr!(Value)) {
        while self.vb.use_list.borrow().is_empty().not() {
            let idx = self.vb.use_list.borrow().first().unwrap().arg_no;
            let user = self
                .vb
                .use_list
                .borrow()
                .first()
                .unwrap()
                .user
                .upgrade()
                .unwrap();
            UserPtr(user).set_operand_by_idx(idx, new_val.clone());
        }
    }
}

impl Value {
    // * check type
    pub fn is_arg(&self) -> bool {
        matches!(self.vx, ValueExt::Arg(_))
    }
    pub fn is_func(&self) -> bool {
        matches!(self.vx, ValueExt::Function(_))
    }
    pub fn is_user(&self) -> bool {
        matches!(self.vx, ValueExt::User(_))
    }
    pub fn is_bb(&self) -> bool {
        matches!(self.vx, ValueExt::BasicBlock(_))
    }
    pub fn is_gv(&self) -> bool {
        match &self.vx {
            ValueExt::User(b) => b.is_gv(),
            _ => false,
        }
    }
    pub fn is_const(&self) -> bool {
        match &self.vx {
            ValueExt::User(b) => b.is_const(),
            _ => false,
        }
    }
    pub fn is_inst(&self) -> bool {
        match &self.vx {
            ValueExt::User(b) => b.is_inst(),
            _ => false,
        }
    }
}

impl Value {
    // * function
    fn create_func(name: String, m: ptr!(Module), ty: Rc<Type>) -> Self {
        assert!(ty.is_func());
        let func = Function::new(downgrade!(&m));
        Value::new(ValueBase::new(ty, Some(name)), ValueExt::Function(func))
    }
}

impl Value {
    // * basicblock
    fn create_bb(name: String, m: ptr!(Module), parent: ptr!(Value)) -> Self {
        assert!(parent.borrow().is_func());
        Value::new(
            ValueBase::new(ModulePtr(m.clone()).get_lable_ty(), Some(name)),
            ValueExt::BasicBlock(BasicBlock::new(downgrade!(&parent), downgrade!(&m))),
        )
    }
    fn bb_get_module(&self) -> Option<ptr!(Module)> {
        match &self.vx {
            ValueExt::BasicBlock(bb) => Some(bb.m.clone().upgrade().unwrap()),
            _ => None,
        }
    }

    fn bb_get_module_ptr(&self) -> ModulePtr {
        ModulePtr(self.bb_get_module().unwrap())
    }

    fn bb_get_parent(&self) -> Option<ptr!(Value)> {
        match &self.vx {
            ValueExt::BasicBlock(bb) => Some(bb.parent.clone().upgrade().unwrap()),
            _ => None,
        }
    }
    fn add_pre_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.pre_bbs.push_back(downgrade!(&bb));
            }
            _ => {}
        }
    }
    fn add_succ_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.succ_bbs.push_back(bb);
            }
            _ => {}
        }
    }
    fn remove_pre_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.pre_bbs
                    .retain(|a| !Rc::ptr_eq(&bb, &a.upgrade().unwrap()));
            }
            _ => {}
        }
    }
    fn remove_succ_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.succ_bbs.retain(|a| !Rc::ptr_eq(&bb, a));
            }
            _ => {}
        }
    }
}

impl Value {
    // * arg
    fn create_arg(name: String, parent: ptr!(Value), ty: Rc<Type>, arg_no: usize) -> Self {
        assert!(parent.borrow().is_func());
        Value::new(
            ValueBase::new(ty, Some(name)),
            ValueExt::Arg(Arg::new(downgrade!(&parent), arg_no)),
        )
    }
}

impl Value {
    // * gv
    fn create_gv(
        name: String,
        m: ptr!(Module),
        ty: Rc<Type>,
        is_const: bool,
        init_val: opt_ptr!(Value),
    ) -> Self {
        let obj = User::new(
            UserBase::new(),
            UserExt::GlobalVariable(GlobalVariable::new(is_const, init_val.clone())),
        );
        let gv = make_ptr!(obj);
        ModulePtr(m).add_gv(gv.clone());
        let mut res = UserPtr(gv);
        if let Some(ptr) = init_val {
            assert!(ptr.borrow().is_const());
            res.add_operand(ptr);
        }
        Value::new(ValueBase::new(ty, Some(name)), ValueExt::User(res))
    }
}

impl Value {
    // * Constant
    fn create_const_int(ty: Rc<Type>, val: i32) -> Self {
        assert!(ty.is_int());
        let user = User::new(UserBase::new(), UserExt::Constant(Constant::Int(val)));
        Value::new(
            ValueBase::new(ty, None),
            ValueExt::User(UserPtr(make_ptr!(user))),
        )
    }

    fn create_const_zero(ty: Rc<Type>) -> Self {
        assert!(ty.is_int());
        Value::create_const_int(ty, 0)
    }

    fn create_const_arr(ty: Rc<Type>, val: Vec<Value>) -> Self {
        assert!(ty.is_arr());
        for i in &val {
            assert!(i.is_const());
        }
        let user = User::new(
            UserBase::new(),
            UserExt::Constant(Constant::Array(ArrayConstant::new_with_vec(val))),
        );
        Value::new(
            ValueBase::new(ty, None),
            ValueExt::User(UserPtr(make_ptr!(user))),
        )
    }

    fn create_const_float(ty: Rc<Type>, val: f32) -> Self {
        assert!(ty.is_float());
        let user = User::new(UserBase::new(), UserExt::Constant(Constant::Float(val)));
        Value::new(
            ValueBase::new(ty, None),
            ValueExt::User(UserPtr(make_ptr!(user))),
        )
    }
}

impl Value {
    // * helper function
    fn make_inst_user(op_id: OpId, bb: ptr!(Value)) -> UserPtr {
        assert!(bb.borrow().is_bb());
        let user = User::new(
            UserBase::new(),
            UserExt::Instruction(Instruction::new(downgrade!(&bb), op_id)),
        );
        UserPtr(make_ptr!(user))
    }

    fn make_val_from_up(up: UserPtr, ty: Rc<Type>) -> Self {
        Value::new(ValueBase::new(ty, None), ValueExt::User(up))
    }
}

impl Value {
    // * Inst
    
    fn create_inst_ibinary(
        op_id: IBinaryId,
        v1: ptr!(Value),
        v2: ptr!(Value),
        bb: ptr!(Value),
    ) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(v1.borrow().get_type().is_int());
        assert!(v2.borrow().get_type().is_int());
        let ty = bb.borrow().bb_get_module_ptr().get_int_ty();
        let mut userptr = Self::make_inst_user(OpId::IBinary(op_id), bb);
        userptr.add_operand(v1);
        userptr.add_operand(v2);
        Self::make_val_from_up(userptr, ty)
    }

    create_inst!(create_inst_add, IBinaryId::Add, create_inst_ibinary);
    create_inst!(create_inst_sub, IBinaryId::Sub, create_inst_ibinary);
    create_inst!(create_inst_mul, IBinaryId::Mul, create_inst_ibinary);
    create_inst!(create_inst_div, IBinaryId::Div, create_inst_ibinary);

    fn create_inst_fbinary(
        op_id: FBinaryId,
        v1: ptr!(Value),
        v2: ptr!(Value),
        bb: ptr!(Value),
    ) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(v1.borrow().get_type().is_float());
        assert!(v2.borrow().get_type().is_float());
        let ty = bb.borrow().bb_get_module_ptr().get_float_ty();
        let mut userptr = Self::make_inst_user(OpId::FBinary(op_id), bb);
        userptr.add_operand(v1);
        userptr.add_operand(v2);
        Self::make_val_from_up(userptr, ty)
    }

    create_inst!(create_inst_fadd, FBinaryId::Add, create_inst_fbinary);
    create_inst!(create_inst_fsub, FBinaryId::Sub, create_inst_fbinary);
    create_inst!(create_inst_fmul, FBinaryId::Mul, create_inst_fbinary);
    create_inst!(create_inst_fdiv, FBinaryId::Div, create_inst_fbinary);

    fn create_inst_cmp(
        op_id: ICmpId,
        v1: ptr!(Value),
        v2: ptr!(Value),
        bb: ptr!(Value),
    ) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(v1.borrow().vb.type_.is_int() && v2.borrow().vb.type_.is_int());
        let ty = bb.borrow().bb_get_module_ptr().get_bool_ty();
        let mut userptr = Self::make_inst_user(OpId::ICmp(op_id), bb);
        userptr.add_operand(v1);
        userptr.add_operand(v2);
        Self::make_val_from_up(userptr, ty)
    }

    create_inst!(create_inst_ge, ICmpId::Ge, create_inst_cmp);
    create_inst!(create_inst_gt, ICmpId::Gt, create_inst_cmp);
    create_inst!(create_inst_le, ICmpId::Le, create_inst_cmp);
    create_inst!(create_inst_lt, ICmpId::Lt, create_inst_cmp);
    create_inst!(create_inst_eq, ICmpId::Eq, create_inst_cmp);
    create_inst!(create_inst_ne, ICmpId::Ne, create_inst_cmp);

    fn create_inst_fcmp(
        op_id: FCmpId,
        v1: ptr!(Value),
        v2: ptr!(Value),
        bb: ptr!(Value),
    ) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(v1.borrow().get_type().is_float() && v2.borrow().get_type().is_float());
        let ty = bb.borrow().bb_get_module_ptr().get_bool_ty();
        let mut userptr = Self::make_inst_user(OpId::FCmp(op_id), bb);
        userptr.add_operand(v1);
        userptr.add_operand(v2);
        Self::make_val_from_up(userptr, ty)
    }

    create_inst!(create_inst_fge, FCmpId::Ge, create_inst_fcmp);
    create_inst!(create_inst_fgt, FCmpId::Gt, create_inst_fcmp);
    create_inst!(create_inst_fle, FCmpId::Le, create_inst_fcmp);
    create_inst!(create_inst_flt, FCmpId::Lt, create_inst_fcmp);
    create_inst!(create_inst_feq, FCmpId::Eq, create_inst_fcmp);
    create_inst!(create_inst_fne, FCmpId::Ne, create_inst_fcmp);

    pub fn create_inst_call(func: ptr!(Value), args: Vec<ptr!(Value)>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(func.borrow().is_func());
        assert!(func.borrow().get_type().get_func_arg_num().unwrap() == args.len());
        let ty = func.borrow().get_type().get_func_ret_ty().unwrap().clone();

        let mut userptr = Self::make_inst_user(OpId::Call, bb);
        for (idx, i) in args.into_iter().enumerate() {
            assert!(Rc::ptr_eq(
                i.borrow().get_type(),
                &func.borrow().get_type().get_func_param_ty(idx).unwrap()
            ));
            userptr.add_operand(i);
        }
        Self::make_val_from_up(userptr, ty)
    }

    fn create_inst_br(if_true: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(if_true.borrow().is_bb());
        assert!(bb.borrow().is_bb());
        let ty = module_ptr!(bb).get_void_ty();
        if_true.borrow_mut().add_pre_bbs(bb.clone());
        bb.borrow_mut().add_succ_bbs(if_true.clone());
        let mut userptr = Self::make_inst_user(OpId::Br, bb);
        userptr.add_operand(if_true);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_cond_br(
        cond: ptr!(Value),
        if_true: ptr!(Value),
        if_false: ptr!(Value),
        bb: ptr!(Value),
    ) -> Self {
        assert!(if_true.borrow().is_bb());
        assert!(if_false.borrow().is_bb());
        assert!(bb.borrow().is_bb());
        assert!(cond.borrow().get_type().is_bool());
        let ty = module_ptr!(bb).get_void_ty();
        // TODO 这里可能产生了循环引用，会发生内存泄漏
        if_true.borrow_mut().add_pre_bbs(bb.clone());
        if_false.borrow_mut().add_pre_bbs(bb.clone());
        bb.borrow_mut().add_succ_bbs(if_true.clone());
        bb.borrow_mut().add_succ_bbs(if_false.clone());
        let mut userptr = Self::make_inst_user(OpId::Br, bb);
        userptr.add_operand(cond);
        userptr.add_operand(if_true);
        userptr.add_operand(if_false);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_ret(val: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(Rc::ptr_eq(
            &bb.borrow()
                .bb_get_parent()
                .unwrap()
                .borrow()
                .get_type()
                .get_func_ret_ty()
                .unwrap(),
            &val.borrow().get_type()
        ));
        let ty = module_ptr!(bb).get_void_ty();
        let mut userptr = Self::make_inst_user(OpId::Ret, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_void_ret(bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(bb
            .borrow()
            .bb_get_parent()
            .unwrap()
            .borrow()
            .get_type()
            .get_func_ret_ty()
            .unwrap()
            .is_void());
        let ty = module_ptr!(bb).get_void_ty();
        let mut userptr = Self::make_inst_user(OpId::Ret, bb);
        Self::make_val_from_up(userptr, ty)
    }

    fn create_inst_gep(ptr: ptr!(Value), idxs: Vec<ptr!(Value)>, bb: ptr!(Value)) -> Self {
        // 从ptr中获取具体类型
        assert!(ptr.borrow().get_type().is_ptr());
        let mut ty = ptr.borrow().get_type().get_ptr_elem_ty().unwrap();
        assert!(ty.is_arr() || ty.is_float() || ty.is_bool_or_int()); // * 注意：没有多级指针这种情况
        let n = idxs.len();
        if ty.is_arr() {
            for i in 0..n {
                ty = ty.get_arr_elem_ty().unwrap();
                if i != n - 1 {
                    assert!(ty.is_arr());
                }
            }
        }
        let ty = module_ptr!(bb).get_ptr_ty(ty);
        let mut userptr = Self::make_inst_user(OpId::GEP, bb);
        for i in idxs {
            assert!(i.borrow().get_type().is_bool_or_int()); // * bool值也可以作为指针
            userptr.add_operand(i);
        }
        Self::make_val_from_up(userptr, ty)
    }

    fn create_inst_store(val: ptr!(Value), ptr: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(Rc::ptr_eq(
            &ptr.borrow().get_type().get_ptr_elem_ty().unwrap(),
            &val.borrow().get_type()
        ));
        let ty = module_ptr!(bb).get_void_ty();
        let mut userptr = Self::make_inst_user(OpId::Store, bb);
        userptr.add_operand(val);
        userptr.add_operand(ptr);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_load(ptr: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        let ty = ptr.borrow().get_type().get_ptr_elem_ty().unwrap();
        assert!(ty.is_bool_or_int() || ty.is_float() || ty.is_ptr());
        let mut userptr = Self::make_inst_user(OpId::Load, bb);
        userptr.add_operand(ptr);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_alloc(ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(ty.is_bool_or_int() || ty.is_float() || ty.is_ptr() || ty.is_arr());
        let mut userptr = Self::make_inst_user(OpId::Alloca, bb.clone());
        Value::new(
            ValueBase::new(module_ptr!(bb).get_ptr_ty(ty), None),
            ValueExt::User(userptr),
        )
    }
    /// 只支持从bool扩展到int
    fn create_inst_zext(val: ptr!(Value), ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val.borrow().get_type().is_bool());
        assert!(ty.is_int());
        let mut userptr = Self::make_inst_user(OpId::Zext, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_fp2si(val: ptr!(Value), ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val.borrow().get_type().is_float());
        assert!(ty.is_bool_or_int());
        let mut userptr = Self::make_inst_user(OpId::Fptosi, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_si2fp(val: ptr!(Value), ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val.borrow().get_type().is_bool_or_int());
        assert!(ty.is_float());
        let mut userptr = Self::make_inst_user(OpId::Sitofp, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    fn create_inst_phi(
        ty: Rc<Type>,
        vals: Vec<ptr!(Value)>,
        val_bbs: Vec<ptr!(Value)>,
        bb: ptr!(Value),
    ) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val_bbs.len() == vals.len());
        let mut userptr = Self::make_inst_user(OpId::Phi, bb);
        for i in 0..val_bbs.len() {
            assert!(Rc::ptr_eq(&ty, &vals[i].borrow().get_type()));
            assert!(val_bbs[i].borrow().is_bb());
            userptr.add_operand(vals[i].clone());
            userptr.add_operand(val_bbs[i].clone());
        }
        Self::make_val_from_up(userptr, ty)
    }
}
