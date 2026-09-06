use crate::ast::{
    AddExp, AddOp, Block, BlockItem, Btype, CompUnit, Cond, ConstDecl, ConstDef,
    ConstExp, ConstInitVal, Decl, EqExp, EqOp, Exp, FuncDef, FuncFParam, FuncType,
    GlobalItem, InitVal, LAndExp, LOrExp, LVal, MulExp, MulOp, Number, PrimaryExp,
    RelExp, RelOp, Stmt, UnaryExp, UnaryOp, VarDecl, VarDef,
};
use crate::ir::inst_builder::InstBuilder;
use crate::ir::module::{Module, ModulePtr};
use crate::ir::type_::Type;
use crate::ir::user::{ConstantPtr, GVPtr, InstPtr, OpID};
use crate::ir::value::{BasicBlockPtr, FunctionPtr, ValuePtr};
use crate::{ir::value::Value, irgen::scope::Scope, make_ptr, opt_ptr, ptr};
use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Not, Rem, Sub};
use std::rc::Rc;

struct Context {
    is_const: bool,
    curr_func: Option<FunctionPtr>,
    func_param_idx: usize,
}

pub struct IRGenerator {
    context: Context,
    scope: Scope,
    buidler: InstBuilder,
    m: ModulePtr,
}

impl Context {
    pub fn new() -> Self {
        Context {
            is_const: false,
            curr_func: None,
            func_param_idx: 0,
        }
    }
}

enum Calcu {
    Number(Number),
    Value(ValuePtr),
}

impl Calcu {
    pub fn is_number(&self) -> bool {
        matches!(self, Calcu::Number(_))
    }
    pub fn is_value(&self) -> bool {
        matches!(self, Calcu::Value(_))
    }
    pub fn get_number(self) -> Option<Number> {
        match self {
            Calcu::Number(a) => Some(a),
            _ => None,
        }
    }
    pub fn get_value(self) -> Option<ValuePtr> {
        match self {
            Calcu::Value(v) => Some(v),
            _ => None,
        }
    }
    pub fn unwrap_value(self) -> ValuePtr {
        match self {
            Calcu::Value(v) => v,
            Calcu::Number(_) => panic!("expected Value, got Number"),
        }
    }
}

impl Number {
    fn as_int(&self) -> i32 {
        match self {
            Number::FloatConst(f) => *f as i32,
            Number::IntConst(i) => *i,
        }
    }
    fn as_float(&self) -> f32 {
        match self {
            Number::FloatConst(f) => *f,
            Number::IntConst(i) => *i as f32,
        }
    }
}

impl Add for Number {
    type Output = Number;
    fn add(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a + b);
        }
        Number::FloatConst(self.as_float() + other.as_float())
    }
}
impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a - b);
        }
        Number::FloatConst(self.as_float() - other.as_float())
    }
}
impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a * b);
        }
        Number::FloatConst(self.as_float() * other.as_float())
    }
}
impl Div for Number {
    type Output = Number;
    fn div(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a / b);
        }
        Number::FloatConst(self.as_float() / other.as_float())
    }
}
impl Rem for Number {
    type Output = Number;
    fn rem(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a % b);
        }
        Number::FloatConst(self.as_float() % other.as_float())
    }
}

// ===== IRGenerator =====

impl IRGenerator {
    pub fn new() -> Self {
        let m = Module::new();
        let mut ir_gen = IRGenerator {
            context: Context::new(),
            scope: Scope::new(),
            buidler: InstBuilder::new(m.clone()),
            m,
        };
        ir_gen.register_runtime_functions();
        ir_gen
    }

    /// 注册运行时库函数声明 (input, output, outputFloat, neg_idx_except)
    fn register_runtime_functions(&mut self) {
        let int_ty = self.m.get_int_ty();
        let float_ty = self.m.get_float_ty();
        let void_ty = self.m.get_void_ty();

        // int input()
        let name = String::from("input");
        let func_ty = self.m.get_func_ty(int_ty.clone(), vec![]);
        let func = FunctionPtr::create(func_ty, name.clone(), self.m.clone());
        self.scope.add(&name, func.to_val());

        // void output(int)
        let name = String::from("output");
        let func_ty = self.m.get_func_ty(void_ty.clone(), vec![int_ty.clone()]);
        let func = FunctionPtr::create(func_ty, name.clone(), self.m.clone());
        self.scope.add(&name, func.to_val());

        // void outputFloat(float)
        let name = String::from("outputFloat");
        let func_ty = self.m.get_func_ty(void_ty.clone(), vec![float_ty.clone()]);
        let func = FunctionPtr::create(func_ty, name.clone(), self.m.clone());
        self.scope.add(&name, func.to_val());

        // void neg_idx_except()
        let name = String::from("neg_idx_except");
        let func_ty = self.m.get_func_ty(void_ty.clone(), vec![]);
        let func = FunctionPtr::create(func_ty, name.clone(), self.m.clone());
        self.scope.add(&name, func.to_val());
    }
    
