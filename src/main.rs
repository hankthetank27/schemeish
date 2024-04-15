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
        match evaluator::eval(exp, &global) {
            Ok(evalulated) => {
                if let Expr::Proc(p) = evalulated {
                    println!("{:?}", p.printable())
                } else {
                    println!("{:?}", evalulated)
                }
            }
            Err(err) => eprintln!("{err}"),
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
    use schemeish::{
        error::EvalErr,
        lexer::Token::Number,
        parser::Expr::{Atom, Dotted, EmptyList},
        primitives::list::Pair,
    };

    use super::*;

    fn eval_test(scm: &str) -> Vec<Expr> {
        let tokens = tokenize(scm);
        let exprs = parse(tokens);
        let global = EnvRef::global();
        exprs
            .into_iter()
            .map(|e| evaluator::eval(e, &global).unwrap_or_else(|err| panic!("{err}")))
            .collect()
    }

    fn eval_err_test(scm: &str) -> Vec<Result<Expr, EvalErr>> {
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
        assert_eq!(res, Atom(Number(27.0)));
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
        assert_eq!(res1, Atom(Number(20.0)));
        assert_eq!(res2, Atom(Number(10.0)));
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
        assert_eq!(res, Atom(Number(7.0)));
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
        assert_eq!(res, Atom(Number(14.0)));
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
        assert_eq!(res, Atom(Number(3628800.0)));
    }

    #[test]
    fn iter_list() {
        let scm = "
            (define (last-pair list)
              (if (null? (cdr list))
                  (car list)
                  (last-pair (cdr list))))
            (define ls (cons 23 (cons 72 (cons 149 (nil)))))
            (last-pair ls)";

        let evalulated = eval_test(scm);
        let res = evalulated.get(2).unwrap().to_owned();
        assert_eq!(res, Atom(Number(149.0)));
    }

    #[test]
    fn map_list() {
        let scm = "
            (define (map ls fn)
              (if (null? ls)
                (nil)
                (cons (fn (car ls))
                      (map (cdr ls) fn))))

            (define ls (cons 1 (cons 2 (cons 3 (nil)))))
            (map ls (lambda (x) (* x 2)))";

        let evalulated = eval_test(scm);
        let res = evalulated.get(2).unwrap().to_owned();
        assert_eq!(
            res,
            Dotted(Pair {
                car: Box::new(Atom(Number(2.0))),
                cdr: Box::new(Dotted(Pair {
                    car: Box::new(Atom(Number(4.0))),
                    cdr: Box::new(Dotted(Pair {
                        car: Box::new(Atom(Number(6.0))),
                        cdr: Box::new(EmptyList)
                    }))
                }))
            })
        )
    }

    #[test]
    fn type_error() {
        let scm = "(define 1 2)";

        let evalulated = eval_err_test(scm);
        let res = evalulated.get(0).unwrap().to_owned();
        match res {
            Err(e) => {
                let x = match e {
                    EvalErr::TypeError(_) => true,
                    _ => false,
                };
                assert!(x)
            }
            Ok(e) => panic!("Expected error, got {:?}", e),
        }
    }

    #[test]
    fn read_file() {
        let mut path = vec!["".to_string(), "./test_scm/factorial.scm".to_string()].into_iter();
        let scm = read(&mut path).unwrap();
        let evalulated = eval_test(&scm);
        let res = evalulated.get(2).unwrap().to_owned();
        assert_eq!(res, Atom(Number(3628800.0)));
    }
}
