use crate::{
    enviroment::{Env, EnvRef},
    error::EvalErr,
    evaluator::{eval, Args},
    parser::Expr,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Proc {
    Primitive(Primitive),
    Compound(Compound),
}

pub type PSig = fn(Args) -> Result<Expr, EvalErr>;

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive(PSig);

#[allow(clippy::new_ret_no_self)]
impl Primitive {
    pub fn new(proc: PSig) -> Proc {
        Proc::Primitive(Primitive(proc))
    }

    pub fn call(self, args: Args) -> Result<Expr, EvalErr> {
        (self.0)(args)
    }

    pub fn inner(&self) -> PSig {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compound {
    params: Vec<String>,
    body: Vec<Expr>,
    env: EnvRef,
}

#[allow(clippy::new_ret_no_self)]
impl Compound {
    pub fn new(body: Vec<Expr>, params: Vec<String>, env: EnvRef) -> Proc {
        Proc::Compound(Compound { body, params, env })
    }

    pub fn call(self, args: Args) -> Result<Expr, EvalErr> {
        if self.params.len() != args.len() {
            return Err(EvalErr::InvalidArgs(
                "amount of args does not match function pararms",
            ));
        }

        let mut args = args.into_iter();
        let mut new_env = Env::new(self.env.clone_rc());

        for param in self.params.iter() {
            let arg = args.next().unwrap();
            new_env.insert_val(param.to_string(), arg.clone());
        }

        let new_env_ref = EnvRef::new(new_env);

        self.body
            .into_iter()
            .try_fold(Expr::EmptyList, |_returned_expr, expr| {
                eval(expr, &new_env_ref)
            })
    }

    pub fn params(self) -> Vec<String> {
        self.params
    }
}
