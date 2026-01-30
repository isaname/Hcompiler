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
    /// 创建函数对象（Function）
    ///
    /// # 参数约束
    /// - `name`      : 函数名称（用于调试/打印，通常对应 IR 中的 @funcname）
    /// - `m`         : 所属的 Module（函数会被加入到该模块中）
    /// - `ty`        : 函数类型（必须是函数类型，即 FunctionType）
    ///   - 包含返回类型 + 参数类型列表
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 Function 结构体
    /// - 类型为传入的 `ty`
    pub fn create_func(name: String, m: ptr!(Module), ty: Rc<Type>) -> Self {
        assert!(ty.is_func());
        let func = Function::new(downgrade!(&m));
        Value::new(ValueBase::new(ty, Some(name)), ValueExt::Function(func))
    }
}

impl Value {
    // * basicblock
    /// 创建基本块（BasicBlock）
    ///
    /// # 参数约束
    /// - `name`      : 基本块名称（可选，用于调试/打印，如 %entry, %loop.body 等）
    /// - `m`         : 所属的 Module（用于获取类型系统等全局信息）
    /// - `parent`    : 所属的函数（必须是 Function 类型的 Value）
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 BasicBlock 结构体
    /// - 类型为 label 类型
    pub fn create_bb(name: String, m: ptr!(Module), parent: ptr!(Value)) -> Self {
        assert!(parent.borrow().is_func());
        Value::new(
            ValueBase::new(ModulePtr(m.clone()).get_lable_ty(), Some(name)),
            ValueExt::BasicBlock(BasicBlock::new(downgrade!(&parent), downgrade!(&m))),
        )
    }
    /// 获取当前基本块所属的 Module（Option 形式）
    ///
    /// # 返回值
    /// - Some(ModulePtr)   如果 self 是 BasicBlock 且 weak 引用有效
    /// - None              如果 self 不是 BasicBlock 或引用已失效
    pub fn bb_get_module(&self) -> Option<ptr!(Module)> {
        match &self.vx {
            ValueExt::BasicBlock(bb) => Some(bb.m.clone().upgrade().unwrap()),
            _ => None,
        }
    }
    /// 获取当前基本块所属的 Module（直接 unwrap 版本）
    ///
    /// # Panic
    /// - 当 self 不是 BasicBlock 或 weak 引用已失效时会 panic
    pub fn bb_get_module_ptr(&self) -> ModulePtr {
        ModulePtr(self.bb_get_module().unwrap())
    }
    /// 获取当前基本块所属的父函数（Option 形式）
    ///
    /// # 返回值
    /// - Some(ptr!(Value))    如果 self 是 BasicBlock 且 weak 引用有效
    /// - None              如果 self 不是 BasicBlock 或引用已失效
    pub fn bb_get_parent(&self) -> Option<ptr!(Value)> {
        match &self.vx {
            ValueExt::BasicBlock(bb) => Some(bb.parent.clone().upgrade().unwrap()),
            _ => None,
        }
    }
    /// 添加前驱基本块（predecessor）
    ///
    /// # 参数约束
    /// - `bb`        : 要添加的前驱基本块（必须是 BasicBlock 类型）
    ///
    /// # 副作用
    /// - 在当前基本块的 pre_bbs 链表中添加 weak 引用
    pub fn add_pre_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.pre_bbs.push_back(downgrade!(&bb));
            }
            _ => {}
        }
    }
    /// 添加后继基本块（successor）
    ///
    /// # 参数约束
    /// - `bb`        : 要添加的后继基本块（必须是 BasicBlock 类型）
    ///
    /// # 副作用
    /// - 在当前基本块的 succ_bbs 链表中添加 strong 引用
    pub fn add_succ_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.succ_bbs.push_back(bb);
            }
            _ => {}
        }
    }
    /// 移除指定的前驱基本块
    ///
    /// # 参数约束
    /// - `bb`        : 要移除的前驱基本块（必须是 BasicBlock 类型）
    ///
    /// # 副作用
    /// - 从 pre_bbs 中删除匹配的 weak 引用（通过 Rc::ptr_eq 判断）
    pub fn remove_pre_bbs(&mut self, bb: ptr!(Value)) {
        assert!(bb.borrow().is_bb());
        match &mut self.vx {
            ValueExt::BasicBlock(b) => {
                b.pre_bbs
                    .retain(|a| !Rc::ptr_eq(&bb, &a.upgrade().unwrap()));
            }
            _ => {}
        }
    }
    /// 移除指定的后继基本块
    ///
    /// # 参数约束
    /// - `bb`        : 要移除的后继基本块（必须是 BasicBlock 类型）
    ///
    /// # 副作用
    /// - 从 succ_bbs 中删除匹配的 strong 引用（通过 Rc::ptr_eq 判断）
    pub fn remove_succ_bbs(&mut self, bb: ptr!(Value)) {
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

    /// 创建函数参数（Argument）
    ///
    /// # 参数约束
    /// - `name`      : 参数名称（可选，用于调试/打印，如 %x, %n 等）
    /// - `parent`    : 所属的函数（必须是 Function 类型的 Value）
    /// - `ty`        : 参数的类型
    /// - `arg_no`    : 参数在函数签名中的位置（从 0 开始）
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 Arg 结构体
    /// - 类型为传入的 `ty`
    pub fn create_arg(name: String, parent: ptr!(Value), ty: Rc<Type>, arg_no: usize) -> Self {
        assert!(parent.borrow().is_func());
        Value::new(
            ValueBase::new(ty, Some(name)),
            ValueExt::Arg(Arg::new(downgrade!(&parent), arg_no)),
        )
    }
}

