use crate::{error::EvalErr, evaluator::Args, parser::Expr, print::Print, utils::GetVals};

pub fn display(args: Args) -> Result<Expr, EvalErr> {
    let expr = args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'display'. expected argument"))?;
    expr.print();
    Ok(expr)
}
