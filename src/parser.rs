use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;

use crate::error::EvalErr;
use crate::lexer::Token;
use crate::primitives::pair::Pair;
use crate::print::Printable;
use crate::procedure::Proc;
use crate::special_form::{And, Assignment, Begin, Define, If, Lambda, Or, SpecialForm};
use crate::utils::{OwnIterVals, ToExpr};

// We treat any list that is expected to be evaluated as a procedure during parsing as a vector
// of expressions rather than a proper list of pairs to simplify and reduce the cost of the parsing process.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Call(Vec<Expr>),
    Pair(Rc<Pair>), //TODO: Maybe Rc -> Rc<RefCell>? unsafe mutation seems to be ok for now...
    Proc(Rc<Proc>),
    SpecialForm(Rc<SpecialForm>),
    Quoted(Box<Expr>),
    Atom(Token),
    EmptyList,
    Void,
}

impl Expr {
    pub fn into_call(self) -> Result<Expr, EvalErr> {
        Ok(vec![self].to_expr())
    }
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    parsed_exprs: Vec<Expr>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            parsed_exprs: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Expr>, EvalErr> {
        while self.tokens.peek().is_some() {
            let expr = self.parse_from_token()?;
            self.parsed_exprs.push(expr)
        }
        Ok(self.parsed_exprs)
    }

    fn parse_from_token(&mut self) -> Result<Expr, EvalErr> {
        match self.next_or_err(EvalErr::UnexpectedEnd)? {
            Token::LParen => match self.peek_or_err(EvalErr::UnexpectedEnd)? {
                Token::If => {
                    self.tokens.next();
                    self.parse_if()
                }
                Token::Lambda => {
                    self.tokens.next();
                    self.parse_lambda()
                }
                Token::Define => {
                    self.tokens.next();
                    self.parse_define()
                }
                Token::And => {
                    self.tokens.next();
                    self.parse_and()
                }
                Token::Or => {
                    self.tokens.next();
                    self.parse_or()
                }
                Token::Assignment => {
                    self.tokens.next();
                    self.parse_assignment()
                }
                Token::Begin => {
                    self.tokens.next();
                    self.parse_begin()
                }
                Token::Cond => {
                    self.tokens.next();
                    self.parse_cond()
                }
                Token::Let => {
                    self.tokens.next();
                    self.parse_let()
                }
                Token::LetStar => {
                    self.tokens.next();
                    self.parse_letstar()
                }
                Token::QuoteProc => {
                    self.tokens.next();
                    let quoted = self.parse_quote()?;
                    self.next_or_err(EvalErr::UnexpectedEnd)?; // consume remaining paren
                    Ok(Expr::Quoted(Box::new(quoted)))
                }
                _ => self.parse_proc_call(),
            },
            Token::QuoteTick => Ok(Expr::Quoted(Box::new(self.parse_quote()?))),
            x @ Token::Number(_)
            | x @ Token::Str(_)
            | x @ Token::Boolean(_)
            | x @ Token::Symbol(_) => Ok(Expr::Atom(x)),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_proc_call(&mut self) -> Result<Expr, EvalErr> {
        let list = self.parse_inner_call()?;
        match !list.is_empty() {
            true => Ok(list.to_expr()),
            false => Ok(Expr::EmptyList),
        }
    }

    fn parse_inner_call(&mut self) -> Result<Vec<Expr>, EvalErr> {
        let mut parsed_exprs: Vec<Expr> = vec![];
        while let Some(t) = self.tokens.peek() {
            match t {
                Token::RParen => {
                    self.tokens.next();
                    return Ok(parsed_exprs);
                }
                _ => parsed_exprs.push(self.parse_from_token()?),
            }
        }
        Err(EvalErr::UnexpectedEnd)
    }

    fn parse_if(&mut self) -> Result<Expr, EvalErr> {
        let (predicate, consequence, alternative) =
            self.parse_inner_call()?.into_iter().own_three_or_else(|| {
                EvalErr::InvalidArgs(
                    "'if' expression. expected condition, predicate, and consequence",
                )
            })?;
        If::new(predicate, consequence, alternative)
            .to_expr()
            .into_call()
    }

    fn parse_cond(&mut self) -> Result<Expr, EvalErr> {
        let clauses = self.parse_inner_call()?;
        match !clauses.is_empty() {
            true => cond_to_if(&mut clauses.into_iter().peekable()),
            false => Err(EvalErr::InvalidArgs("'cond' expression. expected clauses.")),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, EvalErr> {
        let (params, body) = self
            .parse_inner_call()?
            .into_iter()
            .own_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'lambda' expression. expected parameters and body")
            })?;
        Lambda::new(params, body.collect()).to_expr().into_call()
    }

    fn parse_define(&mut self) -> Result<Expr, EvalErr> {
        let (identifier, mut body) = self
            .parse_inner_call()?
            .into_iter()
            .own_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'define' expression. expected identifier and value")
            })?;

        match identifier {
            Expr::Call(args) => {
                let (identifier, params) = args.into_iter().own_one_and_rest_or_else(|| {
                    EvalErr::InvalidArgs("'define' procedure. expected parameters and body")
                })?;
                let proc = Lambda::new(params.collect::<Vec<Expr>>().to_expr(), body.collect())
                    .to_expr()
                    .into_call()?;
                Define::new(identifier, proc).to_expr().into_call()
            }
            identifier => Define::new(
                identifier,
                body.own_one_or_else(|| {
                    EvalErr::InvalidArgs("'define' expression. expected identifier and value")
                })?,
            )
            .to_expr()
            .into_call(),
        }
    }