impl Value {
    // * gv
    /// 创建全局变量（Global Variable）
    ///
    /// # 参数约束
    /// - `name`      : 全局变量名称（通常以 @ 开头，如 @global_var）
    /// - `m`         : 所属的 Module（全局变量会被加入到模块的全局变量列表）
    /// - `ty`        : 全局变量的类型（通常是指针类型，指向实际数据类型）
    /// - `is_const`  : 是否为常量（true 表示不可写，常用于常量全局）
    /// - `init_val`  : 初始值（可以为 None，表示 extern 或未初始化）
    ///   - 若提供，则必须是Constant（is_const() == true）
    ///
    /// # 副作用
    /// - 会自动将创建的全局变量加入到 Module 的全局变量列表中
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 GlobalVariable（通过 User 实现）
    /// - 类型为传入的 `ty`（通常为指针类型）
    pub fn create_gv(
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
    /// 创建整数常量
    ///
    /// # 参数约束
    /// - `ty`        : 整数类型（必须是 Int或bool，例如 i32、i1 等）
    /// - `val`       : 整数值（i32，目前不支持更大的位宽直接构造）
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 Constant::Int
    /// - 类型为传入的 `ty`
    pub fn create_const_int(ty: Rc<Type>, val: i32) -> Self {
        assert!(ty.is_int());
        let user = User::new(UserBase::new(), UserExt::Constant(Constant::Int(val)));
        Value::new(
            ValueBase::new(ty, None),
            ValueExt::User(UserPtr(make_ptr!(user))),
        )
    }
    /// 创建值为 0 的整数常量（便捷函数）
    ///
    /// # 参数约束
    /// - `ty`        : 整数类型（必须是 Int或bool，例如 i32、i1 等）
    ///
    /// # 返回值
    /// - 等价于 create_const_int(ty, 0)
    pub fn create_const_zero(ty: Rc<Type>) -> Self {
        assert!(ty.is_int());
        Value::create_const_int(ty, 0)
    }
    /// 创建数组常量
    ///
    /// # 参数约束
    /// - `ty`        : 数组类型（必须是 ArrayType）
    /// - `val`       : 元素值列表
    ///   - 长度必须与数组类型定义的元素个数匹配（当前代码未强制检查）
    ///   - 每个元素必须是常量（is_const() == true）
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 Constant::Array
    /// - 类型为传入的 `ty`
    pub fn create_const_arr(ty: Rc<Type>, val: Vec<Value>) -> Self {
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
    /// 创建浮点常量
    ///
    /// # 参数约束
    /// - `ty`        : 浮点类型
    /// - `val`       : 浮点值（f32，目前不支持 f64 直接构造）
    ///
    /// # 返回值
    /// - 一个 Value，内部封装了 Constant::Float
    /// - 类型为传入的 `ty`
    pub fn create_const_float(ty: Rc<Type>, val: f32) -> Self {
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
    pub fn make_inst_user(op_id: OpId, bb: ptr!(Value)) -> UserPtr {
        assert!(bb.borrow().is_bb());
        let user = User::new(
            UserBase::new(),
            UserExt::Instruction(Instruction::new(downgrade!(&bb), op_id)),
        );
        UserPtr(make_ptr!(user))
    }

    pub fn make_val_from_up(up: UserPtr, ty: Rc<Type>) -> Self {
        Value::new(ValueBase::new(ty, None), ValueExt::User(up))
    }
}

impl Value {
    // * Inst
    /// 创建整数二元运算指令的通用底层函数（不建议直接调用，请使用 create_inst_add / sub / mul / div 等宏展开版本）
    ///
    /// # 参数约束
    /// - `op_id`     : 整数二元运算种类（Add/Sub/Mul/Div）
    /// - `v1`, `v2`  : 两个操作数，必须都是**整数类型**
    /// - `bb`        : 指令要插入到的基本块（必须是 BasicBlock 类型）
    ///
    /// # 返回值
    /// - 结果类型固定为当前 Module 的默认整数类型（通常 i32）
    pub fn create_inst_ibinary(
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
    /// 创建浮点二元运算指令的通用底层函数（不建议直接调用）
    ///
    /// # 参数约束
    /// - `op_id`     : 浮点二元运算种类（Add/Sub/Mul/Div）
    /// - `v1`, `v2`  : 两个操作数，必须都是**浮点类型**，且类型必须相同
    /// - `bb`        : 指令插入的基本块（必须是 BasicBlock）
    ///
    /// # 返回值
    /// - 结果类型固定为当前 Module 的默认浮点类型（通常 float / f32）
    pub fn create_inst_fbinary(
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

    /// 创建整数比较指令的通用底层函数
    ///
    /// # 参数约束
    /// - `op_id`     : 整数比较谓词（eq, ne, gt, ge, lt, le）
    /// - `v1`, `v2`  : 两个整数操作数
    /// - `bb`        : 插入的基本块
    ///
    /// # 返回值
    /// - 固定返回当前 Module 的 bool 类型（i1）
    pub fn create_inst_cmp(
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
    /// 创建浮点比较指令的通用底层函数
    ///
    /// # 参数约束
    /// - `op_id`     : 浮点比较谓词（oeq, one, ogt, oge, olt, ole, ...）
    /// - `v1`, `v2`  : 两个浮点操作数，**类型必须完全相同**
    /// - `bb`        : 插入的基本块
    ///
    /// # 返回值
    /// - 固定返回当前 Module 的 bool 类型（i1）
    pub fn create_inst_fcmp(
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
    /// 创建函数调用指令
    ///
    /// # 参数约束
    /// - `func`      : 被调用的函数（必须是 Function 类型 Value）
    /// - `args`      : 参数列表
    ///   - 长度必须等于函数签名中的参数个数
    ///   - 每个参数的类型必须与函数对应位置的参数类型**完全匹配**
    /// - `bb`        : 调用指令插入的基本块
    ///
    /// # 返回值
    /// - 如果函数有返回值，则返回该类型的值；否则返回 void（但仍构造 Value）
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
    /// 创建无条件跳转指令 `br label %target`
    ///
    /// # 参数约束
    /// - `if_true`   : 跳转目标基本块（必须是 BasicBlock）
    /// - `bb`        : 当前基本块（发出跳转的块）
    ///
    /// # 注意
    /// - 会自动维护前驱/后继基本块关系（predecessor/successor）
    pub fn create_inst_br(if_true: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(if_true.borrow().is_bb());
        assert!(bb.borrow().is_bb());
        let ty = module_ptr!(bb).get_void_ty();
        if_true.borrow_mut().add_pre_bbs(bb.clone());
        bb.borrow_mut().add_succ_bbs(if_true.clone());
        let mut userptr = Self::make_inst_user(OpId::Br, bb);
        userptr.add_operand(if_true);
        Self::make_val_from_up(userptr, ty)
    }
    /// 创建条件跳转指令 `br i1 %cond, label %if_true, label %if_false`
    ///
    /// # 参数约束
    /// - `cond`      : 条件值，必须是 **bool 类型**（i1）
    /// - `if_true`   : 条件为真时跳转的目标基本块
    /// - `if_false`  : 条件为假时跳转的目标基本块
    /// - `bb`        : 当前基本块
    ///
    /// # 注意
    /// - 会自动维护前驱/后继关系（但存在潜在循环引用问题，需后续 GC 或改用 weak ptr 优化）
    pub fn create_inst_cond_br(
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
    /// 创建带返回值指令 `ret %val`
    ///
    /// # 参数约束
    /// - `val`       : 返回值，其类型必须与当前函数的返回类型**完全一致**
    /// - `bb`        : 当前基本块（通常为函数的退出块）
    pub fn create_inst_ret(val: ptr!(Value), bb: ptr!(Value)) -> Self {
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
    /// 创建空返回指令 `ret void`
    ///
    /// # 参数约束
    /// - `bb`        : 当前基本块
    /// - 当前函数的返回类型必须是 void
    pub fn create_inst_void_ret(bb: ptr!(Value)) -> Self {
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
    /// 创建 GetElementPtr 指令（计算指针偏移）
    ///
    /// # 参数约束
    /// - `ptr`       : 基址指针，必须是指针类型
    /// - `idxs`      : 索引序列（可以包含多个索引）
    ///   - 第一个索引通常为 0（数组/结构体首元素）
    ///   - 后续索引按类型逐层展开
    ///   - 每个索引必须是 **整数类型或 bool**
    /// - `bb`        : 插入的基本块
    ///
    /// # 返回值
    /// - 新的指针类型（指向最终元素类型）
    /// - 当前不支持多级指针的复杂情况
    pub fn create_inst_gep(ptr: ptr!(Value), idxs: Vec<ptr!(Value)>, bb: ptr!(Value)) -> Self {
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
    /// 创建 store 指令 `store %val, %ptr`
    ///
    /// # 参数约束
    /// - `val`       : 要写入的值
    /// - `ptr`       : 目标指针，其指向的元素类型必须与 `val` 类型**完全相同**
    /// - `bb`        : 插入的基本块
    pub fn create_inst_store(val: ptr!(Value), ptr: ptr!(Value), bb: ptr!(Value)) -> Self {
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
    /// 创建 load 指令 `%res = load %ptr`
    ///
    /// # 参数约束
    /// - `ptr`       : 要读取的指针，必须是指针类型
    /// - 指针指向的元素类型必须是 int / float / ptr（当前不支持结构体/数组直接 load）
    /// - `bb`        : 插入的基本块
    pub fn create_inst_load(ptr: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        let ty = ptr.borrow().get_type().get_ptr_elem_ty().unwrap();
        assert!(ty.is_bool_or_int() || ty.is_float() || ty.is_ptr());
        let mut userptr = Self::make_inst_user(OpId::Load, bb);
        userptr.add_operand(ptr);
        Self::make_val_from_up(userptr, ty)
    }
    /// 创建栈上分配指令 `alloca %ty`
    ///
    /// # 参数约束
    /// - `ty`        : 要分配的类型（支持 int/float/ptr/array，不支持 function）
    /// - `bb`        : 插入的基本块（通常放在函数入口块）
    ///
    /// # 返回值
    /// - 指向分配空间的指针（类型为 `ptr ty`）
    pub fn create_inst_alloc(ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(ty.is_bool_or_int() || ty.is_float() || ty.is_ptr() || ty.is_arr());
        let mut userptr = Self::make_inst_user(OpId::Alloca, bb.clone());
        Value::new(
            ValueBase::new(module_ptr!(bb).get_ptr_ty(ty), None),
            ValueExt::User(userptr),
        )
    }
    /// 创建零扩展指令（目前仅支持 bool → integer）
    ///
    /// # 参数约束
    /// - `val`       : 输入值，必须是 bool 类型（i1）
    /// - `ty`        : 目标整数类型（通常 i32）
    /// - `bb`        : 插入的基本块
    pub fn create_inst_zext(val: ptr!(Value), ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val.borrow().get_type().is_bool());
        assert!(ty.is_int());
        let mut userptr = Self::make_inst_user(OpId::Zext, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    /// 创建浮点转有符号整数指令 `fptosi`
    ///
    /// # 参数约束
    /// - `val`       : 浮点值
    /// - `ty`        : 目标整数类型（支持 bool/i1 到任意整数）
    /// - `bb`        : 插入的基本块
    pub fn create_inst_fp2si(val: ptr!(Value), ty: Rc<Type>, bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val.borrow().get_type().is_float());
        assert!(ty.is_bool_or_int());
        let mut userptr = Self::make_inst_user(OpId::Fptosi, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    /// 创建有符号整数转浮点指令 `sitofp`
    ///
    /// # 参数约束
    /// - `val`       : 有符号整数或 bool
    /// - `bb`        : 插入的基本块
    pub fn create_inst_si2fp(val: ptr!(Value), bb: ptr!(Value)) -> Self {
        assert!(bb.borrow().is_bb());
        assert!(val.borrow().get_type().is_bool_or_int());
        let ty = bb.borrow().bb_get_module_ptr().get_float_ty();
        let mut userptr = Self::make_inst_user(OpId::Sitofp, bb);
        userptr.add_operand(val);
        Self::make_val_from_up(userptr, ty)
    }
    /// 创建 PHI 节点（φ 函数）
    ///
    /// # 参数约束
    /// - `ty`        : PHI 节点的值类型（所有 incoming 值必须与此类型一致）
    /// - `vals`      : 来自各个前驱的值列表
    /// - `val_bbs`   : 对应前驱基本块列表（长度必须与 vals 相同）
    ///   - 每个元素必须是 BasicBlock 类型
    /// - `bb`        : PHI 节点所在的基本块（通常是多条前驱汇合的块）
    ///
    /// # 注意
    /// - PHI 必须出现在基本块的**第一条指令**
    pub fn create_inst_phi(
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
