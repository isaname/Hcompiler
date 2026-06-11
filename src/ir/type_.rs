use crate::{ptr, opt_ptr, weak_ptr};

use super::module::Module;

use std::{cell::RefCell, mem::discriminant, rc::{Rc, Weak}};

// #[derive(Ord)]

pub struct Type {
    tdata:TypeData,
    m:weak_ptr!(Module),
}

// #[derive(Ord)]

pub enum TypeData {
    VoidType,
    LabelType,
    IntegerType,
    FloatType,
    BoolType,
    FunctionType(Rc<FunctionType>),
    ArrayType(Rc<ArrayType>),
    PointerType(Rc<PointerType>),
}

pub struct FunctionType {
    pub result:Rc<Type>,// * 我真的需要内部可变性吗
    pub args: Vec<Rc<Type>>,
}

pub struct ArrayType {
    pub item:Rc<Type>,
    pub elem_count:usize
}


pub struct PointerType {
    pub item:Rc<Type>
}

impl Type{
    pub fn new(tdata:TypeData, m:weak_ptr!(Module)) -> Self{
        Type {
            tdata,
            m,
        }
    }

    pub fn tdata(&self) -> &TypeData {
        &self.tdata
    }

    pub fn get_module(&self) -> weak_ptr!(Module) {
        self.m.clone()
    }

    /// 仅仅只是判断id是否相同，具体信息不管
    pub fn is_same_id(&self, other:&Self) -> bool {
        discriminant(&self.tdata) == discriminant(&other.tdata)
    }

    pub fn get_ptr_elem_ty(&self) -> Option<Rc<Type>> {
        match &self.tdata {
            TypeData::PointerType(tdata) => {
                Some(tdata.item.clone())
            },
            _ => {
                None
            }
        }
    }

    pub fn get_arr_elem_ty(&self) -> Option<Rc<Type>> {
        match &self.tdata {
            TypeData::ArrayType(tdata) => {
                Some(tdata.item.clone())
            },
            _ => {
                None
            }
        }
    }
}

impl Type {
    pub fn is_void(&self) -> bool {
        matches!(self.tdata, TypeData::VoidType)
    }
    pub fn is_label(&self) -> bool {
        matches!(self.tdata, TypeData::LabelType)
    }
    pub fn is_int(&self) -> bool {
        matches!(self.tdata, TypeData::IntegerType)
    }
    pub fn is_float(&self) -> bool {
        matches!(self.tdata, TypeData::FloatType)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self.tdata, TypeData::BoolType)
    }
    pub fn is_func(&self) -> bool {
        matches!(self.tdata, TypeData::FunctionType(_))
    }
    pub fn is_arr(&self) -> bool {
        matches!(self.tdata, TypeData::ArrayType(_))
    }
    pub fn is_ptr(&self) -> bool {
        matches!(self.tdata, TypeData::PointerType(_))
    }
    pub fn is_bool_or_int(&self) -> bool {
        self.is_bool() || self.is_int()
    }
}

impl Type {
    // * function type
    /// 获取输入type的return type, 要求输入必须是Function type
    pub fn get_func_ret_ty(&self) -> Option<Rc<Type>> {
        match &self.tdata {
            TypeData::FunctionType(f) => {
                Some(f.result.clone())
            }
            _ => {
                None
            }
        }
    }

    pub fn get_func_arg_num(&self) -> Option<usize> {
        match &self.tdata {
            TypeData::FunctionType(f) => {
                Some(f.args.len())
            }
            _ => {
                None
            }
        }
    }

    pub fn get_func_param_ty(&self, idx: usize) -> Option<Rc<Type>> {
        match &self.tdata {
            TypeData::FunctionType(f) => {
                assert!(idx<f.args.len());
                Some(f.args.get(idx).unwrap().clone())
            }
            _ => {
                None
            }
        }
    }
}