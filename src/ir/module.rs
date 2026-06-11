use std::{
    cell::RefCell,
    collections::{HashMap, LinkedList},
    ops::Not,
    rc::{Rc, Weak},
};


use crate::{
    ir::{
        type_::{ArrayType, FunctionType, PointerType, Type, TypeData},
        user::{ConstantPtr, GVPtr, GlobalVariable, User},
        value::{Function, FunctionPtr, Value}
    },
    make_ptr, ptr, weak_ptr,
};

pub struct ModulePtr(pub ptr!(Module));

pub struct Module {
    pub(crate) gv_list: LinkedList<GVPtr>, //* GlobalValue
    pub(crate) func_list: LinkedList<FunctionPtr>, //* Function
    pub(crate) const_pool: Vec<ConstantPtr>,
    type_cache: TypeCache,
}

pub struct TypeCache {
    bool_ty: Rc<Type>,
    int_ty: Rc<Type>,
    label_ty: Rc<Type>,
    void_ty: Rc<Type>,
    float_ty: Rc<Type>,
    ptr_map: Vec<(Rc<PointerType>, Rc<Type>)>,
    arr_map: Vec<(Rc<ArrayType>, Rc<Type>)>,
    func_map: Vec<(Rc<FunctionType>, Rc<Type>)>,
}

impl Module {
    pub fn new() -> ModulePtr {
        let temp = Module {
            gv_list: LinkedList::new(),
            func_list: LinkedList::new(),
            const_pool: Vec::new(),
            type_cache: TypeCache::new(Weak::new()),
        };
        let temp = make_ptr!(temp);
        let b = TypeCache::new(Rc::downgrade(&temp));
        temp.borrow_mut().type_cache = b;
        ModulePtr(temp)
    }
}

impl TypeCache {
    fn new(m: weak_ptr!(Module)) -> Self {
        TypeCache {
            bool_ty: Type::new(TypeData::BoolType, m.clone()).into(),
            int_ty: Type::new(TypeData::IntegerType, m.clone()).into(),
            label_ty: Type::new(TypeData::LabelType, m.clone()).into(),
            void_ty: Type::new(TypeData::VoidType, m.clone()).into(),
            float_ty: Type::new(TypeData::FloatType, m.clone()).into(),
            ptr_map: Vec::new(),
            arr_map: Vec::new(),
            func_map: Vec::new(),
        }
    }
}

impl ModulePtr {
    pub fn clone(&self) -> Self {
        ModulePtr(self.0.clone())
    }
    pub fn get_void_ty(&self) -> Rc<Type> {
        self.0.borrow().type_cache.void_ty.clone()
    }
    pub fn get_lable_ty(&self) -> Rc<Type> {
        self.0.borrow().type_cache.label_ty.clone()
    }
    pub fn get_bool_ty(&self) -> Rc<Type> {
        self.0.borrow().type_cache.bool_ty.clone()
    }
    pub fn get_int_ty(&self) -> Rc<Type> {
        self.0.borrow().type_cache.int_ty.clone()
    }
    pub fn get_float_ty(&self) -> Rc<Type> {
        self.0.borrow().type_cache.float_ty.clone()
    }
    pub fn get_int_ptr_ty(&mut self) -> Rc<Type> {
        self.get_ptr_ty(self.get_int_ty())
    }
    pub fn get_float_ptr_ty(&mut self) -> Rc<Type> {
        self.get_ptr_ty(self.get_float_ty())
    }
    pub fn get_ptr_ty(&mut self, item: Rc<Type>) -> Rc<Type> {
        let found = self
            .0
            .borrow()
            .type_cache
            .ptr_map
            .iter()
            .find(|a| Rc::ptr_eq(&a.0.item, &item))
            .map(|a| a.1.clone());
        if let Some(ty) = found {
            return ty;
        }
        let key = Rc::new(PointerType { item: item.clone() });
        let value = Type::new(TypeData::PointerType(key.clone()), Rc::downgrade(&self.0));
        let value = Rc::new(value);
        self.0
            .borrow_mut()
            .type_cache
            .ptr_map
            .push((key, value.clone()));
        value
    }
    pub fn get_arr_ty(&mut self, item: Rc<Type>, elem_count: usize) -> Rc<Type> {
        let found = self
            .0
            .borrow()
            .type_cache
            .arr_map
            .iter()
            .find(|a| Rc::ptr_eq(&a.0.item, &item) && elem_count == a.0.elem_count)
            .map(|a| a.1.clone());
        if let Some(ty) = found {
            return ty;
        }
        let key = Rc::new(ArrayType {
            item: item.clone(),
            elem_count,
        });
        let value = Type::new(TypeData::ArrayType(key.clone()), Rc::downgrade(&self.0));
        let value = Rc::new(value);
        self.0
            .borrow_mut()
            .type_cache
            .arr_map
            .push((key, value.clone()));
        value
    }

    pub fn get_func_ty(&mut self, ret_ty: Rc<Type>, args: Vec<Rc<Type>>) -> Rc<Type> {
        let found = self.0.borrow().type_cache.func_map.iter().find(|a| {
            if Rc::ptr_eq(&a.0.result, &ret_ty).not() || a.0.args.len() != args.len() {
                return false;
            }

            for (idx, i) in args.iter().enumerate() {
                if Rc::ptr_eq(&a.0.args[idx], i).not() {
                    return false;
                }
            }
            true
        }).map(|a| a.1.clone());
        if let Some(ty) = found {
            return ty;
        }
        let key = Rc::new(FunctionType {
            result: ret_ty.clone(),
            args: args,
        });
        let value = Type::new(TypeData::FunctionType(key.clone()), Rc::downgrade(&self.0));
        let value = Rc::new(value);
        self.0
            .borrow_mut()
            .type_cache
            .func_map
            .push((key, value.clone()));
        value
    }

    pub fn add_function(&mut self, func: FunctionPtr) {
        self.0.borrow_mut().func_list.push_back(func);
    }

    pub fn add_gv(&mut self, gv: GVPtr) {
        self.0.borrow_mut().gv_list.push_back(gv);
    }

    pub fn add_const(&mut self, c: ConstantPtr) {
        self.0.borrow_mut().const_pool.push(c);
    }
}
