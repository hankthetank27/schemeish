use criterion::{black_box, criterion_group, criterion_main, Criterion};
use schemeish::{
    enviroment::EnvRef,
    evaluator,
    lexer::TokenStream,
    parser::{
        Expr::{self},
        Parser,
    },
};

const TAK: &'static str = "
(define (tak x y z)
  (if (not (< y x))
      z
      (tak (tak (- x 1) y z)
           (tak (- y 1) z x)
           (tak (- z 1) x y))))
(tak 18 12 6)
";

const TAKL: &'static str = " 
(define (listn n)
  (if (= n 0)
    '()
    (cons n (listn (- n 1)))))
 
(define l18 (listn 18))
(define l12 (listn 12))
(define  l6 (listn 6))
 
(define (mas x y z)
  (if (not (shorterp y x))
      z
      (mas (mas (cdr x) y z)
           (mas (cdr y) z x)
           (mas (cdr z) x y))))
 
(define (shorterp x y)
  (and (not (null? y))
       (or (null? x)
           (shorterp (cdr x)
                     (cdr y)))))
(mas 
    (list 1 2 3 4 5 6 7 9 10 11 12 13 14 15 16 17 18)
    (list 1 2 3 4 5 6 7 9 10 11 12)
    (list 1 2 3 4 5 6))
";

const CPSTAK: &'static str = " 
(define (cpstak x y z)

  (define (tak x y z k)
    (if (not (< y x))
        (k z)
        (tak (- x 1)
             y
             z
             (lambda (v1)
               (tak (- y 1)
                    z
                    x
                    (lambda (v2)
                      (tak (- z 1)
                           x
                           y
                           (lambda (v3)
                             (tak v1 v2 v3 k)))))))))

  (tak x y z (lambda (a) a)))
(cpstak 12 6 3)";

const FIB: &'static str = " 
(define (fib n) 
  (if (<= n 2) 
    1 
    (+ (fib (- n 1)) (fib (- n 2)))))

(fib 25)
";

const COIN_COMBO: &'static str = " 
(define (first-denomination coins)
      (car coins))
    (define (except-first-denomination coins)
      (cdr coins))
    (define (no-more? coins)
      (null? coins))

    (define (cc amount coin-values)
      (cond ((= amount 0) 1)
            ((or (< amount 0) (no-more? coin-values)) 0)
            (else
             (+ (cc amount
                    (except-first-denomination coin-values))
                (cc (- amount
                       (first-denomination coin-values))
                    coin-values)))))

(define us-coins (list 25 10 5 1))
(cc 150 us-coins)
";

pub fn eval_test(scm: &str) -> Vec<Expr> {
    let exprs = Parser::new(TokenStream::new(scm).collect_tokens().unwrap())
        .parse()
        .unwrap();
    let global = EnvRef::global();
    global.import_prelude().unwrap();
    exprs
        .into_iter()
        .map(|e| evaluator::eval(e, &global).unwrap_or_else(|err| panic!("{err}")))
        .collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tak 18 12 6", |b| b.iter(|| eval_test(black_box(TAK))));

    c.bench_function("takl 18 12 6", |b| b.iter(|| eval_test(black_box(TAKL))));

    c.bench_function("cpstak 12 6 3", |b| b.iter(|| eval_test(black_box(CPSTAK))));

    c.bench_function("fib 25", |b| b.iter(|| eval_test(black_box(FIB))));

    c.bench_function("coin combo 150", |b| {
        b.iter(|| eval_test(black_box(COIN_COMBO)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