    fn parse_assignment(&mut self) -> Result<Expr, EvalErr> {
        let (first, second) = self.parse_inner_call()?.into_iter().own_two_or_else(|| {
            EvalErr::InvalidArgs("'set!' expression. expected identifier and value")
        })?;
        Assignment::new(first, second).to_expr().into_call()
    }

    fn parse_let(&mut self) -> Result<Expr, EvalErr> {
        let (bindings, body) = self
            .parse_inner_call()?
            .into_iter()
            .own_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'let' expression. expected bindings and body")
            })?;
        let res = Ok(let_to_lambda(bindings, body.collect())?.to_expr());
        println!("star: {:?}", res);
        res
    }

    fn parse_letstar(&mut self) -> Result<Expr, EvalErr> {
        let (bindings, body) = self
            .parse_inner_call()?
            .into_iter()
            .own_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'let*' expression. expected bindings and body")
            })?;

        match bindings {
            Expr::Call(bindings) => {
                let res = letstar_to_lambda(&mut bindings.into_iter().peekable(), body.collect());
                println!("star: {:?}", res);
                res
            }
            Expr::EmptyList => {
                letstar_to_lambda(&mut vec![].into_iter().peekable(), body.collect())
            }
            _ => Err(EvalErr::InvalidArgs(
                "'let*' expression. expected list of binding",
            )),
        }
    }

    fn parse_begin(&mut self) -> Result<Expr, EvalErr> {
        Begin::new(self.parse_inner_call()?).to_expr().into_call()
    }

    fn parse_and(&mut self) -> Result<Expr, EvalErr> {
        And::new(self.parse_inner_call()?).to_expr().into_call()
    }

    fn parse_or(&mut self) -> Result<Expr, EvalErr> {
        Or::new(self.parse_inner_call()?).to_expr().into_call()
    }

    // We treat a quoted expression as a normal expression behind an extra indrection, with the
    // addtional major differnce being we parse lists as pairs instead of vectors. this way they
    // can be accessed at runtime rather than evaluated as procedures.
    //
    // TODO: we might consider parsing this in the same way we do non-quoted tokens, but this is a
    // bit more lenient as any token is valid (besides the hanging closing paren).
    fn parse_quote(&mut self) -> Result<Expr, EvalErr> {
        match self.next_or_err(EvalErr::UnexpectedEnd)? {
            Token::LParen => {
                let res = self.parse_inner_quote()?;
                self.tokens.next(); // consume remaining paren
                Ok(res)
            }

            t @ Token::Assignment
            | t @ Token::Lambda
            | t @ Token::Define
            | t @ Token::Let
            | t @ Token::LetStar
            | t @ Token::If
            | t @ Token::And
            | t @ Token::Cond
            | t @ Token::QuoteTick
            | t @ Token::QuoteProc
            | t @ Token::Begin
            | t @ Token::Or => Ok(t.printable().to_expr()),

            x @ Token::Number(_)
            | x @ Token::Str(_)
            | x @ Token::Boolean(_)
            | x @ Token::Symbol(_) => Ok(Expr::Atom(x)),
            p @ Token::RParen => Err(EvalErr::UnexpectedToken(p.printable())),
        }
    }

    fn parse_inner_quote(&mut self) -> Result<Expr, EvalErr> {
        match self.tokens.peek() {
            Some(t) => match t {
                Token::RParen => Ok(Expr::EmptyList),
                _ => {
                    let current = self.parse_quote()?;
                    let next = self.parse_inner_quote()?;
                    Ok(Pair::new(current, next).to_expr())
                }
            },
            None => Err(EvalErr::UnexpectedEnd),
        }
    }

    fn next_or_err(&mut self, err: EvalErr) -> Result<Token, EvalErr> {
        self.tokens.next().map_or_else(|| Err(err), Ok)
    }

    fn peek_or_err(&mut self, err: EvalErr) -> Result<&Token, EvalErr> {
        self.tokens.peek().map_or_else(|| Err(err), Ok)
    }
}

