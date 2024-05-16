use crate::{
    enviroment::EnvRef,
    error::EvalErr,
    evaluator::eval,
    lexer::Token,
    parser::Expr,
    procedure::Compound,
    utils::{IterInnerVal, ToExpr},
};

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialForm {
    And(And),
    Assignment(Assignment),
    Begin(Begin),
    Define(Define),
    If(If),
    Lambda(Lambda),
    Or(Or),
}

pub trait Eval {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr>;
}

impl Eval for SpecialForm {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match self {
            SpecialForm::And(and_x) => and_x.eval(env),
            SpecialForm::Assignment(ass_x) => ass_x.eval(env),
            SpecialForm::Begin(beg_x) => beg_x.eval(env),
            SpecialForm::Define(def_x) => def_x.eval(env),
            SpecialForm::If(if_x) => if_x.eval(env),
            SpecialForm::Lambda(lam_x) => lam_x.eval(env),
            SpecialForm::Or(or_x) => or_x.eval(env),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Define {
    identifier: Expr,
    body: Expr,
}

impl Define {
    pub fn new(identifier: Expr, body: Expr) -> Self {
        Define { identifier, body }
    }
}

impl Eval for Define {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match &self.identifier {
            Expr::Atom(Token::Symbol(identifier)) => {
                env.insert_val(identifier.clone(), eval(&self.body, env)?)?;
                Ok(Expr::Void)
            }
            identifier => Err(EvalErr::TypeError("symbol or list", identifier.clone())),
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

impl Eval for Lambda {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match &self.params {
            Expr::Call(first_expr) => {
                let proc_args = first_expr.to_owned().into_strings()?;
                Ok(Compound::new(self.body.to_owned(), proc_args, env.clone_rc()?).to_expr())
            }
            Expr::EmptyList => {
                Ok(Compound::new(self.body.to_owned(), vec![], env.clone_rc()?).to_expr())
            }
            first_expr => Err(EvalErr::TypeError("list", first_expr.clone())),
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

impl Eval for If {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match eval(&self.predicate, env)? {
            Expr::Atom(Token::Boolean(true)) => eval(&self.consequence, env),
            Expr::Atom(Token::Boolean(false)) => eval(&self.alternative, env),
            pred => Err(EvalErr::TypeError("bool", pred)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    identifier: Expr,
    value: Expr,
}

impl Assignment {
    pub fn new(identifier: Expr, value: Expr) -> Self {
        Assignment { identifier, value }
    }
}

impl Eval for Assignment {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match &self.identifier {
            Expr::Atom(Token::Symbol(identifier)) => {
                env.update_val(identifier.to_string(), eval(&self.value, env)?)
            }
            expr => Err(EvalErr::TypeError("symbol", expr.clone())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Begin {
    exprs: Vec<Expr>,
}

impl Begin {
    pub fn new(exprs: Vec<Expr>) -> Self {
        Begin { exprs }
    }
}

impl Eval for Begin {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        self.exprs
            .iter()
            .try_fold(Expr::Void, |_returned_expr, expr| eval(expr, env))
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

impl Eval for And {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        for expr in self.body.iter() {
            match eval(expr, env)? {
                Expr::Atom(Token::Boolean(n)) if !n => return Ok(n.to_expr()),
                Expr::Atom(Token::Boolean(n)) if n => Ok(()),
                expr => Err(EvalErr::TypeError("boolean", expr)),
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

impl Eval for Or {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        for expr in self.body.iter() {
            match eval(expr, env)? {
                Expr::Atom(Token::Boolean(n)) if n => return Ok(n.to_expr()),
                Expr::Atom(Token::Boolean(n)) if !n => Ok(()),
                expr => Err(EvalErr::TypeError("boolean", expr)),
            }?;
        }
        Ok(false.to_expr())
    }
}
