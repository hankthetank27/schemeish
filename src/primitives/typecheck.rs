use crate::{
    error::EvalErr,
    evaluator::Args,
    lexer::Token,
    parser::Expr,
    utils::{OwnIterVals, ToExpr},
};

pub fn symbol(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .own_one_or_else(|| EvalErr::InvalidArgs("'symbol?' expected argument"))?
    {
        Expr::Atom(Token::Symbol(_)) => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn string(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .own_one_or_else(|| EvalErr::InvalidArgs("'string?' expected argument"))?
    {
        Expr::Atom(Token::Str(_)) => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn number(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .own_one_or_else(|| EvalErr::InvalidArgs("'number?' expected argument"))?
    {
        Expr::Atom(Token::Number(_)) => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn null(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .own_one_or_else(|| EvalErr::InvalidArgs("'null?'. expected argument"))?
    {
        Expr::EmptyList => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn pair(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .own_one_or_else(|| EvalErr::InvalidArgs("'pair?'. expected argument"))?
    {
        Expr::Pair(_) => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}
