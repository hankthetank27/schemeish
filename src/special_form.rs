use crate::{
    enviroment::EnvRef,
    error::EvalErr,
    evaluator::eval,
    lexer::Token,
    parser::Expr,
    procedure::Compound,
    utils::{GetVals, IterInnerVal, ToExpr},
};

pub trait SpecialForm {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Define {
    identifier: Expr,
    body: Vec<Expr>,
}

impl Define {
    pub fn new(identifier: Expr, body: Vec<Expr>) -> Self {
        Define { identifier, body }
    }
}

impl SpecialForm for Define {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        let mut body = self.body.into_iter();
        match self.identifier {
            //bind var
            Expr::Atom(Token::Symbol(identifier)) => {
                let value = body
                    .next()
                    .ok_or(EvalErr::InvalidArgs("variable has no declared value"))?;

                env.insert_val(identifier.to_string(), eval(value, env)?)
            }
            //bind proc
            Expr::List(args) => {
                let (first, rest) = args.into_iter().get_one_and_rest_or_else(|| {
                    EvalErr::InvalidArgs("'define' procedure. expected parameters and body")
                })?;

                match first {
                    Expr::Atom(Token::Symbol(identifier)) => {
                        let lamba = Lambda::new(Expr::List(rest.collect()), body.collect());
                        env.insert_val(identifier.to_string(), lamba.eval(env)?)
                    }
                    _ => Err(EvalErr::InvalidArgs("expected identifier for procedure")),
                }
            }
            identifier => Err(EvalErr::TypeError(("symbol or list", identifier))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    params: Expr,
    body: Vec<Expr>,
}

impl Lambda {
    pub fn new(params: Expr, body: Vec<Expr>) -> Self {
        Lambda { params, body }
    }
}

impl SpecialForm for Lambda {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match self.params {
            Expr::List(first_expr) => {
                let proc_args = first_expr.into_strings()?;
                Ok(Compound::new(self.body, proc_args, env.clone_rc()).to_expr())
            }
            Expr::EmptyList => Ok(Compound::new(self.body, vec![], env.clone_rc()).to_expr()),
            first_expr => Err(EvalErr::TypeError(("list", first_expr))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    predicate: Expr,
    consequence: Expr,
    alternative: Expr,
}

impl If {
    pub fn new(predicate: Expr, consequence: Expr, alternative: Expr) -> Self {
        If {
            predicate,
            consequence,
            alternative,
        }
    }
}

impl SpecialForm for If {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match eval(self.predicate, env)? {
            Expr::Atom(Token::Boolean(true)) => eval(self.consequence, env),
            Expr::Atom(Token::Boolean(false)) => eval(self.alternative, env),
            pred => Err(EvalErr::TypeError(("bool", pred))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    identifier: Expr,
    body: Vec<Expr>,
}

impl Assignment {
    pub fn new(identifier: Expr, body: Vec<Expr>) -> Self {
        Assignment { identifier, body }
    }
}

impl SpecialForm for Assignment {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        let mut body = self.body.into_iter();
        match self.identifier {
            Expr::Atom(Token::Symbol(identifier)) => {
                let value = body
                    .next()
                    .ok_or(EvalErr::InvalidArgs("variable has no declared value"))?;

                env.update_val(identifier.to_string(), eval(value, env)?)
            }
            x => Err(EvalErr::TypeError(("symbol", x))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct And {
    body: Vec<Expr>,
}

impl And {
    pub fn new(body: Vec<Expr>) -> Self {
        And { body }
    }
}

impl SpecialForm for And {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        for expr in self.body.into_iter() {
            match eval(expr, env)? {
                Expr::Atom(Token::Boolean(n)) => {
                    if !n {
                        return Ok(n.to_expr());
                    } else {
                        Ok(())
                    }
                }
                x => Err(EvalErr::TypeError(("boolean", x))),
            }?;
        }
        Ok(true.to_expr())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Or {
    body: Vec<Expr>,
}

impl Or {
    pub fn new(body: Vec<Expr>) -> Self {
        Or { body }
    }
}

impl SpecialForm for Or {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        for expr in self.body.into_iter() {
            match eval(expr, env)? {
                Expr::Atom(Token::Boolean(n)) => {
                    if n {
                        return Ok(n.to_expr());
                    } else {
                        Ok(())
                    }
                }
                x => Err(EvalErr::TypeError(("boolean", x))),
            }?;
        }
        Ok(false.to_expr())
    }
}
