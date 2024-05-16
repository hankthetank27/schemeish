use std::iter::Peekable;

use crate::{
    enviroment::EnvRef,
    error::EvalErr,
    evaluator::eval,
    lexer::Token,
    parser::Expr,
    print::Printable,
    procedure::Compound,
    utils::{IterInnerVal, OwnIterVals, ToExpr},
};

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialForm {
    And(And),
    Assignment(Assignment),
    Begin(Begin),
    Cond(Cond),
    Define(Define),
    If(If),
    Lambda(Lambda),
    Let(Let),
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
            SpecialForm::Cond(cond_x) => cond_x.eval(env),
            SpecialForm::Define(def_x) => def_x.eval(env),
            SpecialForm::If(if_x) => if_x.eval(env),
            SpecialForm::Lambda(lam_x) => lam_x.eval(env),
            SpecialForm::Let(let_x) => let_x.eval(env),
            SpecialForm::Or(or_x) => or_x.eval(env),
        }
    }
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

impl Eval for Define {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match &self.identifier {
            //bind var
            Expr::Atom(Token::Symbol(identifier)) => {
                let value = self
                    .body
                    .iter()
                    .next()
                    .ok_or(EvalErr::InvalidArgs("variable has no declared value"))?;

                env.insert_val(identifier.to_string(), eval(&value, env)?)?;
                Ok(Expr::Void)
            }
            //bind proc
            Expr::Call(args) => {
                let (first, rest) = args.into_iter().own_one_and_rest_or_else(|| {
                    EvalErr::InvalidArgs("'define' procedure. expected parameters and body")
                })?;

                match first {
                    Expr::Atom(Token::Symbol(identifier)) => {
                        let proc = Lambda::new(Expr::Call(rest.collect()), self.body.to_owned())
                            .eval(env)?;
                        env.insert_val(identifier.to_string(), proc)?;
                        Ok(Expr::Void)
                    }
                    _ => Err(EvalErr::InvalidArgs("expected identifier for procedure")),
                }
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
pub struct Cond {
    clauses: Vec<Expr>,
}

impl Cond {
    pub fn new(clauses: Vec<Expr>) -> Self {
        Cond { clauses }
    }
}

impl Eval for Cond {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        eval(&cond_to_if(&mut self.clauses.iter().peekable())?, env)
    }
}

fn cond_to_if(exprs: &mut Peekable<std::slice::Iter<'_, Expr>>) -> Result<Expr, EvalErr> {
    match exprs.next() {
        Some(expr) => match expr {
            Expr::Call(expr) => {
                let (predicate, consequence) = expr.into_iter().own_one_and_rest_or_else(|| {
                    EvalErr::InvalidArgs("'cond'. clauses expcted two be lists of two values")
                })?;

                let consequence = Begin::new(consequence.collect()).to_expr().into_call()?;

                if exprs.peek().is_some() {
                    If::new(predicate, consequence, cond_to_if(exprs)?)
                        .to_expr()
                        .into_call()
                } else {
                    match predicate {
                        Expr::Atom(Token::Else) => Ok(consequence),
                        _ => If::new(predicate, consequence, cond_to_if(exprs)?)
                            .to_expr()
                            .into_call(),
                    }
                }
            }
            expr => Err(EvalErr::UnexpectedToken(expr.printable())),
        },
        None => Ok(Expr::EmptyList),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Let {
    bindings: Expr,
    body: Vec<Expr>,
}

impl Let {
    pub fn new(bindings: Expr, body: Vec<Expr>) -> Self {
        Let { bindings, body }
    }
}

impl Eval for Let {
    fn eval(&self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match &self.bindings {
            Expr::Call(bindings) => {
                let (params, mut values) = try_unzip_list(bindings)?;

                values.insert(
                    0,
                    Lambda::new(params.to_expr(), self.body.to_owned())
                        .to_expr()
                        .into_call()?,
                );

                eval(&values.to_expr(), env)
            }
            expr => Err(EvalErr::TypeError("list", expr.clone())),
        }
    }
}

fn try_unzip_list(exprs: &Vec<Expr>) -> Result<(Vec<Expr>, Vec<Expr>), EvalErr> {
    exprs
        .into_iter()
        .try_fold((vec![], vec![]), |prev, expr_pair| {
            let (mut params, mut values) = prev;
            match expr_pair {
                Expr::Call(binding) => {
                    let (param, value) = binding.into_iter().own_two_or_else(|| {
                        EvalErr::InvalidArgs("'let' expression. expected bindings as pairs")
                    })?;
                    params.push(param);
                    values.push(value);
                    Ok((params, values))
                }
                expr => Err(EvalErr::TypeError("list", expr.clone())),
            }
        })
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
            .try_fold(Expr::Void, |_returned_expr, expr| eval(&expr, env))
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
            match eval(&expr, env)? {
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
            match eval(&expr, env)? {
                Expr::Atom(Token::Boolean(n)) if n => return Ok(n.to_expr()),
                Expr::Atom(Token::Boolean(n)) if !n => Ok(()),
                expr => Err(EvalErr::TypeError("boolean", expr)),
            }?;
        }
        Ok(false.to_expr())
    }
}
