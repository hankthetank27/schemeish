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

const SCM: &'static str = "(define (first-denomination coins)
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
                    (cc 200 us-coins)";

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
    c.bench_function("coin combo 200", |b| b.iter(|| eval_test(black_box(SCM))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
