use std::collections::HashMap;

use crate::{ir::value::Value, opt_ptr, ptr};
use std::rc::Rc;
use std::cell::RefCell;
pub struct Scope {
    container: Vec<HashMap<String, ptr!(Value)>>
}

impl Scope {
    pub fn new()->Self {
        let mut res = Scope { container: Vec::new() };
        res.enter();
        res
    }
    pub fn enter(&mut self) {
        self.container.push(HashMap::new());
    }
    pub fn exit(&mut self) {
        self.container.pop();
    }
    pub fn find(&self, name: &String) -> opt_ptr!(Value) {
        for i in self.container.iter().rev() {
            if i.contains_key(name) {
                return i.get(name).cloned()
            }
        }
        None
    }
    pub fn add(&mut self, name: &String, val: ptr!(Value)) -> bool {
        let item = self.container.last_mut().unwrap();
        if item.contains_key(name) {
            return false;
        } 
        item.insert(name.clone(), val);
        return true;
    }
}