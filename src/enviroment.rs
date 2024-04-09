use core::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use crate::{
    parser::Expr,
    primitives,
    procedure::{PSig, Primitive, Proc},
    special_forms,
};

#[derive(Debug, Clone, PartialEq)]
pub struct EnvRef(Rc<RefCell<Option<Env>>>);

impl EnvRef {
    pub fn nil() -> EnvRef {
        EnvRef(Rc::new(RefCell::new(None)))
    }

    pub fn new(env: Env) -> EnvRef {
        EnvRef(Rc::new(RefCell::new(Some(env))))
    }

    pub fn global() -> EnvRef {
        let global = EnvRef::new(Env::new(EnvRef::nil()));
        install_primitives(global)
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

fn install_primitives(env: EnvRef) -> EnvRef {
    let primitives = [
        ("define", special_forms::define as PSig),
        ("lambda", special_forms::lambda as PSig),
        ("if", special_forms::if_statement as PSig),
        ("+", primitives::add as PSig),
        ("-", primitives::subtract as PSig),
        ("*", primitives::multiply as PSig),
        ("/", primitives::divide as PSig),
        ("=", primitives::equality as PSig),
        (">", primitives::greater_than as PSig),
        (">=", primitives::greater_than_or_eq as PSig),
        ("<", primitives::less_than as PSig),
        ("<=", primitives::less_than_or_eq as PSig),
    ];

    for (name, proc) in primitives.into_iter() {
        env.insert_val(
            name.to_string(),
            Expr::Proc(Proc::Primitive(Primitive::new(proc))),
        );
    }

    env
}
