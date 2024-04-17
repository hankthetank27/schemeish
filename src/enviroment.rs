use core::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::EvalErr;
use crate::parser::Expr;
use crate::primitives::{numeric, pair, special_form, utils::ToExpr};
use crate::procedure::{PSig, Primitive};

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

    pub fn get_val(&self, name: &str) -> Result<Expr, EvalErr> {
        self.0
            .borrow()
            .as_ref()
            .ok_or_else(|| EvalErr::UnboundVar(name.to_string()))?
            .get_val(name)
    }

    pub fn insert_val(&self, name: String, val: Expr) -> Result<Expr, EvalErr> {
        self.0
            .borrow_mut()
            .as_mut()
            .ok_or(EvalErr::NilEnv)
            .map(|env| env.insert_val(name, val))
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

    pub fn get_val(&self, name: &str) -> Result<Expr, EvalErr> {
        self.values
            .get(name)
            .cloned()
            .map_or_else(|| self.parent.get_val(name), Ok)
    }

    pub fn insert_val(&mut self, name: String, val: Expr) -> Expr {
        self.values.insert(name, val.clone());
        val
    }
}

fn install_primitives(env: EnvRef) -> EnvRef {
    let primitives = [
        ("define", special_form::define as PSig),
        ("lambda", special_form::lambda as PSig),
        ("if", special_form::if_statement as PSig),
        ("+", numeric::add as PSig),
        ("-", numeric::subtract as PSig),
        ("*", numeric::multiply as PSig),
        ("/", numeric::divide as PSig),
        ("=", numeric::equality as PSig),
        (">", numeric::greater_than as PSig),
        (">=", numeric::greater_than_or_eq as PSig),
        ("<", numeric::less_than as PSig),
        ("<=", numeric::less_than_or_eq as PSig),
        ("cons", pair::cons as PSig),
        ("car", pair::car as PSig),
        ("cdr", pair::cdr as PSig),
        ("list", pair::list as PSig),
        ("null?", pair::null_check as PSig),
        ("pair?", pair::pair_check as PSig),
    ];

    for (name, proc) in primitives.into_iter() {
        env.insert_val(name.to_string(), Primitive::new(proc).to_expr())
            .unwrap_or_else(|err| panic!("unable to initalize global enviroment. {err}"));
    }

    env
}