    pub fn get_module(&self) -> ModulePtr {
        self.m.clone()
    }

    /// 将内存中的 IR 以 LLVM IR 格式打印到标准输出
    pub fn dump(&self) {
        use crate::ir::ir_printer::print_module;
        print!("{}", print_module(&self.m));
    }

    /// 将内存中的 IR 以 LLVM IR 格式返回为 String
    pub fn dump_to_string(&self) -> String {
        use crate::ir::ir_printer::print_module;
        print_module(&self.m)
    }

    fn const_int(&mut self, val: i32) -> ValuePtr {
        let c = ConstantPtr::new_int(self.m.get_int_ty(), String::new(), val);
        let v = c.0.borrow().user.to_val();
        self.m.add_const(c);
        v
    }

    fn const_float(&mut self, val: f32) -> ValuePtr {
        let c = ConstantPtr::new_float(self.m.get_float_ty(), String::new(), val);
        let v = c.0.borrow().user.to_val();
        self.m.add_const(c);
        v
    }

    fn const_zero(&mut self, ty: Rc<Type>) -> ValuePtr {
        let c = ConstantPtr::new_zero(ty, String::new());
        let v = c.0.borrow().user.to_val();
        self.m.add_const(c);
        v
    }

    /// 类型提升: 如果两个操作数一个int一个float，将int的那个sitofp
    fn promote(&self, l: ValuePtr, r: ValuePtr) -> (ValuePtr, ValuePtr) {
        let lt = l.get_type();
        let rt = r.get_type();
        if lt.is_int() && rt.is_float() {
            let new_l = self.buidler.create_sitofp(l).to_val();
            (new_l, r)
        } else if lt.is_float() && rt.is_int() {
            let new_r = self.buidler.create_sitofp(r).to_val();
            (l, new_r)
        } else {
            (l, r)
        }
    }
}

impl IRGenerator {
    pub fn visit(&mut self, ast: CompUnit) {
        for item in ast.items {
            match item {
                GlobalItem::Decl(d) => match d {
                    Decl::ConstDecl(c) => self.visit_const_decl(c),
                    Decl::VarDecl(v) => self.visit_global_var_decl(v),
                },
                GlobalItem::FuncDef(f) => self.visit_func_def(f),
            }
        }
    }

    pub fn visit_const_decl(&mut self, const_decl: ConstDecl) {
        let btype = const_decl.btype;
        for def in const_decl.const_defs {
            self.visit_const_def(&btype, def);
        }
    }

    fn visit_const_def(&mut self, btype: &Btype, def: ConstDef) {
        let base_ty = match btype {
            Btype::Int => self.m.get_int_ty(),
            Btype::Float => self.m.get_float_ty(),
        };
        if def.const_exps.is_empty() {
            // 标量常量
            self.context.is_const = true;
            let init_val = self.visit_const_init_val_scalar(&def.const_init_val);
            self.context.is_const = false;
            let val = match init_val {
                Number::IntConst(i) => self.const_int(i),
                Number::FloatConst(f) => self.const_float(f),
            };
            self.scope.add(&def.ident, val);
        } else {
            // 数组常量 - TODO: 数组初始化
            let mut dims = Vec::new();
            for d in &def.const_exps {
                self.context.is_const = true;
                let n = self.visit_const_exp_inner(&d.item).as_int();
                self.context.is_const = false;
                dims.push(n as usize);
            }
            let mut arr_ty = base_ty.clone();
            for d in dims.iter().rev() {
                arr_ty = self.m.get_arr_ty(arr_ty, *d);
            }
            // 全局 alloca
            let ptr = self.buidler.create_alloca(arr_ty);
            self.scope.add(&def.ident, ptr.to_val());
        }
    }

    fn visit_const_init_val_scalar(&mut self, init: &ConstInitVal) -> Number {
        match init {
            ConstInitVal::ConstExp(e) => self.visit_const_exp_inner(&e.item),
            ConstInitVal::ConstInitVals(_) => panic!("expected scalar const init"),
        }
    }

