use crate::ast::{AddExp, AddOp, Block, Btype, CompUnit, ConstDecl, ConstExp, Decl, FuncDef, FuncType, GlobalItem, MulExp, MulOp, Number, PrimaryExp, UnaryExp, UnaryOp, VarDecl};
use crate::ir::module::{Module, ModulePtr};
use crate::ir::type_::Type;
use crate::{make_ptr, opt_ptr};
use crate::{ir::value::Value, ptr, irgen::scope::Scope};
use std::rc::Rc;
use std::cell::RefCell;
use crate::ir::inst_builder::InstBuilder;
use std::ops::{Add, Div, Mul, Not, Rem, Sub};

struct Context {
    in_lval: bool,
    is_const: bool,
    curr_func: opt_ptr!(Value), //* Function */
    func_param_idx: usize
}

pub struct IRGenerator {
    context: Context,
    scope: Scope,
    buidler: InstBuilder,
    m: ModulePtr
}

impl Context {
    pub fn new() -> Self {
        Context { in_lval: false, is_const: false, curr_func: None, func_param_idx: 0 }
    }
}

enum Calcu {
    Number(Number),
    Value(ptr!(Value))
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
            _ => None
        }
    }
}

impl Number {
    fn as_int(&self) -> i32 {
        match self {
            Number::FloatConst(f) => *f as i32,
            Number::IntConst(i) => *i
        }
    }
    fn as_float(&self) -> f32 {
        match self {
            Number::FloatConst(f) => *f,
            Number::IntConst(i) => *i as f32
        }
    }
}


impl Add for Number{
    type Output = Number;
    fn add(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a+b);
        }
        let a = match self {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        let b = match other {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        Number::FloatConst(a+b)
    }
}
impl Mul for Number{
    type Output = Number;
    fn mul(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a*b);
        }
        let a = match self {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        let b = match other {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        Number::FloatConst(a*b)
    }
}
impl Sub for Number{
    type Output = Number;
    fn sub(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a-b);
        }
        let a = match self {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        let b = match other {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        Number::FloatConst(a-b)
    }
}
impl Div for Number{
    type Output = Number;
    fn div(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a/b);
        }
        let a = match self {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        let b = match other {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        Number::FloatConst(a/b)
    }
}
impl Rem for Number{
    type Output = Number;
    fn rem(self, other: Number) -> Self::Output {
        if let Number::IntConst(a) = self && let Number::IntConst(b) = other {
            return Number::IntConst(a%b);
        }
        let a = match self {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        let b = match other {
            Number::FloatConst(f) => f,
            Number::IntConst(i) => i as f32
        };
        Number::FloatConst(a%b)
    }
}

// * 这个一般是用访问者模式来实现，但是rust实现访问者模式比较麻烦，干脆直接实现
impl IRGenerator {
    pub fn new() -> Self {
        let m = Module::new();
        IRGenerator {
            context: Context::new(),
            scope: Scope::new(),
            buidler: InstBuilder::new(m.clone()),
            m
        }
    }
    pub fn get_module(&self) -> ModulePtr {
        self.m.clone()
    }
}

impl IRGenerator {
    pub fn visit(&mut self, ast: CompUnit) {
        for i in ast.items {
            match i {
                GlobalItem::Decl(d) => {
                    match d {
                        Decl::ConstDecl(c) => {
                            
                        },
                        Decl::VarDecl(v) => {

                        }
                    }
                },
                GlobalItem::FuncDef(f) => {

                }
            }
        }
    }
    pub fn visit_const_decl(&mut self, const_decl:ConstDecl) {

    }
    pub fn visit_var_decl(&mut self, var_decl:VarDecl) {

    }
    pub fn visit_func_def(&mut self, func_def: FuncDef) {
        let ret_ty: Rc<Type> =
            match func_def.func_type {
                FuncType::Float => self.get_module().get_float_ty(),
                FuncType::Int => self.get_module().get_int_ty(),
                FuncType::Void => self.get_module().get_void_ty()         
            };
        let mut arg_ty = Vec::new();
        let mut arg_name = Vec::new();
        match func_def.func_f_params {
            Some(a) => {
                for i in a.items {
                    let param_ty:Rc<Type>;
                    if i.dims.is_none() {
                        param_ty = match i.btype {
                            Btype::Float => {
                                self.get_module().get_float_ty()
                            },
                            Btype::Int => {
                                self.get_module().get_int_ty()
                            }
                        }
                    } else {
                        let mut dim_v = Vec::new();
                        for d in i.dims.unwrap() {
                            let temp = self.visit_const_exp(d).as_int();
                            dim_v.push(temp);
                        }
                        let mut base_type = match i.btype {
                            Btype::Float => {
                                self.get_module().get_float_ty()
                            },
                            Btype::Int => {
                                self.get_module().get_int_ty()
                            }
                        };
                        for i in dim_v.iter().rev() {
                            base_type = self.get_module().get_arr_ty(base_type, *i as usize);
                        }
                        param_ty = self.get_module().get_ptr_ty(base_type);
                    }
                    arg_ty.push(param_ty);
                    arg_name.push(i.ident);
                }
            },
            None => {}
        }
        assert!(arg_name.len() == arg_ty.len());
        let func_ty = self.get_module().get_func_ty(ret_ty, arg_ty.clone());
        let func = Value::create_func(func_def.ident.clone(), self.get_module().0, func_ty);
        self.context.curr_func = Some(func.clone());
        self.scope.add(&func_def.ident, func.clone());
        //* 创建arg参数的变量
        for i in 0..arg_ty.len() {
            let arg = Value::create_arg(Some(arg_name[i].clone()), func.clone(), arg_ty[i].clone(), i);
            func.borrow_mut().func_add_arg(make_ptr!(arg), i);
        }
        let bb = Value::create_bb("entry".into(), self.get_module().0, func.clone());
        self.buidler.set_insert_point(bb);
        // TODO 为arg分配空间
        self.visit_compound_stmt(func_def.block);
        if self.buidler.get_insert_block().borrow().bb_is_terminated().not() {
            let ret_ty = func.borrow().get_type().get_func_ret_ty().unwrap();
            if ret_ty.is_bool() {
                let a = Value::create_const_int(self.get_module().get_bool_ty(), 0);
                self.buidler.create_ret(make_ptr!(a));
            } else if ret_ty.is_float() {
                let a = Value::create_const_float(self.get_module().get_float_ty(), 0.0);
                self.buidler.create_ret(make_ptr!(a));
            } else if ret_ty.is_void() {
                self.buidler.create_void_ret();
            } else if ret_ty.is_int() {
                let a = Value::create_const_int(self.get_module().get_int_ty(), 0);
                self.buidler.create_ret(make_ptr!(a));
            }
        }
        self.scope.exit();
    }

