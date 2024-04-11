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

// we can probably make a printing module
// #[derive(Debug)]
// #[allow(dead_code)]
// pub struct PrintProc {
//     params: Vec<String>,
//     body: Vec<Expr>,
// }

impl Proc {
    pub fn printable(&self) -> &str {
        match self {
            Proc::Primitive(_) => "Def Primitive",
            Proc::Compound(_) => "Def Compound",
        }
    }
}

pub type PSig = fn(Args) -> Result<Expr, EvalErr>;

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive(PSig);

impl Primitive {
    pub fn new(proc: PSig) -> Proc {
        Proc::Primitive(Primitive(proc))
    }

    pub fn call(self, args: Args) -> Result<Expr, EvalErr> {
        (self.0)(args)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compound {
    params: Vec<String>,
    body: Vec<Expr>,
    env: EnvRef,
}

impl Compound {
    pub fn new(body: Vec<Expr>, params: Vec<String>, env: EnvRef) -> Proc {
        Proc::Compound(Compound { body, params, env })
    }

    pub fn call(self, args: Vec<Expr>) -> Result<Expr, EvalErr> {
        let mut args = args.into_iter();
        let mut new_env = Env::new(self.env.clone_rc());

        for param in self.params.iter() {
            let arg = args
                .next()
                .expect("Amount of args does not match function pararms");
            new_env.insert_val(param.to_string(), arg.clone());
        }

        let new_env_ref = EnvRef::new(new_env);

        self.body
            .into_iter()
            .fold(None, |_returned_expr, expr| Some(eval(expr, &new_env_ref)))
            .unwrap()
        //return None (undefined) is the empty list??
    }
}