    fn visit_const_exp_inner(&mut self, add_exp: &AddExp) -> Number {
        let pre = self.context.is_const;
        self.context.is_const = true;
        let res = match add_exp {
            AddExp::MulExp(a) => {
                self.visit_mul_exp((**a).clone()).get_number().unwrap()
            }
            AddExp::AddExp(a, op, c) => {
                let lhs = self.visit_const_exp_inner(a);
                let rhs = self.visit_mul_exp(c.clone()).get_number().unwrap();
                match op {
                    AddOp::Add => lhs + rhs,
                    AddOp::Sub => lhs - rhs,
                }
            }
        };
        self.context.is_const = pre;
        res
    }

    pub fn visit_global_var_decl(&mut self, var_decl: VarDecl) {
        let btype = var_decl.btype;
        let base_ty = match &btype {
            Btype::Int => self.m.get_int_ty(),
            Btype::Float => self.m.get_float_ty(),
        };
        for def in var_decl.var_defs {
            match def {
                VarDef::Without(name, dims) => {
                    let ty = if dims.is_empty() {
                        base_ty.clone()
                    } else {
                        let mut arr_ty = base_ty.clone();
                        for d in dims.iter().rev() {
                            let n = self.visit_const_exp(ConstExp { item: d.item.clone() }).as_int();
                            arr_ty = self.m.get_arr_ty(arr_ty, n as usize);
                        }
                        arr_ty
                    };
                    let ptr_ty = self.m.get_ptr_ty(ty);
                    let gv = GVPtr::new(name.clone(), self.m.clone(), ptr_ty, false, None);
                    self.m.add_gv(gv.clone());
                    self.scope.add(&name, gv.0.borrow().user.to_val());
                }
                VarDef::WithInitVal(name, dims, init_val) => {
                    let ty = if dims.is_empty() {
                        base_ty.clone()
                    } else {
                        let mut arr_ty = base_ty.clone();
                        for d in dims.iter().rev() {
                            let n = self.visit_const_exp(ConstExp { item: d.item.clone() }).as_int();
                            arr_ty = self.m.get_arr_ty(arr_ty, n as usize);
                        }
                        arr_ty
                    };
                    let ptr_ty = self.m.get_ptr_ty(ty);
                    // 全局变量初始值必须是常量
                    let init = match init_val {
                        InitVal::Exp(e) => {
                            let c = self.visit_const_exp(ConstExp { item: e.item });
                            match c {
                                Number::IntConst(v) => Some(ConstantPtr::new_int(base_ty.clone(), String::new(), v)),
                                Number::FloatConst(v) => Some(ConstantPtr::new_float(base_ty.clone(), String::new(), v)),
                            }
                        }
                        _ => None,
                    };
                    let gv = GVPtr::new(name.clone(), self.m.clone(), ptr_ty, false, init);
                    self.m.add_gv(gv.clone());
                    self.scope.add(&name, gv.0.borrow().user.to_val());
                }
            }
        }
    }

    pub fn visit_var_decl(&mut self, var_decl: VarDecl) {
        let btype = var_decl.btype;
        for def in var_decl.var_defs {
            self.visit_var_def(&btype, def);
        }
    }

    fn visit_var_def(&mut self, btype: &Btype, def: VarDef) {
        let base_ty = match btype {
            Btype::Int => self.m.get_int_ty(),
            Btype::Float => self.m.get_float_ty(),
        };
        match def {
            VarDef::Without(name, dims) => {
                if dims.is_empty() {
                    let ptr = self.buidler.create_alloca(base_ty);
                    self.scope.add(&name, ptr.to_val());
                } else {
                    let mut arr_ty = base_ty;
                    for d in dims.iter().rev() {
                        let n = self.visit_const_exp(ConstExp { item: d.item.clone() }).as_int();
                        arr_ty = self.m.get_arr_ty(arr_ty, n as usize);
                    }
                    let ptr = self.buidler.create_alloca(arr_ty);
                    self.scope.add(&name, ptr.to_val());
                }
            }
            VarDef::WithInitVal(name, dims, init_val) => {
                if dims.is_empty() {
                    let ptr = self.buidler.create_alloca(base_ty);
                    let val = self.visit_exp_as_value(init_val_to_exp(init_val));
                    self.buidler.create_store(val, ptr.to_val());
                    self.scope.add(&name, ptr.to_val());
                } else {
                    let mut arr_ty = base_ty;
                    for d in dims.iter().rev() {
                        let n = self.visit_const_exp(ConstExp { item: d.item.clone() }).as_int();
                        arr_ty = self.m.get_arr_ty(arr_ty, n as usize);
                    }
                    let ptr = self.buidler.create_alloca(arr_ty);
                    // TODO: 数组初始化
                    self.scope.add(&name, ptr.to_val());
                }
            }
        }
    }

