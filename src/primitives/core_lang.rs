use crate::{error::EvalErr, evaluator, parser::Expr, procedure::Proc, utils::GetVals};

use super::pair::own_rc_pair;

pub fn apply(args: evaluator::Args) -> Result<Expr, EvalErr> {
    let env = args.env()?;
    let (op, args) = args.into_iter().get_two_or_else(|| {
        EvalErr::InvalidArgs("'apply'. expected operation and list of arguments")
    })?;

    let args = match args {
        Expr::List(ls) => Ok(ls),
        Expr::Dotted(p) => Ok(own_rc_pair(p).into_iter().collect()),
        _ => Err(EvalErr::InvalidArgs(
            "'apply'. expected list as second argument",
        )),
    }?;

    let args = evaluator::Args::new(args, &env)?;

    match op {
        Expr::Proc(proc) => match *proc {
            Proc::Primitive(proc) => proc.call(args.eval()?),
            Proc::Compound(proc) => proc.call(args.eval()?),
        },
        op => Err(EvalErr::TypeError("procedure", op)),
    }
}
