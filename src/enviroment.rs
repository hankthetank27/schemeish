use core::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use crate::parser::Expr;

#[derive(Debug, Clone, PartialEq)]
pub struct EnvRef(Rc<RefCell<Option<Env>>>);

impl EnvRef {
    pub fn new_empty() -> EnvRef {
        EnvRef(Rc::new(RefCell::new(None)))
    }

    pub fn new(env: Env) -> EnvRef {
        EnvRef(Rc::new(RefCell::new(Some(env))))
    }

    pub fn clone_rc(&self) -> EnvRef {
        EnvRef(Rc::clone(&self.0))
    }

    pub fn get_val(&self, name: &str) -> Option<Expr> {
        self.0.borrow_mut().as_ref()?.get_val(name)
    }

    pub fn insert_val(&self, name: String, val: Expr) -> Option<Expr> {
        Some(self.0.borrow_mut().as_mut()?.insert_val(name, val))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    parent: EnvRef,
    values: HashMap<String, Expr>,
}

impl Env {
    pub fn new(parent: EnvRef) -> Self {
        Env {
            parent,
            values: HashMap::new(),
        }
    }

    pub fn get_val(&self, name: &str) -> Option<Expr> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| self.parent.get_val(name))
    }

    pub fn insert_val(&mut self, name: String, val: Expr) -> Expr {
        self.values.insert(name, val.clone());
        val
    }
}