    pub fn visit_func_def(&mut self, func_def: FuncDef) {
        let ret_ty: Rc<Type> = match func_def.func_type {
            FuncType::Float => self.m.get_float_ty(),
            FuncType::Int => self.m.get_int_ty(),
            FuncType::Void => self.m.get_void_ty(),
        };
        let mut arg_ty = Vec::new();
        let mut arg_name = Vec::new();
        if let Some(params) = func_def.func_f_params {
            for p in params.items {
                let param_ty = self.get_param_type(&p);
                arg_ty.push(param_ty);
                arg_name.push(p.ident);
            }
        }
        let func_ty = self.m.get_func_ty(ret_ty.clone(), arg_ty.clone());
        let func = FunctionPtr::create(func_ty, func_def.ident.clone(), self.m.clone());
        self.scope.add(&func_def.ident, func.to_val());
        self.context.curr_func = Some(func.clone());

        let entry_bb = BasicBlockPtr::create(self.m.clone(), "entry".into(), func.clone());
        self.buidler.set_insert_point(entry_bb);
        self.scope.enter();

        // 为每个参数 alloca + store
        let args: Vec<ValuePtr> = func.0.borrow().args.iter().map(|a| a.to_val()).collect();
        for i in 0..arg_ty.len() {
            let ptr = self.buidler.create_alloca(arg_ty[i].clone());
            self.buidler.create_store(args[i].clone(), ptr.to_val());
            self.scope.add(&arg_name[i], ptr.to_val());
        }

        self.visit_block(func_def.block);

        // 如果末尾没有终止指令，补上默认return
        if !self.buidler.get_insert_block().is_terminated() {
            if ret_ty.is_void() {
                self.buidler.create_void_ret();
            } else if ret_ty.is_float() {
                let v = self.const_float(0.0);
                self.buidler.create_ret(v);
            } else {
                let v = self.const_int(0);
                self.buidler.create_ret(v);
            }
        }
        self.scope.exit();
    }

    fn get_param_type(&mut self, p: &FuncFParam) -> Rc<Type> {
        let base_ty = match p.btype {
            Btype::Int => self.m.get_int_ty(),
            Btype::Float => self.m.get_float_ty(),
        };
        if p.dims.is_none() {
            base_ty
        } else {
            let dims = p.dims.as_ref().unwrap();
            if dims.is_empty() {
                // 一维指针参数 int a[]
                self.m.get_ptr_ty(base_ty)
            } else {
                let mut inner_ty = base_ty;
                for d in dims.iter().rev() {
                    let n = self.visit_const_exp(ConstExp { item: d.item.clone() }).as_int();
                    inner_ty = self.m.get_arr_ty(inner_ty, n as usize);
                }
                self.m.get_ptr_ty(inner_ty)
            }
        }
    }

    pub fn visit_block(&mut self, block: Block) {
        self.scope.enter();
        for item in block.items {
            if self.buidler.get_insert_block().is_terminated() {
                break;
            }
            match item {
                BlockItem::Decl(d) => match d {
                    Decl::ConstDecl(c) => self.visit_const_decl(c),
                    Decl::VarDecl(v) => self.visit_var_decl(v),
                },
                BlockItem::Stmt(s) => {
                    self.visit_stmt(s);
                }
            }
        }
        self.scope.exit();
    }

