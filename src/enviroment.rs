use core::cell::RefCell;
use core::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::EvalErr;
use crate::evaluator::eval;
use crate::lexer::TokenStream;
use crate::parser::Expr;
use crate::parser::Parser;
use crate::primitives::{compare, core_lang, io, numeric, pair, prelude, typecheck};
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
        EnvRef::new(Env::new(EnvRef::nil())).install_primitives()
    }

    pub fn clone_rc(&self) -> Result<EnvRef, EvalErr> {
        Ok(EnvRef(Some(Rc::clone(
            self.0.as_ref().ok_or(EvalErr::NilEnv)?,
        ))))
    }

    fn borrow_ref(&self) -> Result<Ref<Env>, EvalErr> {
        Ok(self.0.as_ref().ok_or(EvalErr::NilEnv)?.borrow())
    }

    fn borrow_ref_mut(&self) -> Result<RefMut<Env>, EvalErr> {
        Ok(self.0.as_ref().ok_or(EvalErr::NilEnv)?.borrow_mut())
    }

    pub fn get_val(&self, name: &str) -> Result<Expr, EvalErr> {
        self.borrow_ref()
            .map_err(|_| EvalErr::UnboundVar(name.to_string()))?
            .get_val(name)
    }

    pub fn insert_val(&self, name: String, val: Expr) -> Result<(), EvalErr> {
        self.borrow_ref_mut()?.insert_val(name, val);
        Ok(())
    }

    pub fn update_val(&self, name: String, val: Expr) -> Result<Expr, EvalErr> {
        self.borrow_ref_mut()
            .map_err(|_| EvalErr::UnboundVar(name.to_string()))?
            .update_val(name, val)
    }

    fn install_primitives(self) -> EnvRef {
        let primitives = [
            ("apply", core_lang::apply as PSig),
            ("+", numeric::add as PSig),
            ("-", numeric::subtract as PSig),
            ("*", numeric::multiply as PSig),
            ("/", numeric::divide as PSig),
            ("=", numeric::equality as PSig),
            (">", numeric::greater_than as PSig),
            (">=", numeric::greater_than_or_eq as PSig),
            ("<", numeric::less_than as PSig),
            ("<=", numeric::less_than_or_eq as PSig),
            ("remainder", numeric::remainder as PSig),
            ("modulo", numeric::modulo as PSig),
            ("cons", pair::cons as PSig),
            ("car", pair::car as PSig),
            ("cdr", pair::cdr as PSig),
            ("set-car!", pair::set_car as PSig),
            ("set-cdr!", pair::set_cdr as PSig),
            ("list", pair::list as PSig),
            ("display", io::display as PSig),
            ("error", io::error as PSig),
            ("equal?", compare::equal as PSig),
            ("eq?", compare::equal as PSig),
            ("not", compare::not as PSig),
            ("symbol?", typecheck::symbol as PSig),
            ("string?", typecheck::string as PSig),
            ("number?", typecheck::number as PSig),
            ("pair?", typecheck::pair as PSig),
            ("null?", typecheck::null as PSig),
        ];

        for (name, proc) in primitives.into_iter() {
            self.insert_val(name.to_string(), Primitive::new(proc).to_expr())
                .unwrap_or_else(|err| panic!("unable to initalize global enviroment. {err}"));
        }

        self
    }

    pub fn import_prelude(&self) -> Result<(), EvalErr> {
        let tokens = TokenStream::new(prelude::PRELUDE).collect_tokens()?;
        let exprs = Parser::new(tokens).parse()?;
        for exp in exprs.into_iter() {
            match eval(exp, self) {
                Ok(_) => (),
                Err(err) => eprintln!("{err}"),
            }
        }
        Ok(())
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

    pub fn insert_val(&mut self, name: String, val: Expr) {
        self.values.insert(name, val);
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
