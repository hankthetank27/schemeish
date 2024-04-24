use std::env;
use std::error::Error;
use std::fs;
use std::process;

use schemeish::enviroment::EnvRef;
use schemeish::evaluator;
use schemeish::lexer::TokenStream;
use schemeish::parser::Parser;
use schemeish::repl::Repl;

enum Runtime {
    File(String),
    Repl,
}

fn main() {
    let mut args = env::args();

    let runtime = read(&mut args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    match runtime {
        Runtime::File(f) => run_from_file(&f),
        Runtime::Repl => Repl::new().run(),
    }
}

fn run_from_file(file: &str) {
    let exprs = Parser::new(TokenStream::new(file))
        .parse()
        .unwrap_or_else(|err| {
            eprintln!("{err}");
            process::exit(1);
        });

    let global = EnvRef::global();
    for exp in exprs.into_iter() {
        match evaluator::eval(exp, &global) {
            Ok(_) => (),
            Err(err) => eprintln!("{err}"),
        }
    }
}

fn read<T>(args: &mut T) -> Result<Runtime, Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    args.next();

    let Some(path) = args.next() else {
        return Ok(Runtime::Repl);
    };

    Ok(Runtime::File(fs::read_to_string(path)?))
}

#[cfg(test)]
mod test {
    use core::panic;

    use schemeish::{
        error::EvalErr,
        lexer::Token::Number,
        parser::Expr,
        parser::Expr::{Atom, Dotted, EmptyList},
        primitives::pair::Pair,
    };

    use super::*;

    fn eval_test(scm: &str) -> Vec<Expr> {
        let exprs = Parser::new(TokenStream::new(scm)).parse().unwrap();
        let global = EnvRef::global();
        exprs
            .into_iter()
            .map(|e| evaluator::eval(e, &global).unwrap_or_else(|err| panic!("{err}")))
            .collect()
    }

    fn eval_err_test(scm: &str) -> Vec<Result<Expr, EvalErr>> {
        let exprs = Parser::new(TokenStream::new(scm)).parse().unwrap();
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
    fn curry_and_comment() {
        let scm = "
            (((lambda (x) ;this has a comment
                (lambda (y)
                  (+ x y)))
              3)
             4) ";

        let evalulated = eval_test(scm);
        let res = evalulated.get(0).unwrap().to_owned();
        assert_eq!(res, Atom(Number(7.0)));
    }

    #[test]
    fn nested_proc_w_comments() {
        let scm = "
            ; comment much?
            (define (add-with-ten x y) ; yoo
                ; we got a comment here mate
              (define (add b c) (+ 10 b c))
              (+ (add x x) (add y y)));morestuff
;commmentttimte!;haha
            (- (add-with-ten 1 1) 5 5); also end w comment";

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
            (define ls (cons 23 (cons 72 (cons 149 ()))))
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
                ()
                (cons (fn (car ls))
                      (map (cdr ls) fn))))

            (define ls (list 1 2 (+ 1 2)))
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
    fn reassign_var() {
        let scm = "
            (define x 2)
            (define (new-scope depth) 
                (if (= 0 depth) 
                    (set! x 1) 
                    (new-scope (- depth 1))))
            (new-scope 5)
            x";

        let evalulated = eval_test(scm);
        let res = evalulated.get(3).unwrap().to_owned();
        assert_eq!(res, Atom(Number(1.0)));
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
    fn reassign_unassigned() {
        let scm = "
            (define (new-scope) (set! x 1))
            (new-scope)
            x";

        let evalulated = eval_err_test(scm);
        let res = evalulated.get(2).unwrap().to_owned();
        match res {
            Err(e) => {
                let x = match e {
                    EvalErr::UnboundVar(_) => true,
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
        let scm = match read(&mut path).unwrap() {
            Runtime::File(f) => f,
            Runtime::Repl => panic!("expected file"),
        };
        let evalulated = eval_test(&scm);
        let res = evalulated.get(3).unwrap().to_owned();
        assert_eq!(res, Atom(Number(3628800.0)));
    }
}