    pub fn visit_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::AssignStmt(lval, exp) => {
                let val = self.visit_exp_as_value(exp);
                let ptr = self.visit_lval_as_ptr(lval);
                // 类型转换
                let val = self.convert_to_target(val, ptr.get_type().get_ptr_elem_ty().unwrap());
                self.buidler.create_store(val, ptr);
            }
            Stmt::ExpStmt(opt_exp) => {
                if let Some(exp) = opt_exp {
                    self.visit_exp_as_value(exp);
                }
            }
            Stmt::BlockStmt(block) => {
                self.visit_block(block);
            }
            Stmt::IfStmt(cond, if_stmt, else_stmt) => {
                let func = self.context.curr_func.as_ref().unwrap().clone();
                let if_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());
                let cont_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());

                let cond_val = self.visit_cond(cond);

                if let Some(else_stmt) = else_stmt {
                    let else_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());
                    self.buidler.create_cond_br(cond_val, if_bb.clone(), else_bb.clone());

                    self.buidler.set_insert_point(if_bb);
                    self.visit_stmt(*if_stmt);
                    if !self.buidler.get_insert_block().is_terminated() {
                        self.buidler.create_br(cont_bb.clone());
                    }

                    self.buidler.set_insert_point(else_bb);
                    self.visit_stmt(*else_stmt);
                    if !self.buidler.get_insert_block().is_terminated() {
                        self.buidler.create_br(cont_bb.clone());
                    }
                } else {
                    self.buidler.create_cond_br(cond_val, if_bb.clone(), cont_bb.clone());

                    self.buidler.set_insert_point(if_bb);
                    self.visit_stmt(*if_stmt);
                    if !self.buidler.get_insert_block().is_terminated() {
                        self.buidler.create_br(cont_bb.clone());
                    }
                }
                self.buidler.set_insert_point(cont_bb);
            }
            Stmt::WhileStmt(cond, body) => {
                let func = self.context.curr_func.as_ref().unwrap().clone();
                let cond_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());
                let body_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());
                let end_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());

                if !self.buidler.get_insert_block().is_terminated() {
                    self.buidler.create_br(cond_bb.clone());
                }

                self.buidler.set_insert_point(cond_bb.clone());
                let cond_val = self.visit_cond(cond);
                self.buidler.create_cond_br(cond_val, body_bb.clone(), end_bb.clone());

                self.buidler.set_insert_point(body_bb);
                self.visit_stmt(*body);
                if !self.buidler.get_insert_block().is_terminated() {
                    self.buidler.create_br(cond_bb);
                }

                self.buidler.set_insert_point(end_bb);
            }
            Stmt::BreakStmt => {
                // TODO: 需要 break/continue 栈
            }
            Stmt::ContStmt => {
                // TODO: 需要 break/continue 栈
            }
            Stmt::RetStmt(opt_exp) => {
                if let Some(exp) = opt_exp {
                    let val = self.visit_exp_as_value(exp);
                    let func_ret_ty = self.context.curr_func.as_ref().unwrap().get_return_type();
                    let val = self.convert_to_target(val, func_ret_ty);
                    self.buidler.create_ret(val);
                } else {
                    self.buidler.create_void_ret();
                }
            }
        }
    }

    /// 将 val 转换到 target_ty（int<->float）
    fn convert_to_target(&self, val: ValuePtr, target_ty: Rc<Type>) -> ValuePtr {
        let val_ty = val.get_type();
        if Rc::ptr_eq(&val_ty, &target_ty) {
            return val;
        }
        if val_ty.is_int() && target_ty.is_float() {
            self.buidler.create_sitofp(val).to_val()
        } else if val_ty.is_float() && target_ty.is_int() {
            self.buidler.create_fptosi(val, target_ty).to_val()
        } else if val_ty.is_bool() && target_ty.is_int() {
            self.buidler.create_zext(val, target_ty).to_val()
        } else {
            val
        }
    }

    pub fn visit_cond(&mut self, cond: Cond) -> ValuePtr {
        let val = self.visit_lor_exp(cond.item);
        self.to_bool(val)
    }

    pub fn visit_lor_exp(&mut self, exp: LOrExp) -> ValuePtr {
        match exp {
            LOrExp::LAndExp(e) => self.visit_land_exp(e),
            LOrExp::LOrExp(lhs, rhs) => {
                // 短路求值
                let func = self.context.curr_func.as_ref().unwrap().clone();
                let rhs_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());
                let merge_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());

                let lhs_val = self.visit_lor_exp(*lhs);
                let lhs_bool = self.to_bool(lhs_val);
                let lhs_bb = self.buidler.get_insert_block();
                self.buidler.create_cond_br(lhs_bool.clone(), merge_bb.clone(), rhs_bb.clone());

                self.buidler.set_insert_point(rhs_bb.clone());
                let rhs_val = self.visit_land_exp(rhs);
                let rhs_bool = self.to_bool(rhs_val);
                let rhs_end_bb = self.buidler.get_insert_block();
                self.buidler.create_br(merge_bb.clone());

                self.buidler.set_insert_point(merge_bb);
                // phi
                let phi = InstPtr::phi_inst(
                    self.m.get_bool_ty(),
                    vec![self.const_int(1), rhs_bool],
                    vec![lhs_bb, rhs_end_bb],
                    self.buidler.get_insert_block(),
                );
                phi.to_val()
            }
        }
    }

    pub fn visit_land_exp(&mut self, exp: LAndExp) -> ValuePtr {
        match exp {
            LAndExp::EqExp(e) => self.visit_eq_exp(e),
            LAndExp::LAndExp(lhs, rhs) => {
                // 短路求值
                let func = self.context.curr_func.as_ref().unwrap().clone();
                let rhs_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());
                let merge_bb = BasicBlockPtr::create(self.m.clone(), String::new(), func.clone());

                let lhs_val = self.visit_land_exp(*lhs);
                let lhs_bool = self.to_bool(lhs_val);
                let lhs_bb = self.buidler.get_insert_block();
                self.buidler.create_cond_br(lhs_bool.clone(), rhs_bb.clone(), merge_bb.clone());

                self.buidler.set_insert_point(rhs_bb.clone());
                let rhs_val = self.visit_eq_exp(rhs);
                let rhs_bool = self.to_bool(rhs_val);
                let rhs_end_bb = self.buidler.get_insert_block();
                self.buidler.create_br(merge_bb.clone());

                self.buidler.set_insert_point(merge_bb);
                let phi = InstPtr::phi_inst(
                    self.m.get_bool_ty(),
                    vec![self.const_int(0), rhs_bool],
                    vec![lhs_bb, rhs_end_bb],
                    self.buidler.get_insert_block(),
                );
                phi.to_val()
            }
        }
    }

    pub fn visit_eq_exp(&mut self, exp: EqExp) -> ValuePtr {
        match exp {
            EqExp::RelExp(e) => self.visit_rel_exp(e),
            EqExp::EqExp(lhs, op, rhs) => {
                let l = self.visit_eq_exp(*lhs);
                let r = self.visit_rel_exp(rhs);
                let (l, r) = self.promote(l, r);
                let inst = if l.get_type().is_int() || l.get_type().is_bool() {
                    match op {
                        EqOp::EQ => self.buidler.create_icmp_eq(l, r),
                        EqOp::NE => self.buidler.create_icmp_ne(l, r),
                    }
                } else {
                    match op {
                        EqOp::EQ => self.buidler.create_fcmp_eq(l, r),
                        EqOp::NE => self.buidler.create_fcmp_ne(l, r),
                    }
                };
                inst.to_val()
            }
        }
    }

    pub fn visit_rel_exp(&mut self, exp: RelExp) -> ValuePtr {
        match exp {
            RelExp::AddExp(e) => self.visit_add_exp_as_value(e),
            RelExp::RelExp(lhs, op, rhs) => {
                let l = self.visit_rel_exp(*lhs);
                let r = self.visit_add_exp_as_value(rhs);
                let (l, r) = self.promote(l, r);
                let inst = if l.get_type().is_int() || l.get_type().is_bool() {
                    match op {
                        RelOp::GT => self.buidler.create_icmp_gt(l, r),
                        RelOp::GE => self.buidler.create_icmp_ge(l, r),
                        RelOp::LT => self.buidler.create_icmp_lt(l, r),
                        RelOp::LE => self.buidler.create_icmp_le(l, r),
                    }
                } else {
                    match op {
                        RelOp::GT => self.buidler.create_fcmp_gt(l, r),
                        RelOp::GE => self.buidler.create_fcmp_ge(l, r),
                        RelOp::LT => self.buidler.create_fcmp_lt(l, r),
                        RelOp::LE => self.buidler.create_fcmp_le(l, r),
                    }
                };
                inst.to_val()
            }
        }
    }

    fn visit_add_exp_as_value(&mut self, add_exp: AddExp) -> ValuePtr {
        self.visit_add_exp(add_exp).unwrap_value()
    }

    pub fn visit_exp_as_value(&mut self, exp: Exp) -> ValuePtr {
        let calcu = self.visit_add_exp(exp.item);
        match calcu {
            Calcu::Value(v) => v,
            Calcu::Number(n) => match n {
                Number::IntConst(i) => self.const_int(i),
                Number::FloatConst(f) => self.const_float(f),
            },
        }
    }

    pub fn visit_const_exp(&mut self, const_exp: ConstExp) -> Number {
        let pre_is_const = self.context.is_const;
        self.context.is_const = true;
        let res = match const_exp.item {
            AddExp::MulExp(a) => self.visit_mul_exp(*a).get_number().unwrap(),
            AddExp::AddExp(a, b, c) => {
                let add_res = self.visit_add_exp(*a).get_number().unwrap();
                let mul_res = self.visit_mul_exp(c).get_number().unwrap();
                match b {
                    AddOp::Sub => add_res - mul_res,
                    AddOp::Add => add_res + mul_res,
                }
            }
        };
        self.context.is_const = pre_is_const;
        res
    }

    pub fn visit_add_exp(&mut self, add_exp: AddExp) -> Calcu {
        if self.context.is_const {
            let res = match add_exp {
                AddExp::MulExp(a) => self.visit_mul_exp(*a).get_number().unwrap(),
                AddExp::AddExp(a, b, c) => {
                    let add_res = self.visit_add_exp(*a).get_number().unwrap();
                    let mul_res = self.visit_mul_exp(c).get_number().unwrap();
                    match b {
                        AddOp::Sub => add_res - mul_res,
                        AddOp::Add => add_res + mul_res,
                    }
                }
            };
            Calcu::Number(res)
        } else {
            match add_exp {
                AddExp::MulExp(a) => self.visit_mul_exp(*a),
                AddExp::AddExp(a, op, c) => {
                    let l = self.visit_add_exp(*a).unwrap_value();
                    let r = self.visit_mul_exp(c).unwrap_value();
                    let (l, r) = self.promote(l, r);
                    let inst = if l.get_type().is_int() {
                        match op {
                            AddOp::Add => self.buidler.create_iadd(l, r),
                            AddOp::Sub => self.buidler.create_isub(l, r),
                        }
                    } else {
                        match op {
                            AddOp::Add => self.buidler.create_fadd(l, r),
                            AddOp::Sub => self.buidler.create_fsub(l, r),
                        }
                    };
                    Calcu::Value(inst.to_val())
                }
            }
        }
    }

    pub fn visit_mul_exp(&mut self, mul_exp: MulExp) -> Calcu {
        if self.context.is_const {
            let res = match mul_exp {
                MulExp::UnaryExp(a) => self.visit_unary_exp(*a).get_number().unwrap(),
                MulExp::MulExp(a, b, c) => {
                    let lhs = self.visit_mul_exp(*a).get_number().unwrap();
                    let rhs = self.visit_unary_exp(c).get_number().unwrap();
                    match b {
                        MulOp::Mul => lhs * rhs,
                        MulOp::Div => lhs / rhs,
                        MulOp::Mod => lhs % rhs,
                    }
                }
            };
            Calcu::Number(res)
        } else {
            match mul_exp {
                MulExp::UnaryExp(a) => self.visit_unary_exp(*a),
                MulExp::MulExp(a, op, c) => {
                    let l = self.visit_mul_exp(*a).unwrap_value();
                    let r = self.visit_unary_exp(c).unwrap_value();
                    let (l, r) = self.promote(l, r);
                    let inst = if l.get_type().is_int() {
                        match op {
                            MulOp::Mul => self.buidler.create_imul(l, r),
                            MulOp::Div => self.buidler.create_isdiv(l, r),
                            MulOp::Mod => {
                                // a % b = a - (a / b) * b
                                let div = self.buidler.create_isdiv(l.clone(), r.clone());
                                let mul = self.buidler.create_imul(div.to_val(), r);
                                self.buidler.create_isub(l, mul.to_val())
                            }
                        }
                    } else {
                        match op {
                            MulOp::Mul => self.buidler.create_fmul(l, r),
                            MulOp::Div => self.buidler.create_fdiv(l, r),
                            MulOp::Mod => {
                                // float mod: a - floor(a/b)*b  (简化: 直接用fmul/fsub)
                                let div = self.buidler.create_fdiv(l.clone(), r.clone());
                                let mul = self.buidler.create_fmul(div.to_val(), r);
                                self.buidler.create_fsub(l, mul.to_val())
                            }
                        }
                    };
                    Calcu::Value(inst.to_val())
                }
            }
        }
    }

    pub fn visit_unary_exp(&mut self, unary_exp: UnaryExp) -> Calcu {
        if self.context.is_const {
            let res = match unary_exp {
                UnaryExp::UnaryExp(op, b) => {
                    let mut val = self.visit_unary_exp(*b).get_number().unwrap();
                    match op {
                        UnaryOp::Minus => val = Number::IntConst(0) - val,
                        UnaryOp::Not => {
                            val = Number::IntConst(if val.as_int() == 0 { 1 } else { 0 });
                        }
                        UnaryOp::Pos => {}
                    }
                    val
                }
                UnaryExp::PrimaryExp(a) => self.visit_prim_exp(*a).get_number().unwrap(),
                _ => panic!("function call in const context"),
            };
            Calcu::Number(res)
        } else {
            match unary_exp {
                UnaryExp::PrimaryExp(a) => self.visit_prim_exp(*a),
                UnaryExp::UnaryExp(op, b) => {
                    let val = self.visit_unary_exp(*b).unwrap_value();
                    let result = match op {
                        UnaryOp::Pos => val,
                        UnaryOp::Minus => {
                            if val.get_type().is_int() {
                                let zero = self.const_int(0);
                                self.buidler.create_isub(zero, val).to_val()
                            } else {
                                let zero = self.const_float(0.0);
                                self.buidler.create_fsub(zero, val).to_val()
                            }
                        }
                        UnaryOp::Not => {
                            if val.get_type().is_int() {
                                let zero = self.const_int(0);
                                self.buidler.create_icmp_eq(val, zero).to_val()
                            } else {
                                let zero = self.const_float(0.0);
                                self.buidler.create_fcmp_eq(val, zero).to_val()
                            }
                        }
                    };
                    Calcu::Value(result)
                }
                UnaryExp::CallExp(name, args) => {
                    let func_val = self.scope.find(&name).expect("function not found");
                    let func = func_val.to_function().expect("not a function");
                    let mut v_args = Vec::new();
                    if let Some(params) = args {
                        let func_ty = func.get_type();
                        for (idx, arg_exp) in params.items.into_iter().enumerate() {
                            let mut val = self.visit_exp_as_value(arg_exp);
                            let param_ty = func_ty.get_func_param_ty(idx).unwrap();
                            val = self.convert_to_target(val, param_ty);
                            v_args.push(val);
                        }
                    }
                    let inst = InstPtr::call_inst(func, v_args, self.buidler.get_insert_block());
                    Calcu::Value(inst.to_val())
                }
            }
        }
    }

    pub fn visit_prim_exp(&mut self, prim_exp: PrimaryExp) -> Calcu {
        match prim_exp {
            PrimaryExp::Number(n) => {
                if self.context.is_const {
                    Calcu::Number(n)
                } else {
                    match n {
                        Number::IntConst(i) => Calcu::Value(self.const_int(i)),
                        Number::FloatConst(f) => Calcu::Value(self.const_float(f)),
                    }
                }
            }
            PrimaryExp::Exp(e) => {
                if self.context.is_const {
                    Calcu::Number(self.visit_const_exp(ConstExp { item: e.item }))
                } else {
                    Calcu::Value(self.visit_exp_as_value(*e))
                }
            }
            PrimaryExp::LVal(lval) => {
                if self.context.is_const {
                    // 在常量上下文中，从scope中查找常量值
                    let val = self.scope.find(&lval.ident).expect("const not found");
                    let c = val.to_const().expect("not a constant in const context");
                    match &c.0.borrow().ext {
                        crate::ir::user::ConstantClass::Int(i) => Calcu::Number(Number::IntConst(*i)),
                        crate::ir::user::ConstantClass::Float(f) => Calcu::Number(Number::FloatConst(*f)),
                        _ => panic!("unexpected constant type"),
                    }
                } else {
                    if lval.dims.is_empty() {
                        // 标量: load
                        let ptr = self.scope.find(&lval.ident).expect("variable not found");
                        if ptr.get_type().is_ptr() {
                            let val = self.buidler.create_load(ptr).to_val();
                            Calcu::Value(val)
                        } else {
                            // 可能是常量直接引用
                            Calcu::Value(ptr)
                        }
                    } else {
                        // 数组访问
                        let ptr = self.visit_lval_as_ptr(lval);
                        let val = self.buidler.create_load(ptr).to_val();
                        Calcu::Value(val)
                    }
                }
            }
        }
    }

    /// 获取左值地址（用于赋值和数组访问）
    fn visit_lval_as_ptr(&mut self, lval: LVal) -> ValuePtr {
        let base_ptr = self.scope.find(&lval.ident).expect("variable not found");
        if lval.dims.is_empty() {
            base_ptr
        } else {
            // 数组索引: GEP
            let mut idxs = vec![self.const_int(0)];
            for d in lval.dims {
                let idx = self.visit_exp_as_value(d);
                let idx = if idx.get_type().is_float() {
                    self.buidler.create_fptosi(idx, self.m.get_int_ty()).to_val()
                } else {
                    idx
                };
                idxs.push(idx);
            }
            self.buidler.create_gep(base_ptr, idxs).to_val()
        }
    }

    /// 将值转为 bool (用于条件判断)
    fn to_bool(&mut self, val: ValuePtr) -> ValuePtr {
        let ty = val.get_type();
        if ty.is_bool() {
            val
        } else if ty.is_int() {
            let zero = self.const_int(0);
            self.buidler.create_icmp_ne(val, zero).to_val()
        } else {
            let zero = self.const_float(0.0);
            self.buidler.create_fcmp_ne(val, zero).to_val()
        }
    }
}

// helper
fn init_val_to_exp(init: InitVal) -> Exp {
    match init {
        InitVal::Exp(e) => e,
        _ => panic!("expected scalar init val"),
    }
}
