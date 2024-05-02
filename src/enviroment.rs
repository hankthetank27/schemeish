use core::cell::Ref;
use core::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::EvalErr;
use crate::parser::Expr;
use crate::primitives::{io, numeric, pair, string};
use crate::procedure::{PSig, Primitive};
use crate::utils::ToExpr;

type RcCellEnv = Option<Rc<RefCell<Env>>>;

#[derive(Debug, Clone, PartialEq)]
pub struct EnvRef(RcCellEnv);

impl EnvRef {
    pub fn nil() -> EnvRef {
        EnvRef(None)
    }

    pub fn new(env: Env) -> EnvRef {
        EnvRef(Some(Rc::new(RefCell::new(env))))
    }

    pub fn get_env(&self) -> RcCellEnv {
        Some(Rc::clone(self.0.as_ref()?))
    }

    pub fn global() -> EnvRef {
        let global = EnvRef::new(Env::new(EnvRef::nil()));
        install_primitives(global)
    }

    pub fn clone_rc(&self) -> Result<EnvRef, EvalErr> {
        Ok(EnvRef(Some(Rc::clone(
            self.0.as_ref().ok_or(EvalErr::NilEnv)?,
        ))))
    }

    fn borrow_inner(&self) -> Result<Ref<Env>, EvalErr> {
        Ok(self.0.as_ref().ok_or(EvalErr::NilEnv)?.borrow())
    }

    pub fn get_val(&self, name: &str) -> Result<Expr, EvalErr> {
        self.0
            .as_ref()
            .ok_or_else(|| EvalErr::UnboundVar(name.to_string()))?
            .borrow()
            .get_val(name)
    }

    pub fn insert_val(&self, name: String, val: Expr) -> Result<Expr, EvalErr> {
        Ok(self
            .0
            .as_ref()
            .ok_or(EvalErr::NilEnv)?
            .borrow_mut()
            .insert_val(name, val))
    }

    pub fn update_val(&self, name: String, val: Expr) -> Result<Expr, EvalErr> {
        self.0
            .as_ref()
            .ok_or_else(|| EvalErr::UnboundVar(name.to_string()))?
            .borrow_mut()
            .update_val(name, val)
    }

    pub fn lookup_raw_pointer(&self, name: String) -> Result<*mut Expr, EvalErr> {
        match self
            .borrow_inner()
            .map_err(|_| EvalErr::UnboundVar(name.to_string()))?
            .values
            .get(&name)
        {
            Some(v) => Ok(v as *const Expr as *mut Expr),
            None => self.borrow_inner()?.parent.lookup_raw_pointer(name),
        }
    }

    // pub fn find_enclosing_env(&self, name: String) -> Result<Rc<RefCell<Env>>, EvalErr> {
    //     let rc = self
    //         .clone_inner()
    //         .map_err(|_| EvalErr::UnboundVar(name.to_string()))?;

    //     match rc.borrow().values.get(&name) {
    //         Some(_) => Ok(rc),
    //         None => self.borrow_inner()?.parent.find_enclosing_env(name),
    //     }
    // }

    //pub fn mutate_list(&self, name: String, value: *mut Expr) -> Result<Expr, EvalErr> {
    //    match self
    //        .borrow_inner_mut()
    //        .map_err(|_| EvalErr::UnboundVar(name.to_string()))?
    //        .values
    //        .get_mut(&name)
    //    {
    //        Some(v) => match v {
    //            Expr::Dotted(p) => unsafe {
    //                p.cdr = *value;
    //                Ok(p.clone().to_expr())
    //            },
    //            //type error
    //            _ => Err(EvalErr::UnboundVar(name.to_string())),
    //        },
    //        None => self.borrow_inner_mut()?.parent.mutate_list(name, value),
    //    }
    //}
    // pub fn get_ref_val(&self, name: &str) -> Result<&Expr, EvalErr> {
    //     self.0
    //         .as_ref()
    //         .ok_or_else(|| EvalErr::UnboundVar(name.to_string()))?
    //         .borrow()
    //         .get_ref_val(name)
    // }
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

    pub fn update_val(&mut self, name: String, val: Expr) -> Result<Expr, EvalErr> {
        match self.values.get_mut(&name) {
            Some(entry) => {
                *entry = val;
                self.get_val(&name)
            }
            None => self.parent.update_val(name, val),
        }
    }
}

fn install_primitives(env: EnvRef) -> EnvRef {
    let primitives = [
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
        ("display", io::display as PSig),
        ("equal?", string::equal as PSig),
    ];

    for (name, proc) in primitives.into_iter() {
        env.insert_val(name.to_string(), Primitive::new(proc).to_expr())
            .unwrap_or_else(|err| panic!("unable to initalize global enviroment. {err}"));
    }

    env
}