//TODO: handle EmptyList
fn let_to_lambda(bindings: Expr, body: Vec<Expr>) -> Result<Vec<Expr>, EvalErr> {
    match bindings {
        Expr::Call(bindings) => {
            let (params, mut values) = try_unzip_list(bindings)?;
            values.insert(
                0,
                Lambda::new(params.to_expr(), body).to_expr().into_call()?,
            );
            Ok(values)
        }
        expr => Err(EvalErr::TypeError("list", expr.clone())),
    }
}

fn letstar_to_lambda(
    bindings: &mut Peekable<std::vec::IntoIter<Expr>>,
    body: Vec<Expr>,
) -> Result<Expr, EvalErr> {
    match bindings.next() {
        Some(binding) => match letstar_to_lambda(bindings, body)? {
            Expr::Call(body) => make_single_let(binding, body),
            expr => Err(EvalErr::TypeError("list", expr.clone())),
        },
        None => Ok(body.to_expr()),
    }
}

fn make_single_let(binding: Expr, body: Vec<Expr>) -> Result<Expr, EvalErr> {
    match binding {
        Expr::Call(binding) => {
            let (param, val) = binding.into_iter().own_two_or_else(|| {
                EvalErr::InvalidArgs("'let'. expected parameter and value pair")
            })?;
            Ok(vec![
                Lambda::new(param.into_call()?, body)
                    .to_expr()
                    .into_call()?,
                val,
            ]
            .to_expr())
        }
        expr => Err(EvalErr::TypeError("list", expr.clone())),
    }
}

fn cond_to_if(exprs: &mut Peekable<std::vec::IntoIter<Expr>>) -> Result<Expr, EvalErr> {
    match exprs.next() {
        Some(expr) => match expr {
            Expr::Call(expr) => {
                let (predicate, consequence) = expr.into_iter().own_one_and_rest_or_else(|| {
                    EvalErr::InvalidArgs("'cond'. clauses expcted two be lists of two values")
                })?;

                let consequence = Begin::new(consequence.collect()).to_expr().into_call()?;

                if exprs.peek().is_some() {
                    If::new(predicate, consequence, cond_to_if(exprs)?)
                        .to_expr()
                        .into_call()
                } else {
                    match predicate {
                        Expr::Atom(Token::Symbol(s)) if s == "else" => Ok(consequence),
                        _ => If::new(predicate, consequence, cond_to_if(exprs)?)
                            .to_expr()
                            .into_call(),
                    }
                }
            }
            expr => Err(EvalErr::UnexpectedToken(expr.printable())),
        },
        None => Ok(Expr::Void),
    }
}

//TODO: handle EmptyList
fn try_unzip_list(exprs: Vec<Expr>) -> Result<(Vec<Expr>, Vec<Expr>), EvalErr> {
    exprs
        .into_iter()
        .try_fold((vec![], vec![]), |prev, expr_pair| {
            let (mut params, mut values) = prev;
            match expr_pair {
                Expr::Call(binding) => {
                    let (param, value) = binding.into_iter().own_two_or_else(|| {
                        EvalErr::InvalidArgs("'let' expression. expected bindings as pairs")
                    })?;
                    params.push(param);
                    values.push(value);
                    Ok((params, values))
                }
                expr => Err(EvalErr::TypeError("list", expr.clone())),
            }
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::TokenStream;

    #[test]
    fn valid_parse() {
        let scm = "1 (+ 1 (+ 1 2))";
        let res: Vec<Expr> = vec![
            1.0.to_expr(),
            vec![
                "+".to_string().to_expr(),
                1.0.to_expr(),
                vec!["+".to_string().to_expr(), 1.0.to_expr(), 2.0.to_expr()].to_expr(),
            ]
            .to_expr(),
        ];
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        let exprs = Parser::new(tokens).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    fn quoted() {
        let scm = "'(+ 1)";
        let res: Vec<Expr> = vec![Expr::Quoted(Box::new(
            Pair::new(
                "+".to_string().to_expr(),
                Pair::new(1.0.to_expr(), Expr::EmptyList).to_expr(),
            )
            .to_expr(),
        ))];
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        let exprs = Parser::new(tokens).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    fn quoted_fn() {
        let scm = "(quote (+ 1))";
        let res: Vec<Expr> = vec![Expr::Quoted(Box::new(
            Pair::new(
                Expr::Atom(Token::Symbol("+".to_string())),
                Pair::new(1.0.to_expr(), Expr::EmptyList).to_expr(),
            )
            .to_expr(),
        ))];
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        let exprs = Parser::new(tokens).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    #[should_panic]
    fn extra_paren() {
        let scm = "(+ 1 2) (1))";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn opening_rparen() {
        let scm = ")(yo)";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn no_close() {
        let scm = "(+ 1 (1)";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }
}