    pub fn visit_compound_stmt(&mut self, compound_stmt: Block) {
        todo!();
    }

    pub fn visit_const_exp(&mut self, const_exp: ConstExp) -> Number {
        let pre_is_const = self.context.is_const;
        self.context.is_const = true;
        let res:Number;
        match const_exp.item {
            AddExp::MulExp(a) => {
                res = self.visit_mul_exp(*a).get_number().unwrap();
            },
            AddExp::AddExp(a, b, c) => {
                let add_res = self.visit_add_exp(*a).get_number().unwrap(); 
                let mul_res = self.visit_mul_exp(c).get_number().unwrap();
                match b {
                    AddOp::Sub => {
                        res = add_res - mul_res
                    },
                    AddOp::Add => {
                        res = add_res + mul_res
                    }
                }
            }
        }
        self.context.is_const = pre_is_const;
        res
    }

    pub fn visit_add_exp(&mut self, add_exp: AddExp) -> Calcu {
        if self.context.is_const {
            let res:Number;
            match add_exp {
                AddExp::MulExp(a) => {
                    res = self.visit_mul_exp(*a).get_number().unwrap();
                },
                AddExp::AddExp(a, b, c) => {
                    let add_res = self.visit_add_exp(*a).get_number().unwrap(); 
                    let mul_res = self.visit_mul_exp(c).get_number().unwrap();
                    match b {
                        AddOp::Sub => {
                            res = add_res - mul_res
                        },
                        AddOp::Add => {
                            res = add_res + mul_res
                        }
                    }
                }
            }
            Calcu::Number(res)
        } else {
            todo!();
        }
    }

    pub fn visit_mul_exp(&mut self, mul_exp: MulExp) -> Calcu {
        if self.context.is_const {
            let res:Number;
            match mul_exp {
                MulExp::UnaryExp(a) => {
                    res = self.visit_unary_exp(*a).get_number().unwrap();
                },
                MulExp::MulExp(a, b, c) => {
                    let add_res = self.visit_mul_exp(*a).get_number().unwrap(); 
                    let mul_res = self.visit_unary_exp(c).get_number().unwrap();
                    match b {
                        MulOp::Mul => {
                            res = add_res * mul_res
                        },
                        MulOp::Div => {
                            res = add_res / mul_res
                        }
                        MulOp::Mod => {
                            res = add_res % mul_res
                        }
                    }
                }
            }
            Calcu::Number(res)
        } else {
            todo!();
        }
    }

    pub fn visit_unary_exp(&mut self, unary_exp: UnaryExp) -> Calcu {
        if self.context.is_const {
            let mut res:Number;
            match unary_exp {
                UnaryExp::UnaryExp(a,b) => {
                    res = self.visit_unary_exp(*b).get_number().unwrap();
                    // TODO 这里会出现Not这个操作符吗
                    if let UnaryOp::Minus = a {
                        res = Number::IntConst(0) - res;
                    }
                },
                UnaryExp::PrimaryExp(a) => {
                    res = self.visit_prim_exp(*a).get_number().unwrap(); 
                },
                _ => panic!("")
            }
            Calcu::Number(res)
        } else {
            todo!();
        }
    }
    
    pub fn visit_prim_exp(&mut self, prim_exp: PrimaryExp) -> Calcu {
        todo!()
    }
}