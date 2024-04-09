use std::env;
use std::error::Error;
use std::fs;
use std::process;

use schemeish::enviroment::EnvRef;
use schemeish::evaluator;
use schemeish::lexer::tokenize;
use schemeish::parser::parse;
use schemeish::parser::Expr;

fn main() {
    let mut args = env::args();

    let file = read(&mut args).unwrap_or_else(|err| {
        eprint!("{err}");
        process::exit(1);
    });

    let tokens = tokenize(&file);
    let exprs = parse(tokens);
    let global = EnvRef::global();
    for exp in exprs.into_iter() {
        let evalulated = evaluator::eval(exp, &global);
        if let Expr::Proc(p) = evalulated {
            println!("{:?}", p.printable())
        } else {
            println!("{:?}", evalulated)
        }
    }
}

fn read<T>(args: &mut T) -> Result<String, Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    args.next();

    let path = match args.next() {
        Some(path) => path,
        None => return Err("Usage: rsscheme file_path.scm".into()),
    };

    Ok(fs::read_to_string(path)?)
}

#[cfg(test)]
mod test {
    use schemeish::{lexer::Token, parser::Expr};

    use super::*;

    fn eval_test(scm: &str) -> Vec<Expr> {
        let tokens = tokenize(scm);
        let exprs = parse(tokens);
        let global = EnvRef::global();
        exprs
            .into_iter()
            .map(|e| evaluator::eval(e, &global))
            .collect()
    }

    #[test]
    fn arithemtic() {
        let scm = "
            (+ 1 (+ (+ 1 2)(- 2 1) 6 7 8 (- 3 2)))";

        let evalulated = eval_test(scm);
        let res = evalulated.get(0).unwrap().to_owned();
        assert_eq!(res, Expr::Atom(Token::Number(27.0)));
    }

    #[test]
    fn if_cmp() {
        let scm = "
            (define (test-if x y)
              (if (= x y)
                10
                20))
            (test-if 1 2)
            (test-if 1 1) ";

        let evalulated = eval_test(scm);
        let res1 = evalulated.get(1).unwrap().to_owned();
        let res2 = evalulated.get(2).unwrap().to_owned();
        assert_eq!(res1, Expr::Atom(Token::Number(20.0)));
        assert_eq!(res2, Expr::Atom(Token::Number(10.0)));
    }

    #[test]
    fn curry() {
        let scm = "
            (((lambda (x)
                (lambda (y)
                  (+ x y)))
              3)
             4) ";

        let evalulated = eval_test(scm);
        let res = evalulated.get(0).unwrap().to_owned();
        assert_eq!(res, Expr::Atom(Token::Number(7.0)));
    }

    #[test]
    fn factorial() {
        let scm = "
            (define (product term a next b)
              (if (> a b)
                  1
                  (* (term a)
                     (product term (next a) next b))))
            (define (factorial x)
              (define (id x) x)
              (define (inc x) (+ x 1))
              (product id 1 inc x))
            (factorial 10) ";

        let evalulated = eval_test(scm);
        let res = evalulated.get(2).unwrap().to_owned();
        assert_eq!(res, Expr::Atom(Token::Number(3628800.0)));
    }

    #[test]
    fn nested_proc() {
        let scm = "
            (define (add-with-ten x y)
              (define (add b c) (+ 10 b c))
              (+ (add x x) (add y y))) 
            (- (add-with-ten 1 1) 5 5)";

        let evalulated = eval_test(scm);
        let res = evalulated.get(1).unwrap().to_owned();
        assert_eq!(res, Expr::Atom(Token::Number(14.0)));
    }

    #[test]
    fn read_file() {
        let mut path = vec!["".to_string(), "./test_scm/factorial.scm".to_string()].into_iter();
        let scm = read(&mut path).unwrap();
        let evalulated = eval_test(&scm);
        let res = evalulated.get(2).unwrap().to_owned();
        assert_eq!(res, Expr::Atom(Token::Number(3628800.0)));
    }
}
