use crate::{
    enviroment::{Env, EnvRef},
    evaluator::eval,
    parser::Expr,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Proc {
    params: Vec<String>,
    body: Vec<Expr>,
    env: EnvRef,
}

// we can probably make a printing module
#[derive(Debug)]
#[allow(dead_code)]
pub struct PrintProc {
    params: Vec<String>,
    body: Vec<Expr>,
}

impl Proc {
    pub fn new(body: Vec<Expr>, params: Vec<String>, env: EnvRef) -> Proc {
        Proc { body, params, env }
    }

    pub fn call(&self, args: Vec<Expr>) -> Expr {
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
            .iter()
            .fold(None, |_returned_expr, expr| {
                Some(eval(expr, new_env_ref.clone_rc()))
            })
            .unwrap()
        //return None (undefined) is the empty list??
    }

    pub fn printable(&self) -> PrintProc {
        PrintProc {
            params: self.params.clone(),
            body: self.body.clone(),
        }
    }
}
