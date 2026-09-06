use crate::ir::module::ModulePtr;
use crate::ir::user::Inst;
use crate::ir::value::{BasicBlock, Value, ValuePtr};
use crate::{ir::value::Function, weak_ptr};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub struct AsmGen {
    pub m: ModulePtr,
    pub context: Context,
    pub output: Vec<AsmInst>,
}

pub struct Context {
    pub func: weak_ptr!(Function),
    pub bb: weak_ptr!(BasicBlock),
    pub inst: weak_ptr!(Inst),
    pub frame_size: usize,
    pub offset_map: HashMap<usize, i32>, // Use Value ID as key
    pub fcmp_cnt: usize,
}

impl Context {
    pub fn new() -> Self {
        Context {
            func: Weak::new(),
            bb: Weak::new(),
            inst: Weak::new(),
            frame_size: 0,
            offset_map: HashMap::new(),
            fcmp_cnt: 0,
        }
    }

    pub fn clear(&mut self) {
        self.func = Weak::new();
        self.bb = Weak::new();
        self.inst = Weak::new();
        self.frame_size = 0;
        self.fcmp_cnt = 0;
        self.offset_map.clear();
    }
}

#[derive(Clone)]
pub enum AsmInst {
    Instruction(String),
    Attribute(String),
    Label(String),
    Comment(String),
}

impl AsmInst {
    pub fn format(&self) -> String {
        match self {
            AsmInst::Instruction(s) | AsmInst::Attribute(s) => format!("\t{}\n", s),
            AsmInst::Label(s) => format!("{}:\n", s),
            AsmInst::Comment(s) => format!("# {}\n", s),
        }
    }
}
