use core::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use crate::parser::Expr;

pub type EnvRef = Rc<RefCell<Option<Env>>>;

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
            .or_else(|| match self.parent.as_ref().borrow().as_ref() {
                Some(parent) => parent.get_val(name),
                None => None, //unbound variable
            })
    }

    pub fn insert_val(&mut self, name: String, val: Expr) -> Expr {
        self.values.insert(name, val.clone());
        val
    }
}
