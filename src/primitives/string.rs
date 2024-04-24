use crate::{
    error::EvalErr,
    evaluator::Args,
    lexer::Token,
    parser::Expr,
    utils::{GetVals, ToExpr},
};

pub fn equal(args: Args) -> Result<Expr, EvalErr> {
    let first_two = args
        .eval()?
        .into_iter()
        .get_two_or_else(|| EvalErr::InvalidArgs("'equal?'. expected two arguments."))?;

    match first_two {
        (
            Expr::Atom(Token::Str(x) | Token::Symbol(x)),
            Expr::Atom(Token::Str(y) | Token::Symbol(y)),
        ) => Ok((x == y).to_expr()),
        _ => Err(EvalErr::InvalidArgs("expected strings")),
    }
}
