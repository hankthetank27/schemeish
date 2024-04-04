use crate::{enviroment::EnvRef, evaluator, lexer::Token, parser::Expr};

pub fn add(args: &Vec<Expr>, env: EnvRef) -> Expr {
    evaluator::eval_list(args, env)
        .to_nums()
        .sum::<f64>()
        .to_num_expr()
}

pub fn subtract(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let nums: Vec<f64> = evaluator::eval_list(args, env).to_nums().collect();
    nums.iter()
        .skip(1)
        .fold(nums[0], |diff, num| diff - num)
        .to_num_expr()
}

trait Collect {
    fn to_nums(&self) -> impl Iterator<Item = f64>;
}

impl Collect for Vec<Expr> {
    fn to_nums(&self) -> impl Iterator<Item = f64> {
        self.iter().map(|expr| match expr {
            Expr::Atom(Token::Number(n)) => *n,
            _ => panic!("attempted to perform arithmetic on a non-number value"),
        })
    }
}

trait ToExpr {
    fn to_num_expr(self) -> Expr;
}

impl ToExpr for f64 {
    fn to_num_expr(self) -> Expr {
        Expr::Atom(Token::Number(self))
    }
}
