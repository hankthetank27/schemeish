use crate::enviroment::EnvRef;
use crate::error::EvalErr;
use crate::lexer::Token;
use crate::parser::Expr;
use crate::primitives::utils::GetVals;
use crate::procedure::Proc;

pub fn eval(expr: Expr, env: &EnvRef) -> Result<Expr, EvalErr> {
    match expr {
        // variable lookup
        Expr::Atom(Token::Symbol(ref identifier)) => env.get_val(identifier),
        // self evaluating
        x @ Expr::Atom(_) | x @ Expr::Proc(_) | x @ Expr::Dotted(_) | x @ Expr::EmptyList => Ok(x),
        // procedure
        Expr::List(ls) => {
            let (op, args) = ls.into_iter().get_one_and_rest()?;
            let args = Args::new(args.collect(), env);
            match op {
                Expr::Atom(Token::Symbol(_)) => apply(op, args),
                Expr::List(_) => apply(op, args),
                op => Err(EvalErr::TypeError(("symbol or list", op))),
            }
        }
    }
}

pub fn apply(op: Expr, args: Args) -> Result<Expr, EvalErr> {
    match eval(op, &args.env())? {
        Expr::Proc(proc) => match proc {
            Proc::Primitive(proc) => proc.call(args),
            Proc::Compound(proc) => proc.call(args.eval()?),
        },
        op => Err(EvalErr::TypeError(("procedure", op))),
    }
}

pub struct Args {
    args: Vec<Expr>,
    env: EnvRef,
}

impl Args {
    pub fn new(args: Vec<Expr>, env: &EnvRef) -> Args {
        Args {
            args,
            env: env.clone_rc(),
        }
    }

    pub fn eval(self) -> Result<Vec<Expr>, EvalErr> {
        self.args
            .into_iter()
            .map(|expr| eval(expr, &self.env))
            .collect()
    }

    pub fn env(&self) -> EnvRef {
        self.env.clone_rc()
    }
}

impl IntoIterator for Args {
    type Item = Expr;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}
