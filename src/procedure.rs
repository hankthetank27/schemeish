use crate::{
    enviroment::{Env, EnvRef},
    evaluator::eval,
    parser::Expr,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct Proc {
    params: Vec<String>,
    body: Vec<Expr>,
    env: EnvRef,
}

impl Proc {
    pub fn new(body: Vec<Expr>, params: Vec<String>, env: EnvRef) -> Proc {
        Proc { body, params, env }
    }

    pub fn call(&self, args: Vec<Expr>) -> Expr {
        let mut args = args.into_iter();
        let mut new_env = Env::new(Rc::clone(&self.env));

        for param in self.params.iter() {
            let arg = args
                .next()
                .expect("Amount of args does not match function pararms");
            new_env.insert_val(param.to_string(), arg.clone());
        }

        let new_env_ref = Rc::new(RefCell::new(Some(new_env)));
        self.body
            .iter()
            .fold(None, |_returned_expr, expr| {
                // TODO this call to eval is producing infinite recussion hmmm
                println!("{:?}", expr);
                Some(eval(expr, Rc::clone(&new_env_ref)))
            })
            .unwrap()
    }
}
