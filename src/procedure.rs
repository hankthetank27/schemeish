use crate::{enviroment::Env, parser::Expr};

#[derive(Debug, Clone, PartialEq)]
pub struct Proc {
    params: Vec<String>,
    body: Vec<Expr>,
    // we point to parent env where proc has been defined
    env: Env,
}

impl Proc {
    pub fn new(body: Vec<Expr>, params: Vec<String>, env: Env) -> Self {
        Proc { body, params, env }
    }
}
